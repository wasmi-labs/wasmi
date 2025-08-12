use crate::{
    oracle::{DifferentialOracle, DifferentialOracleMeta},
    FuzzError,
    FuzzSmithConfig,
    FuzzVal,
    FuzzValType,
};
use wasmi::{
    Config,
    Engine,
    Instance,
    Linker,
    Module,
    Store,
    StoreLimits,
    StoreLimitsBuilder,
    Val,
    ValType,
};

use super::ModuleExports;

/// Differential fuzzing backend for the register-machine Wasmi.
#[derive(Debug)]
pub struct WasmiOracle {
    store: Store<StoreLimits>,
    instance: Instance,
    params: Vec<Val>,
    results: Vec<Val>,
}

impl WasmiOracle {
    /// Returns the Wasm module export names.
    pub fn exports(&self) -> ModuleExports {
        let mut exports = ModuleExports::default();
        for export in self.instance.exports(&self.store) {
            let name = export.name();
            match export.ty(&self.store) {
                wasmi::ExternType::Func(ty) => exports.push_func(name, ty),
                wasmi::ExternType::Global(_) => exports.push_global(name),
                wasmi::ExternType::Memory(_) => exports.push_memory(name),
                wasmi::ExternType::Table(_) => exports.push_table(name),
            };
        }
        exports
    }
}

impl DifferentialOracleMeta for WasmiOracle {
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
        config.set_min_stack_height(1024); // 1 kiB
        config.set_min_stack_height(1024 * 1024 * 10); // 10 MiB
        config.set_max_recursion_depth(1024);
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
        let Ok(instance) = linker.instantiate_and_start(&mut store, &module) else {
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

impl DifferentialOracle for WasmiOracle {
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

impl From<FuzzValType> for ValType {
    fn from(ty: FuzzValType) -> Self {
        match ty {
            FuzzValType::I32 => Self::I32,
            FuzzValType::I64 => Self::I64,
            FuzzValType::F32 => Self::F32,
            FuzzValType::F64 => Self::F64,
            FuzzValType::V128 => Self::V128,
            FuzzValType::FuncRef => Self::FuncRef,
            FuzzValType::ExternRef => Self::ExternRef,
        }
    }
}

impl From<Val> for FuzzVal {
    fn from(value: Val) -> Self {
        match value {
            Val::I32(value) => Self::I32(value),
            Val::I64(value) => Self::I64(value),
            Val::F32(value) => Self::F32(value.into()),
            Val::F64(value) => Self::F64(value.into()),
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

impl From<wasmi::Error> for FuzzError {
    fn from(error: wasmi::Error) -> Self {
        use wasmi::TrapCode;
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
