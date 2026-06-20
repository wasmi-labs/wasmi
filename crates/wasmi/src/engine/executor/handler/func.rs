use crate::{
    CallHook,
    Error,
    Instance,
    Store,
    engine::{
        CodeView,
        EngineFunc,
        LiftFromCells,
        LowerToCells,
        code_map::ResultRegs,
        executor::handler::{
            dispatch::{ExecutionOutcome, execute_until_done},
            state::{Freg32, Freg64, Inst, Ip, Ireg, Sp, Stack, VmState},
            utils::{self, resolve_instance},
        },
    },
    func::HostFuncEntity,
    ir::{BoundedSlotSpan, Slot, SlotSpan},
    store::{CallHooks, StoreError},
};
use core::marker::PhantomData;

pub struct WasmFuncCall<'a, T, State> {
    store: &'a mut Store<T>,
    stack: &'a mut Stack,
    code: CodeView<'a>,
    callee_ip: Ip,
    callee_sp: Sp,
    instance: Inst,
    state: State,
    ireg: Ireg,
    freg32: Freg32,
    freg64: Freg64,
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
            ireg: self.ireg,
            freg32: self.freg32,
            freg64: self.freg64,
        }
    }
}

mod state {
    use super::Sp;
    use crate::{engine::InOutParams, func::Trampoline};
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
        pub inout: InOutParams<'a>,
        pub trampoline: Trampoline,
    }

    pub struct InitHost<'a> {
        pub sp: Sp,
        pub inout: InOutParams<'a>,
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
    pub fn write_params<Params>(self, params: Params) -> WasmFuncCall<'a, T, state::Init>
    where
        Params: LowerToCells,
    {
        let mut sp = self.callee_sp;
        let Ok(_) = params.lower_to_cells(&*self.store, &mut sp) else {
            panic!("failed to write parameter values to cells")
        };
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
        let mut state = VmState::new(store, self.stack, self.code);
        execute_until_done(
            &mut state,
            self.callee_ip,
            self.callee_sp,
            mem0,
            mem0_len,
            self.instance,
            self.ireg,
            self.freg32,
            self.freg64,
        )
    }
}

impl<'a, T> WasmFuncCall<'a, T, state::Resumed> {
    pub fn provide_host_results<Params>(
        mut self,
        params: Params,
        slots: SlotSpan,
        result_regs: ResultRegs,
    ) -> WasmFuncCall<'a, T, state::Init>
    where
        Params: LowerToCells,
    {
        let base = self.callee_sp.offset(slots.head());
        let mut sp = base;
        let Ok(_) = params.lower_to_cells(&*self.store, &mut sp) else {
            panic!("failed to store provided host results to cells")
        };
        // Mirror the provided results into accumulator registers so that resuming a `return_call`
        // to a host function delivers them to a caller that expects results in registers. The
        // slots remain valid, so a caller expecting results in slots is unaffected.
        let (ireg, freg32, freg64) = utils::load_result_regs(result_regs, base);
        self.ireg = ireg;
        self.freg32 = freg32;
        self.freg64 = freg64;
        self.new_state(PhantomData)
    }
}

impl<'a, T> WasmFuncCall<'a, T, state::Done> {
    /// Spills results returned in accumulator registers into their result slots.
    ///
    /// # Note
    ///
    /// Required at the host (root call) boundary since the host reads all results from the
    /// result slots and cannot read accumulator registers. The final accumulator register
    /// values were persisted by [`exec_return`](super::utils::exec_return) upon returning from
    /// the root function.
    pub fn spill_result_regs(&mut self, regs: ResultRegs) {
        if regs.is_empty() {
            return;
        }
        let (ireg, freg32, freg64) = self.stack.regs();
        utils::spill_result_regs(regs, self.state.sp, ireg, freg32, freg64);
    }

    pub fn write_results<Results>(self, results: Results) -> Results::Value
    where
        Results: LiftFromCells,
    {
        let mut sp = self.state.sp;
        let Ok(value) = results.lift_from_cells(&*self.store, &mut sp) else {
            panic!("failed to load result values from cells")
        };
        value
    }
}

pub fn init_wasm_func_call<'a, T>(
    store: &'a mut Store<T>,
    code: CodeView<'a>,
    stack: &'a mut Stack,
    func: EngineFunc,
    instance: Instance,
) -> Result<WasmFuncCall<'a, T, state::Uninit>, Error> {
    let Some(compiled_func) = code.get_or_compile(Some(store.inner.fuel_mut()), func)? else {
        panic!("missing function entry at: {func:?}")
    };
    let callee_ip = Ip::from(compiled_func.ops());
    let len_local_slots = compiled_func.len_local_slots();
    let len_stack_slots = compiled_func.len_stack_slots();
    // Note: using a length of 0 for `callee_params` simply has the effect that all frame
    //       cells are initialized to zero which is a safe default. There currently is not
    //       an easy and efficient way to get the number of parameter cells at this point
    //       so we simply default to 0.
    let callee_params = BoundedSlotSpan::new(SlotSpan::new(Slot::from(0)), 0);
    let instance = resolve_instance(store.prune(), &instance).into();
    let callee_sp = stack.push_frame(
        None,
        callee_ip,
        callee_params,
        len_local_slots,
        len_stack_slots,
        Some(instance),
        // The root frame's results are spilled by the host call boundary, not on return.
        ResultRegs::default(),
    )?;
    let (ireg, freg32, freg64) = stack.regs();
    Ok(WasmFuncCall {
        store,
        stack,
        code,
        callee_ip,
        callee_sp,
        instance,
        state: PhantomData,
        ireg,
        freg32,
        freg64,
    })
}

pub fn resume_wasm_func_call<'a, T>(
    store: &'a mut Store<T>,
    code: CodeView<'a>,
    stack: &'a mut Stack,
) -> Result<WasmFuncCall<'a, T, state::Resumed>, Error> {
    let (callee_ip, callee_sp, instance, ireg, freg32, freg64) = stack.restore_frame();
    Ok(WasmFuncCall {
        store,
        stack,
        code,
        callee_ip,
        callee_sp,
        instance,
        state: PhantomData,
        ireg,
        freg32,
        freg64,
    })
}

pub fn init_host_func_call<'a, T>(
    store: &'a mut Store<T>,
    stack: &'a mut Stack,
    func: HostFuncEntity,
) -> Result<HostFuncCall<'a, T, state::UninitHost<'a>>, Error> {
    let len_param_cells = func.len_param_cells();
    let len_result_cells = func.len_result_cells();
    let trampoline = *func.trampoline();
    let callee_params = BoundedSlotSpan::new(SlotSpan::new(Slot::from(0)), len_param_cells);
    let (sp, inout) = stack.prepare_host_frame(None, callee_params, len_result_cells)?;
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
    pub fn write_params<Params>(self, params: Params) -> HostFuncCall<'a, T, state::InitHost<'a>>
    where
        Params: LowerToCells,
    {
        let state::UninitHost {
            sp,
            inout,
            trampoline,
        } = self.state;
        let mut sp_writer = sp;
        let Ok(_) = params.lower_to_cells(&*self.store, &mut sp_writer) else {
            panic!("failed to store parameter values to cells")
        };
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
    pub fn write_results<Results>(self, results: Results) -> Results::Value
    where
        Results: LiftFromCells,
    {
        let mut sp = self.state.sp;
        let Ok(value) = results.lift_from_cells(&*self.store, &mut sp) else {
            panic!("failed to load result value from cells")
        };
        value
    }
}
