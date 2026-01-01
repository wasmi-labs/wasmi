use crate::{
    engine::{
        executor::handler::{
            dispatch::{execute_until_done, ExecutionOutcome},
            state::{Inst, Ip, Sp, Stack, VmState},
            utils::{self, resolve_instance},
        },
        CodeMap,
        EngineFunc,
        LoadFromCells,
        StoreToCells,
    },
    func::HostFuncEntity,
    ir::{BoundedSlotSpan, Slot, SlotSpan},
    store::{CallHooks, StoreError},
    CallHook,
    Error,
    Instance,
    Store,
    engine::{
        CallParams,
        CallResults,
        CodeMap,
        EngineFunc,
        executor::handler::{
            dispatch::{ExecutionOutcome, execute_until_done},
            state::{Inst, Ip, Sp, Stack, VmState},
            utils::{self, resolve_instance, set_value},
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
    pub fn write_params<Params>(self, params: &Params) -> WasmFuncCall<'a, T, state::Init>
    where
        Params: StoreToCells + ?Sized,
    {
        let mut sp = self.callee_sp;
        let Ok(_) = params.store_to_cells(&mut sp) else {
            panic!("TODO")
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
        )
    }
}

impl<'a, T> WasmFuncCall<'a, T, state::Resumed> {
    pub fn provide_host_results<Params>(
        self,
        params: &Params,
        slots: SlotSpan,
    ) -> WasmFuncCall<'a, T, state::Init>
    where
        Params: StoreToCells + ?Sized,
    {
        let mut sp = self.callee_sp.offset(slots.head());
        let Ok(_) = params.store_to_cells(&mut sp) else {
            panic!("TODO")
        };
        self.new_state(PhantomData)
    }
}

impl<'a, T> WasmFuncCall<'a, T, state::Done> {
    pub fn write_results<Results>(self, results: &mut Results)
    where
        Results: LoadFromCells + ?Sized,
    {
        let mut sp = self.state.sp;
        let Ok(_) = results.load_from_cells(&mut sp) else {
            panic!("TODO")
        };
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
    pub fn write_params<Params>(self, params: &Params) -> HostFuncCall<'a, T, state::InitHost<'a>>
    where
        Params: StoreToCells + ?Sized,
    {
        let state::UninitHost {
            sp,
            inout,
            trampoline,
        } = self.state;
        let mut sp = sp;
        let Ok(_) = params.store_to_cells(&mut sp) else {
            panic!("TODO")
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
    pub fn write_results<Results>(self, results: &mut Results)
    where
        Results: LoadFromCells + ?Sized,
    {
        let mut sp = self.state.sp;
        let Ok(_) = results.load_from_cells(&mut sp) else {
            panic!("TODO")
        };
    }
}
