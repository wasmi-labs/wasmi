use crate::{
    oracle::{DifferentialOracle, DifferentialOracleMeta},
    FuzzError,
    FuzzSmithConfig,
    FuzzVal,
};
use wasmi_v048::{
    core::{TrapCode, V128},
    Config,
    Engine,
    Error,
    ExternRef,
    FuncRef,
    Instance,
    Linker,
    Module,
    StackLimits,
    Store,
    StoreLimits,
    StoreLimitsBuilder,
    Val,
};

/// Differential fuzzing backend for the register-machine Wasmi v0.48.0.
#[derive(Debug)]
pub struct WasmiV048Oracle {
    store: Store<StoreLimits>,
    instance: Instance,
    params: Vec<Val>,
    results: Vec<Val>,
}

impl DifferentialOracleMeta for WasmiV048Oracle {
    fn configure(_config: &mut FuzzSmithConfig) {}

    fn setup(wasm: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        let mut config = Config::default();
        // We set custom limits since Wasmi (register) might use more
        // stack space than Wasmi (stack) for some malicious recursive workloads.
        // Wasmtime technically suffers from the same problems (register machine)
        // but can offset them due to its superior optimizations.
        //
        // We increase the maximum stack space for Wasmi (register) to avoid
        // common stack overflows in certain generated fuzz test cases this way.
        config.set_stack_limits(
            StackLimits::new(
                1024,             // 1 kiB
                1024 * 1024 * 10, // 10 MiB
                1024,
            )
            .unwrap(),
        );
        config.wasm_custom_page_sizes(true);
        config.wasm_wide_arithmetic(true);
        let engine = Engine::new(&config);
        let linker = Linker::new(&engine);
        let limiter = StoreLimitsBuilder::new()
            .memory_size(1000 * 0x10000)
            .build();
        let mut store = Store::new(&engine, limiter);
        store.limiter(|lim| lim);
        let module = Module::new(store.engine(), wasm).unwrap();
        let Ok(unstarted_instance) = linker.instantiate(&mut store, &module) else {
            return None;
        };
        let Ok(instance) = unstarted_instance.ensure_no_start(&mut store) else {
            return None;
        };
        Some(Self {
            store,
            instance,
            params: Vec::new(),
            results: Vec::new(),
        })
    }
}

impl DifferentialOracle for WasmiV048Oracle {
    fn name(&self) -> &'static str {
        "Wasmi"
    }

    fn call(&mut self, name: &str, params: &[FuzzVal]) -> Result<Box<[FuzzVal]>, FuzzError> {
        let Some(func) = self.instance.get_func(&self.store, name) else {
            panic!(
                "{}: could not find exported function: \"{name}\"",
                self.name(),
            )
        };
        let ty = func.ty(&self.store);
        self.params.clear();
        self.results.clear();
        self.params.extend(params.iter().cloned().map(Val::from));
        self.results
            .extend(ty.results().iter().copied().map(Val::default));
        func.call(&mut self.store, &self.params[..], &mut self.results[..])
            .map_err(FuzzError::from)?;
        let results = self.results.iter().cloned().map(FuzzVal::from).collect();
        Ok(results)
    }

    fn get_global(&mut self, name: &str) -> Option<FuzzVal> {
        let value = self
            .instance
            .get_global(&self.store, name)?
            .get(&self.store);
        Some(FuzzVal::from(value))
    }

    fn get_memory(&mut self, name: &str) -> Option<&[u8]> {
        let data = self
            .instance
            .get_memory(&self.store, name)?
            .data(&self.store);
        Some(data)
    }
}

impl From<FuzzVal> for Val {
    fn from(value: FuzzVal) -> Self {
        match value {
            FuzzVal::I32(value) => Self::I32(value),
            FuzzVal::I64(value) => Self::I64(value),
            FuzzVal::F32(value) => Self::F32(value.into()),
            FuzzVal::F64(value) => Self::F64(value.into()),
            FuzzVal::V128(value) => Self::V128(V128::from(value)),
            FuzzVal::FuncRef { is_null } => {
                assert!(is_null);
                Self::FuncRef(FuncRef::null())
            }
            FuzzVal::ExternRef { is_null } => {
                assert!(is_null);
                Self::ExternRef(ExternRef::null())
            }
        }
    }
}

impl From<Val> for FuzzVal {
    fn from(value: Val) -> Self {
        match value {
            Val::I32(value) => Self::I32(value),
            Val::I64(value) => Self::I64(value),
            Val::F32(value) => Self::F32(value.to_float()),
            Val::F64(value) => Self::F64(value.to_float()),
            Val::V128(value) => Self::V128(value.as_u128()),
            Val::FuncRef(value) => Self::FuncRef {
                is_null: value.is_null(),
            },
            Val::ExternRef(value) => Self::ExternRef {
                is_null: value.is_null(),
            },
        }
    }
}

impl From<Error> for FuzzError {
    fn from(error: Error) -> Self {
        let Some(trap_code) = error.as_trap_code() else {
            return FuzzError::Other;
        };
        let trap_code = match trap_code {
            TrapCode::UnreachableCodeReached => crate::TrapCode::UnreachableCodeReached,
            TrapCode::MemoryOutOfBounds => crate::TrapCode::MemoryOutOfBounds,
            TrapCode::TableOutOfBounds => crate::TrapCode::TableOutOfBounds,
            TrapCode::IndirectCallToNull => crate::TrapCode::IndirectCallToNull,
            TrapCode::IntegerDivisionByZero => crate::TrapCode::IntegerDivisionByZero,
            TrapCode::IntegerOverflow => crate::TrapCode::IntegerOverflow,
            TrapCode::BadConversionToInteger => crate::TrapCode::BadConversionToInteger,
            TrapCode::StackOverflow => crate::TrapCode::StackOverflow,
            TrapCode::BadSignature => crate::TrapCode::BadSignature,
            TrapCode::OutOfFuel | TrapCode::GrowthOperationLimited => return FuzzError::Other,
        };
        FuzzError::Trap(trap_code)
    }
}
