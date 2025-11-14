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
    func::HostFuncEntity,
    ir,
    ir::{BoundedSlotSpan, OpCode, Slot, SlotSpan},
    store::{CallHooks, StoreError},
    CallHook,
    Error,
    Instance,
    Store,
    TrapCode,
};
use core::{marker::PhantomData, ops::ControlFlow};

#[inline(always)]
pub fn fetch_handler(ip: Ip) -> Handler {
    match cfg!(feature = "indirect-dispatch") {
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
    use crate::func::{FuncInOut, Trampoline};
    use core::marker::PhantomData;

    pub type Uninit = PhantomData<marker::Uninit>;
    pub type Init = PhantomData<marker::Init>;
    pub type Resumed = PhantomData<marker::Resumed>;

    mod marker {
        pub enum Uninit {}
        pub enum Init {}
        pub enum Resumed {}
    }

    pub struct UninitHost<'a> {
        pub sp: Sp,
        pub inout: FuncInOut<'a>,
        pub trampoline: Trampoline,
    }

    pub struct InitHost<'a> {
        pub sp: Sp,
        pub inout: FuncInOut<'a>,
        pub trampoline: Trampoline,
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
    let callee_params = BoundedSlotSpan::new(SlotSpan::new(Slot::from(0)), 0);
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

pub fn init_host_func_call<'a, T>(
    store: &'a mut Store<T>,
    stack: &'a mut Stack,
    func: HostFuncEntity,
) -> Result<HostFuncCall<'a, T, state::UninitHost<'a>>, Error> {
    let len_params = func.len_params();
    let len_results = func.len_results();
    let trampoline = *func.trampoline();
    let callee_params = BoundedSlotSpan::new(SlotSpan::new(Slot::from(0)), len_params);
    let (sp, inout) = stack.prepare_host_frame(None, callee_params, len_results)?;
    Ok(HostFuncCall {
        store,
        state: state::UninitHost {
            sp,
            inout,
            trampoline,
        },
    })
}

#[derive(Debug)]
pub struct HostFuncCall<'a, T, State> {
    store: &'a mut Store<T>,
    state: State,
}

impl<'a, T> HostFuncCall<'a, T, state::UninitHost<'a>> {
    pub fn write_params(self, params: impl CallParams) -> HostFuncCall<'a, T, state::InitHost<'a>> {
        let state::UninitHost {
            sp,
            inout,
            trampoline,
        } = self.state;
        let mut param_slot = Slot::from(0);
        for param_value in params.call_params() {
            set_value(sp, param_slot, param_value);
            param_slot = param_slot.next();
        }
        HostFuncCall {
            store: self.store,
            state: state::InitHost {
                sp,
                inout,
                trampoline,
            },
        }
    }
}

impl<'a, T> HostFuncCall<'a, T, state::InitHost<'a>> {
    pub fn execute(self) -> Result<HostFuncCall<'a, T, state::Done>, Error> {
        let state::InitHost {
            sp,
            inout,
            trampoline,
        } = self.state;
        let outcome = self
            .store
            .prune()
            .call_host_func(trampoline, None, inout, CallHooks::Ignore);
        if let Err(error) = outcome {
            match error {
                StoreError::External(error) => return Err(error),
                StoreError::Internal(error) => panic!("internal interpreter error: {error}"),
            }
        }
        Ok(HostFuncCall {
            store: self.store,
            state: state::Done { sp },
        })
    }
}

impl<'a, T> HostFuncCall<'a, T, state::Done> {
    pub fn write_results<R: CallResults>(self, results: R) -> <R as CallResults>::Results {
        let len_results = results.len_results();
        let sp = self.state.sp;
        let slice = unsafe { sp.as_slice(len_results) };
        results.call_results(slice)
    }
}

#[cfg(feature = "portable-dispatch")]
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

#[cfg(not(feature = "portable-dispatch"))]
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

#[cfg(not(feature = "portable-dispatch"))]
#[derive(Debug)]
pub enum NextState {}

#[cfg(feature = "portable-dispatch")]
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
pub type Done = Control<NextState, Break>;

#[cfg(feature = "portable-dispatch")]
pub trait ControlContinue: Sized {
    fn control_continue(ip: Ip, sp: Sp, mem0: Mem0Ptr, mem0_len: Mem0Len, instance: Inst) -> Self;
}

#[cfg(feature = "portable-dispatch")]
impl ControlContinue for Done {
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

impl<T> ControlBreak for Control<T, Break> {
    fn control_break() -> Self {
        Self::Break(Break::WithReason)
    }
}

type Handler =
    fn(&mut VmState, ip: Ip, sp: Sp, mem0: Mem0Ptr, mem0_len: Mem0Len, instance: Inst) -> Done;

#[cfg(not(feature = "portable-dispatch"))]
macro_rules! dispatch {
    ($state:expr, $ip:expr, $sp:expr, $mem0:expr, $mem0_len:expr, $instance:expr) => {{
        let handler = $crate::engine::executor::handler::dispatch::fetch_handler($ip);
        return handler($state, $ip, $sp, $mem0, $mem0_len, $instance);
    }};
}

#[cfg(feature = "portable-dispatch")]
macro_rules! dispatch {
    ($state:expr, $ip:expr, $sp:expr, $mem0:expr, $mem0_len:expr, $instance:expr) => {{
        let _: &mut VmState = $state;
        return <$crate::engine::executor::handler::dispatch::Done as $crate::engine::executor::handler::ControlContinue>::control_continue(
            $ip, $sp, $mem0, $mem0_len, $instance,
        );
    }};
}

macro_rules! expand_op_code_to_handler {
    ( $( $snake_case:ident => $camel_case:ident ),* $(,)? ) => {
        #[inline(always)]
        pub fn op_code_to_handler(code: OpCode) -> Handler {
            match code {
                $( OpCode::$camel_case => exec::$snake_case, )*
            }
        }
    };
}
ir::for_each_op!(expand_op_code_to_handler);
