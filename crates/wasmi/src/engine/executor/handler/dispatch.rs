use super::{
    exec,
    state::{Inst, Ip, Mem0Len, Mem0Ptr, Sp, Stack, VmState},
    utils::{resolve_instance, set_value},
};
use crate::{
    engine::{
        executor::handler::utils,
        CallParams,
        CallResults,
        CodeMap,
        EngineFunc,
        ResumableHostTrapError,
        ResumableOutOfFuelError,
    },
    ir::{BoundedSlotSpan, OpCode, Slot, SlotSpan},
    CallHook,
    Error,
    Instance,
    Store,
    TrapCode,
};
use core::{marker::PhantomData, ops::ControlFlow};

#[inline(always)]
pub fn fetch_handler(ip: Ip) -> Handler {
    match cfg!(feature = "compact") {
        true => {
            let (_, op_code) = unsafe { ip.decode::<OpCode>() };
            op_code_to_handler(op_code)
        }
        false => {
            let (_, addr) = unsafe { ip.decode::<usize>() };
            unsafe {
                ::core::mem::transmute::<*const (), Handler>(::core::ptr::with_exposed_provenance(
                    addr,
                ))
            }
        }
    }
}

pub struct WasmFuncCall<'a, T, State> {
    store: &'a mut Store<T>,
    stack: &'a mut Stack,
    code: &'a CodeMap,
    callee_ip: Ip,
    callee_sp: Sp,
    instance: Inst,
    state: State,
}

impl<'a, T, State> WasmFuncCall<'a, T, State> {
    fn new_state<NewState>(self, state: NewState) -> WasmFuncCall<'a, T, NewState> {
        WasmFuncCall {
            store: self.store,
            stack: self.stack,
            code: self.code,
            callee_ip: self.callee_ip,
            callee_sp: self.callee_sp,
            instance: self.instance,
            state,
        }
    }
}

mod state {
    use super::Sp;
    use core::marker::PhantomData;

    pub type Uninit = PhantomData<marker::Uninit>;
    pub type Init = PhantomData<marker::Init>;
    pub type Resumed = PhantomData<marker::Resumed>;

    mod marker {
        pub enum Uninit {}
        pub enum Init {}
        pub enum Resumed {}
    }
    pub trait Execute {}
    impl Execute for Init {}
    impl Execute for Resumed {}
    pub struct Done {
        pub sp: Sp,
    }
}

impl<'a, T> WasmFuncCall<'a, T, state::Uninit> {
    pub fn write_params(self, params: impl CallParams) -> WasmFuncCall<'a, T, state::Init> {
        let mut param_slot = Slot::from(0);
        for param_value in params.call_params() {
            set_value(self.callee_sp, param_slot, param_value);
            param_slot = param_slot.next();
        }
        self.new_state(PhantomData)
    }
}

impl<'a, T, State: state::Execute> WasmFuncCall<'a, T, State> {
    pub fn execute(mut self) -> Result<WasmFuncCall<'a, T, state::Done>, ExecutionOutcome> {
        self.store.invoke_call_hook(CallHook::CallingWasm)?;
        let outcome = self.execute_until_done();
        self.store.invoke_call_hook(CallHook::ReturningFromWasm)?;
        let sp = outcome?;
        Ok(self.new_state(state::Done { sp }))
    }

    fn execute_until_done(&mut self) -> Result<Sp, ExecutionOutcome> {
        let store = self.store.prune();
        let (mem0, mem0_len) = utils::extract_mem0(store, self.instance);
        let state = VmState::new(store, self.stack, self.code);
        execute_until_done(
            state,
            self.callee_ip,
            self.callee_sp,
            mem0,
            mem0_len,
            self.instance,
        )
    }
}

impl<'a, T> WasmFuncCall<'a, T, state::Resumed> {
    pub fn provide_host_results(
        self,
        params: impl CallParams,
        slots: SlotSpan,
    ) -> WasmFuncCall<'a, T, state::Init> {
        let mut param_slot = slots.head();
        for param_value in params.call_params() {
            set_value(self.callee_sp, param_slot, param_value);
            param_slot = param_slot.next();
        }
        self.new_state(PhantomData)
    }
}

impl<'a, T> WasmFuncCall<'a, T, state::Done> {
    pub fn write_results<R: CallResults>(self, results: R) -> <R as CallResults>::Results {
        let len_results = results.len_results();
        let sp = self.state.sp;
        let slice = unsafe { sp.as_slice(len_results) };
        results.call_results(slice)
    }
}

pub fn init_wasm_func_call<'a, T>(
    store: &'a mut Store<T>,
    code: &'a CodeMap,
    stack: &'a mut Stack,
    engine_func: EngineFunc,
    instance: Instance,
) -> Result<WasmFuncCall<'a, T, state::Uninit>, Error> {
    let compiled_func = code.get(Some(store.inner.fuel_mut()), engine_func)?;
    let callee_ip = Ip::from(compiled_func.ops());
    let frame_size = compiled_func.len_stack_slots();
    let callee_params = BoundedSlotSpan::new(SlotSpan::new(Slot::from(0)), frame_size);
    let instance = resolve_instance(store.prune(), &instance).into();
    let callee_sp = stack.push_frame(
        None,
        callee_ip,
        callee_params,
        usize::from(frame_size),
        Some(instance),
    )?;
    Ok(WasmFuncCall {
        store,
        stack,
        code,
        callee_ip,
        callee_sp,
        instance,
        state: PhantomData,
    })
}

pub fn resume_wasm_func_call<'a, T>(
    store: &'a mut Store<T>,
    code: &'a CodeMap,
    stack: &'a mut Stack,
) -> Result<WasmFuncCall<'a, T, state::Resumed>, Error> {
    let (callee_ip, callee_sp, instance) = stack.restore_frame();
    Ok(WasmFuncCall {
        store,
        stack,
        code,
        callee_ip,
        callee_sp,
        instance,
        state: PhantomData,
    })
}

#[cfg(feature = "trampolines")]
pub fn execute_until_done(
    mut state: VmState,
    mut ip: Ip,
    mut sp: Sp,
    mut mem0: Mem0Ptr,
    mut mem0_len: Mem0Len,
    mut instance: Inst,
) -> Result<Sp, ExecutionOutcome> {
    let mut handler = fetch_handler(ip);
    'exec: loop {
        match handler(&mut state, ip, sp, mem0, mem0_len, instance) {
            Done::Continue(next) => {
                handler = fetch_handler(next.ip);
                ip = next.ip;
                sp = next.sp;
                mem0 = next.mem0;
                mem0_len = next.mem0_len;
                instance = next.instance;
                continue 'exec;
            }
            Done::Break(reason) => {
                if let Some(trap_code) = reason.trap_code() {
                    return Err(ExecutionOutcome::from(trap_code));
                }
                break 'exec;
            }
        }
    }
    state.into_execution_outcome()
}

#[cfg(not(feature = "trampolines"))]
pub fn execute_until_done(
    state: VmState,
    ip: Ip,
    sp: Sp,
    mem0: Mem0Ptr,
    mem0_len: Mem0Len,
    instance: Inst,
) -> Result<Sp, ExecutionOutcome> {
    let mut state = state;
    let handler = fetch_handler(ip);
    let Control::Break(reason) = handler(&mut state, ip, sp, mem0, mem0_len, instance);
    if let Some(trap_code) = reason.trap_code() {
        return Err(ExecutionOutcome::from(trap_code));
    }
    state.into_execution_outcome()
}

#[derive(Debug)]
pub enum ExecutionOutcome {
    Host(ResumableHostTrapError),
    OutOfFuel(ResumableOutOfFuelError),
    Error(Error),
}

impl From<ExecutionOutcome> for Error {
    fn from(error: ExecutionOutcome) -> Self {
        match error {
            ExecutionOutcome::Host(error) => error.into(),
            ExecutionOutcome::OutOfFuel(error) => error.into(),
            ExecutionOutcome::Error(error) => error,
        }
    }
}

impl From<ResumableHostTrapError> for ExecutionOutcome {
    fn from(error: ResumableHostTrapError) -> Self {
        Self::Host(error)
    }
}

impl From<ResumableOutOfFuelError> for ExecutionOutcome {
    fn from(error: ResumableOutOfFuelError) -> Self {
        Self::OutOfFuel(error)
    }
}

impl From<TrapCode> for ExecutionOutcome {
    fn from(error: TrapCode) -> Self {
        Self::Error(error.into())
    }
}

impl From<Error> for ExecutionOutcome {
    fn from(error: Error) -> Self {
        Self::Error(error)
    }
}

#[cfg(not(feature = "trampolines"))]
#[derive(Debug)]
pub enum NextState {}

#[cfg(feature = "trampolines")]
#[derive(Debug, Copy, Clone)]
pub struct NextState {
    ip: Ip,
    sp: Sp,
    mem0: Mem0Ptr,
    mem0_len: Mem0Len,
    instance: Inst,
}

#[derive(Debug, Copy, Clone)]
pub enum Break {
    UnreachableCodeReached = TrapCode::UnreachableCodeReached as _,
    MemoryOutOfBounds = TrapCode::MemoryOutOfBounds as _,
    TableOutOfBounds = TrapCode::TableOutOfBounds as _,
    IndirectCallToNull = TrapCode::IndirectCallToNull as _,
    IntegerDivisionByZero = TrapCode::IntegerDivisionByZero as _,
    IntegerOverflow = TrapCode::IntegerOverflow as _,
    BadConversionToInteger = TrapCode::BadConversionToInteger as _,
    StackOverflow = TrapCode::StackOverflow as _,
    BadSignature = TrapCode::BadSignature as _,
    OutOfFuel = TrapCode::OutOfFuel as _,
    GrowthOperationLimited = TrapCode::GrowthOperationLimited as _,
    OutOfSystemMemory = TrapCode::OutOfSystemMemory as _,
    /// Signals that there must be a reason stored externally supplying the caller with more information.
    WithReason,
}

impl From<TrapCode> for Break {
    #[inline]
    fn from(trap_code: TrapCode) -> Self {
        match trap_code {
            TrapCode::UnreachableCodeReached => Self::UnreachableCodeReached,
            TrapCode::MemoryOutOfBounds => Self::MemoryOutOfBounds,
            TrapCode::TableOutOfBounds => Self::TableOutOfBounds,
            TrapCode::IndirectCallToNull => Self::IndirectCallToNull,
            TrapCode::IntegerDivisionByZero => Self::IntegerDivisionByZero,
            TrapCode::IntegerOverflow => Self::IntegerOverflow,
            TrapCode::BadConversionToInteger => Self::BadConversionToInteger,
            TrapCode::StackOverflow => Self::StackOverflow,
            TrapCode::BadSignature => Self::BadSignature,
            TrapCode::OutOfFuel => Self::OutOfFuel,
            TrapCode::GrowthOperationLimited => Self::GrowthOperationLimited,
            TrapCode::OutOfSystemMemory => Self::OutOfSystemMemory,
        }
    }
}

impl Break {
    #[inline]
    pub fn trap_code(self) -> Option<TrapCode> {
        let trap_code = match self {
            Self::UnreachableCodeReached => TrapCode::UnreachableCodeReached,
            Self::MemoryOutOfBounds => TrapCode::MemoryOutOfBounds,
            Self::TableOutOfBounds => TrapCode::TableOutOfBounds,
            Self::IndirectCallToNull => TrapCode::IndirectCallToNull,
            Self::IntegerDivisionByZero => TrapCode::IntegerDivisionByZero,
            Self::IntegerOverflow => TrapCode::IntegerOverflow,
            Self::BadConversionToInteger => TrapCode::BadConversionToInteger,
            Self::StackOverflow => TrapCode::StackOverflow,
            Self::BadSignature => TrapCode::BadSignature,
            Self::OutOfFuel => TrapCode::OutOfFuel,
            Self::GrowthOperationLimited => TrapCode::GrowthOperationLimited,
            Self::OutOfSystemMemory => TrapCode::OutOfSystemMemory,
            _ => return None,
        };
        Some(trap_code)
    }
}

pub type Control<C = (), B = Break> = ControlFlow<B, C>;
pub type Done<T = NextState> = Control<T, Break>;

#[cfg(feature = "trampolines")]
pub trait ControlContinue: Sized {
    fn control_continue(ip: Ip, sp: Sp, mem0: Mem0Ptr, mem0_len: Mem0Len, instance: Inst) -> Self;
}

#[cfg(feature = "trampolines")]
impl ControlContinue for Done<NextState> {
    fn control_continue(ip: Ip, sp: Sp, mem0: Mem0Ptr, mem0_len: Mem0Len, instance: Inst) -> Self {
        Self::Continue(NextState {
            ip,
            sp,
            mem0,
            mem0_len,
            instance,
        })
    }
}

pub trait ControlBreak: Sized {
    fn control_break() -> Self;
}

impl<T> ControlBreak for Done<T> {
    fn control_break() -> Self {
        Self::Break(Break::WithReason)
    }
}

type Handler =
    fn(&mut VmState, ip: Ip, sp: Sp, mem0: Mem0Ptr, mem0_len: Mem0Len, instance: Inst) -> Done;

macro_rules! compile_or_get_func {
    ($state:expr, $func:expr) => {{
        match $crate::engine::executor::handler::utils::compile_or_get_func($state, $func) {
            Ok((ip, size)) => (ip, size),
            Err(error) => done!($state, DoneReason::error(error)),
        }
    }};
}

macro_rules! trap {
    ($trap_code:expr) => {{
        return $crate::engine::executor::handler::Control::Break(
            $crate::engine::executor::handler::Break::from($trap_code),
        );
    }};
}

macro_rules! done {
    ($state:expr, $reason:expr $(,)? ) => {{
        $state.done_with(move || <_ as ::core::convert::Into<
            $crate::engine::executor::handler::DoneReason,
        >>::into($reason));
        return <$crate::engine::executor::handler::Done<_> as $crate::engine::executor::handler::ControlBreak>::control_break();
    }};
}

#[cfg(not(feature = "trampolines"))]
macro_rules! dispatch {
    ($state:expr, $ip:expr, $sp:expr, $mem0:expr, $mem0_len:expr, $instance:expr) => {{
        let handler = $crate::engine::executor::handler::dispatch::fetch_handler($ip);
        return handler($state, $ip, $sp, $mem0, $mem0_len, $instance);
    }};
}

#[cfg(feature = "trampolines")]
macro_rules! dispatch {
    ($state:expr, $ip:expr, $sp:expr, $mem0:expr, $mem0_len:expr, $instance:expr) => {{
        let _: &mut VmState = $state;
        return <$crate::engine::executor::handler::dispatch::Done as $crate::engine::executor::handler::ControlContinue>::control_continue(
            $ip, $sp, $mem0, $mem0_len, $instance,
        );
    }};
}

#[inline(always)]
pub fn op_code_to_handler(code: OpCode) -> Handler {
    match code {
        // misc
        OpCode::Trap => exec::trap,
        OpCode::ConsumeFuel => exec::consume_fuel,
        // copy
        OpCode::Copy => exec::copy,
        OpCode::Copy32 => exec::copy32,
        OpCode::Copy64 => exec::copy64,
        OpCode::CopySpanAsc => exec::copy_span_asc,
        OpCode::CopySpanDes => exec::copy_span_des,
        // global
        OpCode::GlobalGet => exec::global_get,
        OpCode::GlobalSet => exec::global_set,
        OpCode::GlobalSet32 => exec::global_set_32,
        OpCode::GlobalSet64 => exec::global_set_64,
        // return
        OpCode::Return => exec::r#return,
        OpCode::Return32 => exec::return32,
        OpCode::Return64 => exec::return64,
        OpCode::ReturnSlot => exec::return_slot,
        OpCode::ReturnSpan => exec::return_span,
        // call
        OpCode::CallInternal => exec::call_internal,
        OpCode::CallImported => exec::call_imported,
        OpCode::CallIndirect => exec::call_indirect,
        OpCode::ReturnCallInternal => exec::return_call_internal,
        OpCode::ReturnCallImported => exec::return_call_imported,
        OpCode::ReturnCallIndirect => exec::return_call_indirect,
        // memory
        OpCode::MemorySize => exec::memory_size,
        OpCode::MemoryGrow => exec::memory_grow,
        OpCode::MemoryCopy => exec::memory_copy,
        OpCode::MemoryFill => exec::memory_fill,
        OpCode::MemoryInit => exec::memory_init,
        OpCode::DataDrop => exec::data_drop,
        // table
        OpCode::TableSize => exec::table_size,
        OpCode::TableGrow => exec::table_grow,
        OpCode::TableCopy => exec::table_copy,
        OpCode::TableFill => exec::table_fill,
        OpCode::TableInit => exec::table_init,
        OpCode::TableGet_Ss => exec::table_get_ss,
        OpCode::TableGet_Si => exec::table_get_si,
        OpCode::TableSet_Ss => exec::table_set_ss,
        OpCode::TableSet_Si => exec::table_set_si,
        OpCode::TableSet_Is => exec::table_set_is,
        OpCode::TableSet_Ii => exec::table_set_ii,
        OpCode::ElemDrop => exec::elem_drop,
        OpCode::RefFunc => exec::ref_func,
        // wide-arithmetic
        OpCode::I64Add128 => exec::i64_add128,
        OpCode::I64Sub128 => exec::i64_sub128,
        OpCode::I64MulWide => exec::i64_mul_wide,
        OpCode::U64MulWide => exec::u64_mul_wide,
        // unconditional branch
        OpCode::Branch => exec::branch,
        OpCode::BranchTable => exec::branch_table,
        OpCode::BranchTableSpan => exec::branch_table_span,
        // unary
        OpCode::I32Popcnt_Ss => exec::i32_popcnt_ss,
        OpCode::I32Ctz_Ss => exec::i32_ctz_ss,
        OpCode::I32Clz_Ss => exec::i32_clz_ss,
        OpCode::I32Sext8_Ss => exec::i32_sext8_ss,
        OpCode::I32Sext16_Ss => exec::i32_sext16_ss,
        OpCode::I32WrapI64_Ss => exec::i32_wrap_i64,
        OpCode::I64Popcnt_Ss => exec::i64_popcnt_ss,
        OpCode::I64Ctz_Ss => exec::i64_ctz_ss,
        OpCode::I64Clz_Ss => exec::i64_clz_ss,
        OpCode::I64Sext8_Ss => exec::i64_sext8_ss,
        OpCode::I64Sext16_Ss => exec::i64_sext16_ss,
        OpCode::I64Sext32_Ss => exec::i64_sext32_ss,
        OpCode::F32Abs_Ss => exec::f32_abs_ss,
        OpCode::F32Neg_Ss => exec::f32_neg_ss,
        OpCode::F32Ceil_Ss => exec::f32_ceil_ss,
        OpCode::F32Floor_Ss => exec::f32_floor_ss,
        OpCode::F32Trunc_Ss => exec::f32_trunc_ss,
        OpCode::F32Nearest_Ss => exec::f32_nearest_ss,
        OpCode::F32Sqrt_Ss => exec::f32_sqrt_ss,
        OpCode::F32ConvertI32_Ss => exec::f32_convert_i32_ss,
        OpCode::F32ConvertU32_Ss => exec::f32_convert_u32_ss,
        OpCode::F32ConvertI64_Ss => exec::f32_convert_i64_ss,
        OpCode::F32ConvertU64_Ss => exec::f32_convert_u64_ss,
        OpCode::F32DemoteF64_Ss => exec::f32_demote_f64_ss,
        OpCode::F64Abs_Ss => exec::f64_abs_ss,
        OpCode::F64Neg_Ss => exec::f64_neg_ss,
        OpCode::F64Ceil_Ss => exec::f64_ceil_ss,
        OpCode::F64Floor_Ss => exec::f64_floor_ss,
        OpCode::F64Trunc_Ss => exec::f64_trunc_ss,
        OpCode::F64Nearest_Ss => exec::f64_nearest_ss,
        OpCode::F64Sqrt_Ss => exec::f64_sqrt_ss,
        OpCode::F64ConvertI32_Ss => exec::f64_convert_i32_ss,
        OpCode::F64ConvertU32_Ss => exec::f64_convert_u32_ss,
        OpCode::F64ConvertI64_Ss => exec::f64_convert_i64_ss,
        OpCode::F64ConvertU64_Ss => exec::f64_convert_u64_ss,
        OpCode::F64PromoteF32_Ss => exec::f64_demote_f64_ss,
        OpCode::I32TruncF32_Ss => exec::i32_trunc_f32,
        OpCode::U32TruncF32_Ss => exec::u32_trunc_f32,
        OpCode::I32TruncF64_Ss => exec::i32_trunc_f64,
        OpCode::U32TruncF64_Ss => exec::u32_trunc_f64,
        OpCode::I64TruncF32_Ss => exec::i64_trunc_f32,
        OpCode::U64TruncF32_Ss => exec::u64_trunc_f32,
        OpCode::I64TruncF64_Ss => exec::i64_trunc_f64,
        OpCode::U64TruncF64_Ss => exec::u64_trunc_f64,
        OpCode::I32TruncSatF32_Ss => exec::i32_trunc_sat_f32,
        OpCode::U32TruncSatF32_Ss => exec::u32_trunc_sat_f32,
        OpCode::I32TruncSatF64_Ss => exec::i32_trunc_sat_f64,
        OpCode::U32TruncSatF64_Ss => exec::u32_trunc_sat_f64,
        OpCode::I64TruncSatF32_Ss => exec::i64_trunc_sat_f32,
        OpCode::U64TruncSatF32_Ss => exec::u64_trunc_sat_f32,
        OpCode::I64TruncSatF64_Ss => exec::i64_trunc_sat_f64,
        OpCode::U64TruncSatF64_Ss => exec::u64_trunc_sat_f64,
        // binary
        // i32
        OpCode::I32Eq_Sss => exec::i32_eq_sss,
        OpCode::I32Eq_Ssi => exec::i32_eq_ssi,
        OpCode::I32And_Sss => exec::i32_and_sss,
        OpCode::I32And_Ssi => exec::i32_and_ssi,
        OpCode::I32Or_Sss => exec::i32_or_sss,
        OpCode::I32Or_Ssi => exec::i32_or_ssi,
        OpCode::I32NotEq_Sss => exec::i32_not_eq_sss,
        OpCode::I32NotEq_Ssi => exec::i32_not_eq_ssi,
        OpCode::I32NotAnd_Sss => exec::i32_not_and_sss,
        OpCode::I32NotAnd_Ssi => exec::i32_not_and_ssi,
        OpCode::I32NotOr_Sss => exec::i32_not_or_sss,
        OpCode::I32NotOr_Ssi => exec::i32_not_or_ssi,
        OpCode::I32Add_Sss => exec::i32_add_sss,
        OpCode::I32Add_Ssi => exec::i32_add_ssi,
        OpCode::I32Mul_Sss => exec::i32_mul_sss,
        OpCode::I32Mul_Ssi => exec::i32_mul_ssi,
        OpCode::I32BitAnd_Sss => exec::i32_bitand_sss,
        OpCode::I32BitAnd_Ssi => exec::i32_bitand_ssi,
        OpCode::I32BitOr_Sss => exec::i32_bitor_sss,
        OpCode::I32BitOr_Ssi => exec::i32_bitor_ssi,
        OpCode::I32BitXor_Sss => exec::i32_bitxor_sss,
        OpCode::I32BitXor_Ssi => exec::i32_bitxor_ssi,
        OpCode::I32Sub_Sss => exec::i32_sub_sss,
        OpCode::I32Sub_Ssi => exec::i32_sub_ssi,
        OpCode::I32Sub_Sis => exec::i32_sub_sis,
        OpCode::I32Div_Sss => exec::i32_div_sss,
        OpCode::I32Div_Ssi => exec::i32_div_ssi,
        OpCode::I32Div_Sis => exec::i32_div_sis,
        OpCode::U32Div_Sss => exec::u32_div_sss,
        OpCode::U32Div_Ssi => exec::u32_div_ssi,
        OpCode::U32Div_Sis => exec::u32_div_sis,
        OpCode::I32Rem_Sss => exec::i32_rem_sss,
        OpCode::I32Rem_Ssi => exec::i32_rem_ssi,
        OpCode::I32Rem_Sis => exec::i32_rem_sis,
        OpCode::U32Rem_Sss => exec::u32_rem_sss,
        OpCode::U32Rem_Ssi => exec::u32_rem_ssi,
        OpCode::U32Rem_Sis => exec::u32_rem_sis,
        OpCode::I32Le_Sss => exec::i32_le_sss,
        OpCode::I32Le_Ssi => exec::i32_le_ssi,
        OpCode::I32Le_Sis => exec::i32_le_sis,
        OpCode::I32Lt_Sss => exec::i32_lt_sss,
        OpCode::I32Lt_Ssi => exec::i32_lt_ssi,
        OpCode::I32Lt_Sis => exec::i32_lt_sis,
        OpCode::U32Le_Sss => exec::u32_le_sss,
        OpCode::U32Le_Ssi => exec::u32_le_ssi,
        OpCode::U32Le_Sis => exec::u32_le_sis,
        OpCode::U32Lt_Sss => exec::u32_lt_sss,
        OpCode::U32Lt_Ssi => exec::u32_lt_ssi,
        OpCode::U32Lt_Sis => exec::u32_lt_sis,
        OpCode::I32Shl_Sss => exec::i32_shl_sss,
        OpCode::I32Shl_Ssi => exec::i32_shl_ssi,
        OpCode::I32Shl_Sis => exec::i32_shl_sis,
        OpCode::I32Shr_Sss => exec::i32_shr_sss,
        OpCode::I32Shr_Ssi => exec::i32_shr_ssi,
        OpCode::I32Shr_Sis => exec::i32_shr_sis,
        OpCode::U32Shr_Sss => exec::u32_shr_sss,
        OpCode::U32Shr_Ssi => exec::u32_shr_ssi,
        OpCode::U32Shr_Sis => exec::u32_shr_sis,
        OpCode::I32Rotl_Sss => exec::i32_rotl_sss,
        OpCode::I32Rotl_Ssi => exec::i32_rotl_ssi,
        OpCode::I32Rotl_Sis => exec::i32_rotl_sis,
        OpCode::I32Rotr_Sss => exec::i32_rotr_sss,
        OpCode::I32Rotr_Ssi => exec::i32_rotr_ssi,
        OpCode::I32Rotr_Sis => exec::i32_rotr_sis,
        // binary
        // i64
        OpCode::I64Eq_Sss => exec::i64_eq_sss,
        OpCode::I64Eq_Ssi => exec::i64_eq_ssi,
        OpCode::I64And_Sss => exec::i64_and_sss,
        OpCode::I64And_Ssi => exec::i64_and_ssi,
        OpCode::I64Or_Sss => exec::i64_or_sss,
        OpCode::I64Or_Ssi => exec::i64_or_ssi,
        OpCode::I64NotEq_Sss => exec::i64_not_eq_sss,
        OpCode::I64NotEq_Ssi => exec::i64_not_eq_ssi,
        OpCode::I64NotAnd_Sss => exec::i64_not_and_sss,
        OpCode::I64NotAnd_Ssi => exec::i64_not_and_ssi,
        OpCode::I64NotOr_Sss => exec::i64_not_or_sss,
        OpCode::I64NotOr_Ssi => exec::i64_not_or_ssi,
        OpCode::I64Add_Sss => exec::i64_add_sss,
        OpCode::I64Add_Ssi => exec::i64_add_ssi,
        OpCode::I64Mul_Sss => exec::i64_mul_sss,
        OpCode::I64Mul_Ssi => exec::i64_mul_ssi,
        OpCode::I64BitAnd_Sss => exec::i64_bitand_sss,
        OpCode::I64BitAnd_Ssi => exec::i64_bitand_ssi,
        OpCode::I64BitOr_Sss => exec::i64_bitor_sss,
        OpCode::I64BitOr_Ssi => exec::i64_bitor_ssi,
        OpCode::I64BitXor_Sss => exec::i64_bitxor_sss,
        OpCode::I64BitXor_Ssi => exec::i64_bitxor_ssi,
        OpCode::I64Sub_Sss => exec::i64_sub_sss,
        OpCode::I64Sub_Ssi => exec::i64_sub_ssi,
        OpCode::I64Sub_Sis => exec::i64_sub_sis,
        OpCode::I64Div_Sss => exec::i64_div_sss,
        OpCode::I64Div_Ssi => exec::i64_div_ssi,
        OpCode::I64Div_Sis => exec::i64_div_sis,
        OpCode::U64Div_Sss => exec::u64_div_sss,
        OpCode::U64Div_Ssi => exec::u64_div_ssi,
        OpCode::U64Div_Sis => exec::u64_div_sis,
        OpCode::I64Rem_Sss => exec::i64_rem_sss,
        OpCode::I64Rem_Ssi => exec::i64_rem_ssi,
        OpCode::I64Rem_Sis => exec::i64_rem_sis,
        OpCode::U64Rem_Sss => exec::u64_rem_sss,
        OpCode::U64Rem_Ssi => exec::u64_rem_ssi,
        OpCode::U64Rem_Sis => exec::u64_rem_sis,
        OpCode::I64Le_Sss => exec::i64_le_sss,
        OpCode::I64Le_Ssi => exec::i64_le_ssi,
        OpCode::I64Le_Sis => exec::i64_le_sis,
        OpCode::I64Lt_Sss => exec::i64_lt_sss,
        OpCode::I64Lt_Ssi => exec::i64_lt_ssi,
        OpCode::I64Lt_Sis => exec::i64_lt_sis,
        OpCode::U64Le_Sss => exec::u64_le_sss,
        OpCode::U64Le_Ssi => exec::u64_le_ssi,
        OpCode::U64Le_Sis => exec::u64_le_sis,
        OpCode::U64Lt_Sss => exec::u64_lt_sss,
        OpCode::U64Lt_Ssi => exec::u64_lt_ssi,
        OpCode::U64Lt_Sis => exec::u64_lt_sis,
        OpCode::I64Shl_Sss => exec::i64_shl_sss,
        OpCode::I64Shl_Ssi => exec::i64_shl_ssi,
        OpCode::I64Shl_Sis => exec::i64_shl_sis,
        OpCode::I64Shr_Sss => exec::i64_shr_sss,
        OpCode::I64Shr_Ssi => exec::i64_shr_ssi,
        OpCode::I64Shr_Sis => exec::i64_shr_sis,
        OpCode::U64Shr_Sss => exec::u64_shr_sss,
        OpCode::U64Shr_Ssi => exec::u64_shr_ssi,
        OpCode::U64Shr_Sis => exec::u64_shr_sis,
        OpCode::I64Rotl_Sss => exec::i64_rotl_sss,
        OpCode::I64Rotl_Ssi => exec::i64_rotl_ssi,
        OpCode::I64Rotl_Sis => exec::i64_rotl_sis,
        OpCode::I64Rotr_Sss => exec::i64_rotr_sss,
        OpCode::I64Rotr_Ssi => exec::i64_rotr_ssi,
        OpCode::I64Rotr_Sis => exec::i64_rotr_sis,
        // f32
        // f32: binary
        OpCode::F32Add_Sss => exec::f32_add_sss,
        OpCode::F32Add_Ssi => exec::f32_add_ssi,
        OpCode::F32Add_Sis => exec::f32_add_sis,
        OpCode::F32Sub_Sss => exec::f32_sub_sss,
        OpCode::F32Sub_Ssi => exec::f32_sub_ssi,
        OpCode::F32Sub_Sis => exec::f32_sub_sis,
        OpCode::F32Mul_Sss => exec::f32_mul_sss,
        OpCode::F32Mul_Ssi => exec::f32_mul_ssi,
        OpCode::F32Mul_Sis => exec::f32_mul_sis,
        OpCode::F32Div_Sss => exec::f32_div_sss,
        OpCode::F32Div_Ssi => exec::f32_div_ssi,
        OpCode::F32Div_Sis => exec::f32_div_sis,
        OpCode::F32Min_Sss => exec::f32_min_sss,
        OpCode::F32Min_Ssi => exec::f32_min_ssi,
        OpCode::F32Min_Sis => exec::f32_min_sis,
        OpCode::F32Max_Sss => exec::f32_max_sss,
        OpCode::F32Max_Ssi => exec::f32_max_ssi,
        OpCode::F32Max_Sis => exec::f32_max_sis,
        OpCode::F32Copysign_Sss => exec::f32_copysign_sss,
        OpCode::F32Copysign_Ssi => exec::f32_copysign_ssi,
        OpCode::F32Copysign_Sis => exec::f32_copysign_sis,
        OpCode::F32Eq_Sss => exec::f32_eq_sss,
        OpCode::F32Eq_Ssi => exec::f32_eq_ssi,
        OpCode::F32Lt_Sss => exec::f32_lt_sss,
        OpCode::F32Lt_Ssi => exec::f32_lt_ssi,
        OpCode::F32Lt_Sis => exec::f32_lt_sis,
        OpCode::F32Le_Sss => exec::f32_le_sss,
        OpCode::F32Le_Ssi => exec::f32_le_ssi,
        OpCode::F32Le_Sis => exec::f32_le_sis,
        OpCode::F32NotEq_Sss => exec::f32_not_eq_sss,
        OpCode::F32NotEq_Ssi => exec::f32_not_eq_ssi,
        OpCode::F32NotLt_Sss => exec::f32_not_lt_sss,
        OpCode::F32NotLt_Ssi => exec::f32_not_lt_ssi,
        OpCode::F32NotLt_Sis => exec::f32_not_lt_sis,
        OpCode::F32NotLe_Sss => exec::f32_not_le_sss,
        OpCode::F32NotLe_Ssi => exec::f32_not_le_ssi,
        OpCode::F32NotLe_Sis => exec::f32_not_le_sis,
        // f64
        // f64: binary
        OpCode::F64Add_Sss => exec::f64_add_sss,
        OpCode::F64Add_Ssi => exec::f64_add_ssi,
        OpCode::F64Add_Sis => exec::f64_add_sis,
        OpCode::F64Sub_Sss => exec::f64_sub_sss,
        OpCode::F64Sub_Ssi => exec::f64_sub_ssi,
        OpCode::F64Sub_Sis => exec::f64_sub_sis,
        OpCode::F64Mul_Sss => exec::f64_mul_sss,
        OpCode::F64Mul_Ssi => exec::f64_mul_ssi,
        OpCode::F64Mul_Sis => exec::f64_mul_sis,
        OpCode::F64Div_Sss => exec::f64_div_sss,
        OpCode::F64Div_Ssi => exec::f64_div_ssi,
        OpCode::F64Div_Sis => exec::f64_div_sis,
        OpCode::F64Min_Sss => exec::f64_min_sss,
        OpCode::F64Min_Ssi => exec::f64_min_ssi,
        OpCode::F64Min_Sis => exec::f64_min_sis,
        OpCode::F64Max_Sss => exec::f64_max_sss,
        OpCode::F64Max_Ssi => exec::f64_max_ssi,
        OpCode::F64Max_Sis => exec::f64_max_sis,
        OpCode::F64Copysign_Sss => exec::f64_copysign_sss,
        OpCode::F64Copysign_Ssi => exec::f64_copysign_ssi,
        OpCode::F64Copysign_Sis => exec::f64_copysign_sis,
        OpCode::F64Eq_Sss => exec::f64_eq_sss,
        OpCode::F64Eq_Ssi => exec::f64_eq_ssi,
        OpCode::F64Lt_Sss => exec::f64_lt_sss,
        OpCode::F64Lt_Ssi => exec::f64_lt_ssi,
        OpCode::F64Lt_Sis => exec::f64_lt_sis,
        OpCode::F64Le_Sss => exec::f64_le_sss,
        OpCode::F64Le_Ssi => exec::f64_le_ssi,
        OpCode::F64Le_Sis => exec::f64_le_sis,
        OpCode::F64NotEq_Sss => exec::f64_not_eq_sss,
        OpCode::F64NotEq_Ssi => exec::f64_not_eq_ssi,
        OpCode::F64NotLt_Sss => exec::f64_not_lt_sss,
        OpCode::F64NotLt_Ssi => exec::f64_not_lt_ssi,
        OpCode::F64NotLt_Sis => exec::f64_not_lt_sis,
        OpCode::F64NotLe_Sss => exec::f64_not_le_sss,
        OpCode::F64NotLe_Ssi => exec::f64_not_le_ssi,
        OpCode::F64NotLe_Sis => exec::f64_not_le_sis,
        // cmp+branch
        OpCode::BranchI32Eq_Ss => exec::branch_i32_eq_ss,
        OpCode::BranchI32Eq_Si => exec::branch_i32_eq_si,
        OpCode::BranchI32And_Ss => exec::branch_i32_and_ss,
        OpCode::BranchI32And_Si => exec::branch_i32_and_si,
        OpCode::BranchI32Or_Ss => exec::branch_i32_or_ss,
        OpCode::BranchI32Or_Si => exec::branch_i32_or_si,
        OpCode::BranchI32NotEq_Ss => exec::branch_i32_not_eq_ss,
        OpCode::BranchI32NotEq_Si => exec::branch_i32_not_eq_si,
        OpCode::BranchI32NotAnd_Ss => exec::branch_i32_not_and_ss,
        OpCode::BranchI32NotAnd_Si => exec::branch_i32_not_and_si,
        OpCode::BranchI32NotOr_Ss => exec::branch_i32_not_or_ss,
        OpCode::BranchI32NotOr_Si => exec::branch_i32_not_or_si,
        OpCode::BranchI32Le_Ss => exec::branch_i32_le_ss,
        OpCode::BranchI32Le_Si => exec::branch_i32_le_si,
        OpCode::BranchI32Le_Is => exec::branch_i32_le_is,
        OpCode::BranchI32Lt_Ss => exec::branch_i32_lt_ss,
        OpCode::BranchI32Lt_Si => exec::branch_i32_lt_si,
        OpCode::BranchI32Lt_Is => exec::branch_i32_lt_is,
        OpCode::BranchU32Le_Ss => exec::branch_u32_le_ss,
        OpCode::BranchU32Le_Si => exec::branch_u32_le_si,
        OpCode::BranchU32Le_Is => exec::branch_u32_le_is,
        OpCode::BranchU32Lt_Ss => exec::branch_u32_lt_ss,
        OpCode::BranchU32Lt_Si => exec::branch_u32_lt_si,
        OpCode::BranchU32Lt_Is => exec::branch_u32_lt_is,
        OpCode::BranchI64Eq_Ss => exec::branch_i64_eq_ss,
        OpCode::BranchI64Eq_Si => exec::branch_i64_eq_si,
        OpCode::BranchI64And_Ss => exec::branch_i64_and_ss,
        OpCode::BranchI64And_Si => exec::branch_i64_and_si,
        OpCode::BranchI64Or_Ss => exec::branch_i64_or_ss,
        OpCode::BranchI64Or_Si => exec::branch_i64_or_si,
        OpCode::BranchI64NotEq_Ss => exec::branch_i64_not_eq_ss,
        OpCode::BranchI64NotEq_Si => exec::branch_i64_not_eq_si,
        OpCode::BranchI64NotAnd_Ss => exec::branch_i64_not_and_ss,
        OpCode::BranchI64NotAnd_Si => exec::branch_i64_not_and_si,
        OpCode::BranchI64NotOr_Ss => exec::branch_i64_not_or_ss,
        OpCode::BranchI64NotOr_Si => exec::branch_i64_not_or_si,
        OpCode::BranchI64Le_Ss => exec::branch_i64_le_ss,
        OpCode::BranchI64Le_Si => exec::branch_i64_le_si,
        OpCode::BranchI64Le_Is => exec::branch_i64_le_is,
        OpCode::BranchI64Lt_Ss => exec::branch_i64_lt_ss,
        OpCode::BranchI64Lt_Si => exec::branch_i64_lt_si,
        OpCode::BranchI64Lt_Is => exec::branch_i64_lt_is,
        OpCode::BranchU64Le_Ss => exec::branch_u64_le_ss,
        OpCode::BranchU64Le_Si => exec::branch_u64_le_si,
        OpCode::BranchU64Le_Is => exec::branch_u64_le_is,
        OpCode::BranchU64Lt_Ss => exec::branch_u64_lt_ss,
        OpCode::BranchU64Lt_Si => exec::branch_u64_lt_si,
        OpCode::BranchU64Lt_Is => exec::branch_u64_lt_is,
        OpCode::BranchF32Eq_Ss => exec::branch_f32_eq_ss,
        OpCode::BranchF32Eq_Si => exec::branch_f32_eq_si,
        OpCode::BranchF32Le_Ss => exec::branch_f32_le_ss,
        OpCode::BranchF32Le_Si => exec::branch_f32_le_si,
        OpCode::BranchF32Le_Is => exec::branch_f32_le_is,
        OpCode::BranchF32Lt_Ss => exec::branch_f32_lt_ss,
        OpCode::BranchF32Lt_Si => exec::branch_f32_lt_si,
        OpCode::BranchF32Lt_Is => exec::branch_f32_lt_is,
        OpCode::BranchF32NotEq_Ss => exec::branch_f32_not_eq_ss,
        OpCode::BranchF32NotEq_Si => exec::branch_f32_not_eq_si,
        OpCode::BranchF32NotLe_Ss => exec::branch_f32_not_le_ss,
        OpCode::BranchF32NotLe_Si => exec::branch_f32_not_le_si,
        OpCode::BranchF32NotLe_Is => exec::branch_f32_not_le_is,
        OpCode::BranchF32NotLt_Ss => exec::branch_f32_not_lt_ss,
        OpCode::BranchF32NotLt_Si => exec::branch_f32_not_lt_si,
        OpCode::BranchF32NotLt_Is => exec::branch_f32_not_lt_is,
        OpCode::BranchF64Eq_Ss => exec::branch_f64_eq_ss,
        OpCode::BranchF64Eq_Si => exec::branch_f64_eq_si,
        OpCode::BranchF64Le_Ss => exec::branch_f64_le_ss,
        OpCode::BranchF64Le_Si => exec::branch_f64_le_si,
        OpCode::BranchF64Le_Is => exec::branch_f64_le_is,
        OpCode::BranchF64Lt_Ss => exec::branch_f64_lt_ss,
        OpCode::BranchF64Lt_Si => exec::branch_f64_lt_si,
        OpCode::BranchF64Lt_Is => exec::branch_f64_lt_is,
        OpCode::BranchF64NotEq_Ss => exec::branch_f64_not_eq_ss,
        OpCode::BranchF64NotEq_Si => exec::branch_f64_not_eq_si,
        OpCode::BranchF64NotLe_Ss => exec::branch_f64_not_le_ss,
        OpCode::BranchF64NotLe_Si => exec::branch_f64_not_le_si,
        OpCode::BranchF64NotLe_Is => exec::branch_f64_not_le_is,
        OpCode::BranchF64NotLt_Ss => exec::branch_f64_not_lt_ss,
        OpCode::BranchF64NotLt_Si => exec::branch_f64_not_lt_si,
        OpCode::BranchF64NotLt_Is => exec::branch_f64_not_lt_is,
        // select
        OpCode::SelectI32Eq_Sss => exec::select_i32_eq_sss,
        OpCode::SelectI32Eq_Ssi => exec::select_i32_eq_ssi,
        OpCode::SelectI32And_Sss => exec::select_i32_and_sss,
        OpCode::SelectI32And_Ssi => exec::select_i32_and_ssi,
        OpCode::SelectI32Or_Sss => exec::select_i32_or_sss,
        OpCode::SelectI32Or_Ssi => exec::select_i32_or_ssi,
        OpCode::SelectI32Le_Sss => exec::select_i32_le_sss,
        OpCode::SelectI32Le_Ssi => exec::select_i32_le_ssi,
        OpCode::SelectI32Lt_Sss => exec::select_i32_lt_sss,
        OpCode::SelectI32Lt_Ssi => exec::select_i32_lt_ssi,
        OpCode::SelectU32Le_Sss => exec::select_u32_le_sss,
        OpCode::SelectU32Le_Ssi => exec::select_u32_le_ssi,
        OpCode::SelectU32Lt_Sss => exec::select_u32_lt_sss,
        OpCode::SelectU32Lt_Ssi => exec::select_u32_lt_ssi,
        OpCode::SelectI64Eq_Sss => exec::select_i64_eq_sss,
        OpCode::SelectI64Eq_Ssi => exec::select_i64_eq_ssi,
        OpCode::SelectI64And_Sss => exec::select_i64_and_sss,
        OpCode::SelectI64And_Ssi => exec::select_i64_and_ssi,
        OpCode::SelectI64Or_Sss => exec::select_i64_or_sss,
        OpCode::SelectI64Or_Ssi => exec::select_i64_or_ssi,
        OpCode::SelectI64Le_Sss => exec::select_i64_le_sss,
        OpCode::SelectI64Le_Ssi => exec::select_i64_le_ssi,
        OpCode::SelectI64Lt_Sss => exec::select_i64_lt_sss,
        OpCode::SelectI64Lt_Ssi => exec::select_i64_lt_ssi,
        OpCode::SelectU64Le_Sss => exec::select_u64_le_sss,
        OpCode::SelectU64Le_Ssi => exec::select_u64_le_ssi,
        OpCode::SelectU64Lt_Sss => exec::select_u64_lt_sss,
        OpCode::SelectU64Lt_Ssi => exec::select_u64_lt_ssi,
        OpCode::SelectF32Eq_Sss => exec::select_f32_eq_sss,
        OpCode::SelectF32Eq_Ssi => exec::select_f32_eq_ssi,
        OpCode::SelectF32Le_Sss => exec::select_f32_le_sss,
        OpCode::SelectF32Le_Ssi => exec::select_f32_le_ssi,
        OpCode::SelectF32Le_Sis => exec::select_f32_le_sis,
        OpCode::SelectF32Lt_Sss => exec::select_f32_lt_sss,
        OpCode::SelectF32Lt_Ssi => exec::select_f32_lt_ssi,
        OpCode::SelectF32Lt_Sis => exec::select_f32_lt_sis,
        OpCode::SelectF64Eq_Sss => exec::select_f64_eq_sss,
        OpCode::SelectF64Eq_Ssi => exec::select_f64_eq_ssi,
        OpCode::SelectF64Le_Sss => exec::select_f64_le_sss,
        OpCode::SelectF64Le_Ssi => exec::select_f64_le_ssi,
        OpCode::SelectF64Le_Sis => exec::select_f64_le_sis,
        OpCode::SelectF64Lt_Sss => exec::select_f64_lt_sss,
        OpCode::SelectF64Lt_Ssi => exec::select_f64_lt_ssi,
        OpCode::SelectF64Lt_Sis => exec::select_f64_lt_sis,
        // load
        OpCode::Load32_Ss => exec::load32_ss,
        OpCode::Load64_Ss => exec::load64_ss,
        OpCode::I32Load8_Ss => exec::i32_load8_ss,
        OpCode::U32Load8_Ss => exec::u32_load8_ss,
        OpCode::I32Load16_Ss => exec::i32_load16_ss,
        OpCode::U32Load16_Ss => exec::u32_load16_ss,
        OpCode::I64Load8_Ss => exec::i64_load8_ss,
        OpCode::U64Load8_Ss => exec::u64_load8_ss,
        OpCode::I64Load16_Ss => exec::i64_load16_ss,
        OpCode::U64Load16_Ss => exec::u64_load16_ss,
        OpCode::I64Load32_Ss => exec::i64_load32_ss,
        OpCode::U64Load32_Ss => exec::u64_load32_ss,
        OpCode::Load32_Si => exec::load32_si,
        OpCode::Load64_Si => exec::load64_si,
        OpCode::I32Load8_Si => exec::i32_load8_si,
        OpCode::U32Load8_Si => exec::u32_load8_si,
        OpCode::I32Load16_Si => exec::i32_load16_si,
        OpCode::U32Load16_Si => exec::u32_load16_si,
        OpCode::I64Load8_Si => exec::i64_load8_si,
        OpCode::U64Load8_Si => exec::u64_load8_si,
        OpCode::I64Load16_Si => exec::i64_load16_si,
        OpCode::U64Load16_Si => exec::u64_load16_si,
        OpCode::I64Load32_Si => exec::i64_load32_si,
        OpCode::U64Load32_Si => exec::u64_load32_si,
        OpCode::Load32Mem0Offset16_Ss => exec::load32_mem0_offset16_ss,
        OpCode::Load64Mem0Offset16_Ss => exec::load64_mem0_offset16_ss,
        OpCode::I32Load8Mem0Offset16_Ss => exec::i32_load8_mem0_offset16_ss,
        OpCode::U32Load8Mem0Offset16_Ss => exec::u32_load8_mem0_offset16_ss,
        OpCode::I32Load16Mem0Offset16_Ss => exec::i32_load16_mem0_offset16_ss,
        OpCode::U32Load16Mem0Offset16_Ss => exec::u32_load16_mem0_offset16_ss,
        OpCode::I64Load8Mem0Offset16_Ss => exec::i64_load8_mem0_offset16_ss,
        OpCode::U64Load8Mem0Offset16_Ss => exec::u64_load8_mem0_offset16_ss,
        OpCode::I64Load16Mem0Offset16_Ss => exec::i64_load16_mem0_offset16_ss,
        OpCode::U64Load16Mem0Offset16_Ss => exec::u64_load16_mem0_offset16_ss,
        OpCode::I64Load32Mem0Offset16_Ss => exec::i64_load32_mem0_offset16_ss,
        OpCode::U64Load32Mem0Offset16_Ss => exec::u64_load32_mem0_offset16_ss,
        // store
        OpCode::Store32_Ss => exec::store32_ss,
        OpCode::Store32_Si => exec::store32_si,
        OpCode::Store64_Ss => exec::store64_ss,
        OpCode::Store64_Si => exec::store64_si,
        OpCode::I32Store8_Ss => exec::i32_store8_ss,
        OpCode::I32Store8_Si => exec::i32_store8_si,
        OpCode::I32Store16_Ss => exec::i32_store16_ss,
        OpCode::I32Store16_Si => exec::i32_store16_si,
        OpCode::I64Store8_Ss => exec::i64_store8_ss,
        OpCode::I64Store8_Si => exec::i64_store8_si,
        OpCode::I64Store16_Ss => exec::i64_store16_ss,
        OpCode::I64Store16_Si => exec::i64_store16_si,
        OpCode::I64Store32_Ss => exec::i64_store32_ss,
        OpCode::I64Store32_Si => exec::i64_store32_si,
        OpCode::Store32_Is => exec::store32_is,
        OpCode::Store32_Ii => exec::store32_ii,
        OpCode::Store64_Is => exec::store64_is,
        OpCode::Store64_Ii => exec::store64_ii,
        OpCode::I32Store8_Is => exec::i32_store8_is,
        OpCode::I32Store8_Ii => exec::i32_store8_ii,
        OpCode::I32Store16_Is => exec::i32_store16_is,
        OpCode::I32Store16_Ii => exec::i32_store16_ii,
        OpCode::I64Store8_Is => exec::i64_store8_is,
        OpCode::I64Store8_Ii => exec::i64_store8_ii,
        OpCode::I64Store16_Is => exec::i64_store16_is,
        OpCode::I64Store16_Ii => exec::i64_store16_ii,
        OpCode::I64Store32_Is => exec::i64_store32_is,
        OpCode::I64Store32_Ii => exec::i64_store32_ii,
        OpCode::Store32Mem0Offset16_Ss => exec::store32_mem0_offset16_ss,
        OpCode::Store32Mem0Offset16_Si => exec::store32_mem0_offset16_si,
        OpCode::Store64Mem0Offset16_Ss => exec::store64_mem0_offset16_ss,
        OpCode::Store64Mem0Offset16_Si => exec::store64_mem0_offset16_si,
        OpCode::I32Store8Mem0Offset16_Ss => exec::i32_store8_mem0_offset16_ss,
        OpCode::I32Store8Mem0Offset16_Si => exec::i32_store8_mem0_offset16_si,
        OpCode::I32Store16Mem0Offset16_Ss => exec::i32_store16_mem0_offset16_ss,
        OpCode::I32Store16Mem0Offset16_Si => exec::i32_store16_mem0_offset16_si,
        OpCode::I64Store8Mem0Offset16_Ss => exec::i64_store8_mem0_offset16_ss,
        OpCode::I64Store8Mem0Offset16_Si => exec::i64_store8_mem0_offset16_si,
        OpCode::I64Store16Mem0Offset16_Ss => exec::i64_store16_mem0_offset16_ss,
        OpCode::I64Store16Mem0Offset16_Si => exec::i64_store16_mem0_offset16_si,
        OpCode::I64Store32Mem0Offset16_Ss => exec::i64_store32_mem0_offset16_ss,
        OpCode::I64Store32Mem0Offset16_Si => exec::i64_store32_mem0_offset16_si,
        #[cfg(feature = "simd")]
        unsupported => unsafe {
            crate::engine::utils::unreachable_unchecked!("unsupported op-code: {unsupported:?}")
        },
    }
}
