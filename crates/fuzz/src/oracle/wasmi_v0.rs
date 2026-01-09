use crate::{
    oracle::{DifferentialOracle, DifferentialOracleMeta},
    FuzzError,
    FuzzSmithConfig,
    FuzzVal,
};
use wasmi_v0::{
    Config,
    Engine,
    Error,
    ExternRef,
    FuncRef,
    Instance,
    Linker,
    Module,
    Store,
    StoreLimits,
    StoreLimitsBuilder,
    Value,
};

/// Differential fuzzing backend for the stack-machine Wasmi.
#[derive(Debug)]
pub struct WasmiV0Oracle {
    store: Store<StoreLimits>,
    instance: Instance,
    params: Vec<Value>,
    results: Vec<Value>,
}

impl DifferentialOracleMeta for WasmiV0Oracle {
    fn configure(config: &mut FuzzSmithConfig) {
        config.disable_multi_memory();
        config.disable_custom_page_sizes();
        config.disable_memory64();
        config.disable_wide_arithmetic();
        config.disable_simd();
        config.disable_relaxed_simd();
    }

    fn setup(wasm: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        let mut config = Config::default();
        config.wasm_tail_call(true);
        config.wasm_extended_const(true);
        let engine = Engine::new(&config);
        let linker = Linker::new(&engine);
        let limiter = StoreLimitsBuilder::new()
            .memory_size(10_000_000)
            .table_elements(10_000_000)
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

impl DifferentialOracle for WasmiV0Oracle {
    fn name(&self) -> &'static str {
        "Wasmi v0.31"
    }

    fn call(&mut self, name: &str, params: &[FuzzVal]) -> Result<Box<[FuzzVal]>, FuzzError> {
        let Some(func) = self.instance.get_func(&self.store, name) else {
            panic!(
                "{}: could not find exported function: \"{name}\"",
                self.name()
            )
        };
        let ty = func.ty(&self.store);
        self.params.clear();
        self.results.clear();
        self.params.extend(params.iter().cloned().map(Value::from));
        self.results
            .extend(ty.results().iter().copied().map(Value::default));
        func.call(&mut self.store, &self.params[..], &mut self.results[..])?;
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

impl From<Value> for FuzzVal {
    fn from(value: Value) -> Self {
        match value {
            Value::I32(value) => Self::I32(value),
            Value::I64(value) => Self::I64(value),
            Value::F32(value) => Self::F32(value.into()),
            Value::F64(value) => Self::F64(value.into()),
            Value::FuncRef(value) => Self::FuncRef {
                is_null: value.is_null(),
            },
            Value::ExternRef(value) => Self::ExternRef {
                is_null: value.is_null(),
            },
        }
    }
}

impl From<FuzzVal> for Value {
    fn from(value: FuzzVal) -> Self {
        match value {
            FuzzVal::I32(value) => Self::I32(value),
            FuzzVal::I64(value) => Self::I64(value),
            FuzzVal::F32(value) => Self::F32(value.into()),
            FuzzVal::F64(value) => Self::F64(value.into()),
            FuzzVal::V128(_value) => {
                unimplemented!("Wasmi (stack) does not support Wasm `simd` proposal")
            }
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

impl From<Error> for FuzzError {
    fn from(error: Error) -> Self {
        use wasmi_v0::core::TrapCode;
        let Error::Trap(trap) = error else {
            return FuzzError::Other;
        };
        let Some(trap_code) = trap.trap_code() else {
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
