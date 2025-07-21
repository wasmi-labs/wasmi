use crate::{
    oracle::{DifferentialOracle, DifferentialOracleMeta},
    FuzzError,
    FuzzVal,
};
use wasmtime::{Config, Engine, Instance, Linker, Module, Store, StoreLimitsBuilder, Val, V128};

/// Differential fuzzing backend for Wasmtime.
pub struct WasmtimeOracle {
    store: Store<wasmtime::StoreLimits>,
    instance: Instance,
    params: Vec<Val>,
    results: Vec<Val>,
}

impl DifferentialOracleMeta for WasmtimeOracle {
    fn configure(_config: &mut crate::FuzzSmithConfig) {}

    fn setup(wasm: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        let mut config = Config::default();
        // We disabled backtraces since they sometimes become so large
        // that the entire output is obliterated by them. Generally we are
        // more interested what kind of error occurred and now how an error
        // occurred.
        config.wasm_backtrace(false);
        // We're disabling POSIX signals on errors in the engine because
        // some fuzzers will catch them and report them as false positives.
        config.signals_based_traps(false);
        // We tell Cranelift to not optimize the Wasm input since this
        // improves performance of the translation process that can quickly
        // become very slow.
        config.cranelift_opt_level(wasmtime::OptLevel::None);
        // We tell Cranelift to use a somewhat faster register allocation
        // scheme that might yield worse codegen, but this translation time
        // trade-off usually is not worth it during fuzzing.

        // TODO: re-enable fast regalloc once the Wasmtime issue has been resolved:
        //       https://github.com/bytecodealliance/wasmtime/pull/10554
        // config.cranelift_regalloc_algorithm(wasmtime::RegallocAlgorithm::SinglePass);

        config.wasm_custom_page_sizes(true);
        config.wasm_wide_arithmetic(true);
        config.relaxed_simd_deterministic(true);
        let engine = Engine::new(&config).unwrap();
        let linker = Linker::new(&engine);
        let limiter = StoreLimitsBuilder::new()
            .memory_size(1000 * 0x10000)
            .build();
        let mut store = Store::new(&engine, limiter);
        store.limiter(|lim| lim);
        let module = Module::new(store.engine(), wasm).unwrap();
        let Ok(instance) = linker.instantiate(&mut store, &module) else {
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

impl DifferentialOracle for WasmtimeOracle {
    fn name(&self) -> &'static str {
        "Wasmtime"
    }

    fn call(&mut self, name: &str, params: &[FuzzVal]) -> Result<Box<[FuzzVal]>, FuzzError> {
        let Some(func) = self.instance.get_func(&mut self.store, name) else {
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
            .extend(ty.results().map(|ty| Val::default_for_ty(&ty).unwrap()));
        func.call(&mut self.store, &self.params[..], &mut self.results[..])?;
        let results = self.results.iter().cloned().map(FuzzVal::from).collect();
        Ok(results)
    }

    fn get_global(&mut self, name: &str) -> Option<FuzzVal> {
        let value = self
            .instance
            .get_global(&mut self.store, name)?
            .get(&mut self.store);
        Some(FuzzVal::from(value))
    }

    fn get_memory(&mut self, name: &str) -> Option<&[u8]> {
        let data = self
            .instance
            .get_memory(&mut self.store, name)?
            .data(&mut self.store);
        Some(data)
    }
}

impl From<Val> for FuzzVal {
    fn from(value: Val) -> Self {
        match value {
            Val::I32(value) => Self::I32(value),
            Val::I64(value) => Self::I64(value),
            Val::F32(value) => Self::F32(f32::from_bits(value)),
            Val::F64(value) => Self::F64(f64::from_bits(value)),
            Val::V128(value) => Self::V128(value.as_u128()),
            Val::FuncRef(value) => Self::FuncRef {
                is_null: value.is_none(),
            },
            Val::ExternRef(value) => Self::ExternRef {
                is_null: value.is_none(),
            },
            val => panic!("Wasmtime: unsupported `Val`: {val:?}"),
        }
    }
}

impl From<FuzzVal> for Val {
    fn from(value: FuzzVal) -> Self {
        match value {
            FuzzVal::I32(value) => Self::I32(value),
            FuzzVal::I64(value) => Self::I64(value),
            FuzzVal::F32(value) => Self::F32(value.to_bits()),
            FuzzVal::F64(value) => Self::F64(value.to_bits()),
            FuzzVal::V128(value) => Self::V128(V128::from(value)),
            FuzzVal::FuncRef { is_null } => {
                assert!(is_null);
                Self::FuncRef(None)
            }
            FuzzVal::ExternRef { is_null } => {
                assert!(is_null);
                Self::ExternRef(None)
            }
        }
    }
}

impl From<wasmtime::Error> for FuzzError {
    fn from(error: wasmtime::Error) -> Self {
        use wasmtime::Trap;
        let Some(trap_code) = error.downcast_ref::<wasmtime::Trap>() else {
            return FuzzError::Other;
        };
        let trap_code = match trap_code {
            Trap::UnreachableCodeReached => crate::TrapCode::UnreachableCodeReached,
            Trap::MemoryOutOfBounds => crate::TrapCode::MemoryOutOfBounds,
            Trap::TableOutOfBounds => crate::TrapCode::TableOutOfBounds,
            Trap::IndirectCallToNull => crate::TrapCode::IndirectCallToNull,
            Trap::IntegerDivisionByZero => crate::TrapCode::IntegerDivisionByZero,
            Trap::IntegerOverflow => crate::TrapCode::IntegerOverflow,
            Trap::BadConversionToInteger => crate::TrapCode::BadConversionToInteger,
            Trap::StackOverflow => crate::TrapCode::StackOverflow,
            Trap::BadSignature => crate::TrapCode::BadSignature,
            _ => return FuzzError::Other,
        };
        FuzzError::Trap(trap_code)
    }
}
