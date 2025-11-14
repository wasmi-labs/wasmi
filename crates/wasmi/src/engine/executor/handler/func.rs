use crate::{
    engine::{
        executor::handler::{
            dispatch::{execute_until_done, ExecutionOutcome},
            state::{Inst, Ip, Sp, Stack, VmState},
            utils::{self, resolve_instance, set_value},
        },
        CallParams,
        CallResults,
        CodeMap,
        EngineFunc,
    },
    func::HostFuncEntity,
    ir::{BoundedSlotSpan, Slot, SlotSpan},
    store::{CallHooks, StoreError},
    CallHook,
    Error,
    Instance,
    Store,
};
use core::marker::PhantomData;

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
