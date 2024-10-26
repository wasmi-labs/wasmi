use crate::{oracle::DifferentialOracle, FuzzError, FuzzVal};
use wasmtime::{Config, Engine, Instance, Linker, Module, Store, StoreLimitsBuilder, Val, ValType};

/// Differential fuzzing backend for Wasmtime.
struct Wasmtime {
    store: Store<wasmtime::StoreLimits>,
    instance: Instance,
    params: Vec<Val>,
    results: Vec<Val>,
}

impl Wasmtime {
    fn type_to_value(ty: ValType) -> wasmtime::Val {
        match ty {
            ValType::I32 => wasmtime::Val::I32(1),
            ValType::I64 => wasmtime::Val::I64(1),
            ValType::F32 => wasmtime::Val::F32(1.0_f32.to_bits()),
            ValType::F64 => wasmtime::Val::F64(1.0_f64.to_bits()),
            unsupported => panic!(
                "differential fuzzing does not support reference types, yet but found: {unsupported:?}"
            ),
        }
    }
}

impl DifferentialOracle for Wasmtime {
    const NAME: &str = "Wasmtime";

    fn configure(_config: &mut crate::FuzzSmithConfig) {}

    fn setup(wasm: &[u8]) -> Option<Self> {
        let mut config = Config::default();
        // We disabled backtraces since they sometimes become so large
        // that the entire output is obliterated by them. Generally we are
        // more interested what kind of error occurred and now how an error
        // occurred.
        config.wasm_backtrace(false);
        let engine = Engine::default();
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

    fn call(&mut self, name: &str, params: &[FuzzVal]) -> Result<Box<[FuzzVal]>, FuzzError> {
        let Some(func) = self.instance.get_func(&mut self.store, name) else {
            panic!(
                "{}: could not find exported function: \"{name}\"",
                Self::NAME
            )
        };
        let ty = func.ty(&self.store);
        self.params.clear();
        self.results.clear();
        self.params.extend(params.iter().cloned().map(Val::from));
        self.results.extend(ty.results().map(Self::type_to_value));
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
