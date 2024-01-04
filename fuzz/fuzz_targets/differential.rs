#![no_main]

mod utils;

use libfuzzer_sys::fuzz_target;
use utils::{ty_to_val, ExecConfig};
use wasm_smith::ConfiguredModule;
use wasmi as wasmi_reg;
use wasmi_reg::core::{F32, F64};

/// Trait implemented by differential fuzzing backends.
trait DifferentialTarget: Sized {
    /// The value type of the backend.
    type Value;
    /// The error type of the backend.
    type Error;

    /// Sets up the store and exported functions for the backend if possible.
    fn setup(wasm: &[u8]) -> Option<Self>;

    /// Calls the exported function with `name` and returns the result.
    fn call(&mut self, name: &str) -> Result<&[Self::Value], Self::Error>;
}

/// Differential fuzzing backend for the register-machine Wasmi.
struct WasmiRegister {
    store: wasmi_reg::Store<wasmi_reg::StoreLimits>,
    instance: wasmi_reg::Instance,
    params: Vec<wasmi_reg::Value>,
    results: Vec<wasmi_reg::Value>,
}

impl WasmiRegister {
    /// Returns the names of all exported functions.
    pub fn exported_funcs(&self) -> Vec<Box<str>> {
        self.instance
            .exports(&self.store)
            .filter(|e| matches!(e.ty(&self.store), wasmi_reg::ExternType::Func(_)))
            .map(|e| e.name().into())
            .collect()
    }

    fn type_to_value(ty: &wasmi_reg::core::ValueType) -> wasmi_reg::Value {
        ty_to_val(ty)
    }
}

impl DifferentialTarget for WasmiRegister {
    type Value = wasmi_reg::Value;
    type Error = wasmi_reg::Error;

    fn call(&mut self, name: &str) -> Result<&[Self::Value], Self::Error> {
        let Some(func) = self.instance.get_func(&self.store, name) else {
            panic!(
                "wasmi (register) is missing exported function {name} that exists in wasmi (register)"
            )
        };
        let ty = func.ty(&self.store);
        self.params.clear();
        self.results.clear();
        self.params
            .extend(ty.params().iter().map(Self::type_to_value));
        self.results
            .extend(ty.results().iter().map(Self::type_to_value));
        func.call(&mut self.store, &self.params[..], &mut self.results[..])?;
        Ok(&self.results[..])
    }

    fn setup(wasm: &[u8]) -> Option<Self> {
        use wasmi_reg::{Engine, Linker, Module, Store, StoreLimitsBuilder};
        let engine = Engine::default();
        let linker = Linker::new(&engine);
        let limiter = StoreLimitsBuilder::new()
            .memory_size(1000 * 0x10000)
            .build();
        let mut store = Store::new(&engine, limiter);
        store.limiter(|lim| lim);
        let module = Module::new(store.engine(), wasm).unwrap();
        let Ok(preinstance) = linker.instantiate(&mut store, &module) else {
            return None;
        };
        let Ok(instance) = preinstance.ensure_no_start(&mut store) else {
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

/// Differential fuzzing backend for the stack-machine Wasmi.
struct WasmiStack {
    store: wasmi_stack::Store<wasmi_stack::StoreLimits>,
    instance: wasmi_stack::Instance,
    params: Vec<wasmi_stack::Value>,
    results: Vec<wasmi_stack::Value>,
}

impl WasmiStack {
    fn type_to_value(ty: &wasmi_stack::core::ValueType) -> wasmi_stack::Value {
        use wasmi_stack::core::ValueType;
        match ty {
            ValueType::I32 => wasmi_stack::Value::I32(1),
            ValueType::I64 => wasmi_stack::Value::I64(1),
            ValueType::F32 => wasmi_stack::Value::F32(1.0.into()),
            ValueType::F64 => wasmi_stack::Value::F64(1.0.into()),
            unsupported => panic!(
                "execution fuzzing does not support reference types, yet but found: {unsupported:?}"
            ),
        }
    }
}

impl DifferentialTarget for WasmiStack {
    type Value = wasmi_stack::Value;
    type Error = wasmi_stack::Error;

    fn call(&mut self, name: &str) -> Result<&[Self::Value], Self::Error> {
        let Some(func) = self.instance.get_func(&self.store, name) else {
            panic!(
                "wasmi (stack) is missing exported function {name} that exists in wasmi (register)"
            )
        };
        let ty = func.ty(&self.store);
        self.params.clear();
        self.results.clear();
        self.params
            .extend(ty.params().iter().map(Self::type_to_value));
        self.results
            .extend(ty.results().iter().map(Self::type_to_value));
        func.call(&mut self.store, &self.params[..], &mut self.results[..])?;
        Ok(&self.results[..])
    }

    fn setup(wasm: &[u8]) -> Option<Self> {
        use wasmi_stack::{Engine, Linker, Module, Store, StoreLimitsBuilder};
        let engine = Engine::default();
        let linker = Linker::new(&engine);
        let limiter = StoreLimitsBuilder::new()
            .memory_size(1000 * 0x10000)
            .build();
        let mut store = Store::new(&engine, limiter);
        store.limiter(|lim| lim);
        let module = Module::new(store.engine(), wasm).unwrap();
        let Ok(preinstance) = linker.instantiate(&mut store, &module) else {
            return None;
        };
        let Ok(instance) = preinstance.ensure_no_start(&mut store) else {
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

/// Differential fuzzing backend for Wasmtime.
struct Wasmtime {
    store: wasmtime::Store<wasmtime::StoreLimits>,
    instance: wasmtime::Instance,
    params: Vec<wasmtime::Val>,
    results: Vec<wasmtime::Val>,
}

impl Wasmtime {
    fn type_to_value(ty: wasmtime::ValType) -> wasmtime::Val {
        match ty {
            wasmtime::ValType::I32 => wasmtime::Val::I32(1),
            wasmtime::ValType::I64 => wasmtime::Val::I64(1),
            wasmtime::ValType::F32 => wasmtime::Val::F32(1.0_f32.to_bits()),
            wasmtime::ValType::F64 => wasmtime::Val::F64(1.0_f64.to_bits()),
            unsupported => panic!(
                "execution fuzzing does not support reference types, yet but found: {unsupported:?}"
            ),
        }
    }
}

impl DifferentialTarget for Wasmtime {
    type Value = wasmtime::Val;
    type Error = wasmtime::Error;

    fn call(&mut self, name: &str) -> Result<&[Self::Value], Self::Error> {
        let Some(func) = self.instance.get_func(&mut self.store, name) else {
            panic!("wasmtime is missing exported function {name} that exists in wasmi (register)")
        };
        let ty = func.ty(&self.store);
        self.params.clear();
        self.results.clear();
        self.params.extend(ty.params().map(Self::type_to_value));
        self.results.extend(ty.results().map(Self::type_to_value));
        func.call(&mut self.store, &self.params[..], &mut self.results[..])?;
        Ok(&self.results[..])
    }

    fn setup(wasm: &[u8]) -> Option<Self> {
        use wasmtime::{Engine, Linker, Module, Store, StoreLimitsBuilder};
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
}

fuzz_target!(|cfg_module: ConfiguredModule<ExecConfig>| {
    let mut smith_module = cfg_module.module;
    // Note: We cannot use built-in fuel metering of the different engines since that
    //       would introduce unwanted non-determinism with respect to fuzz testing.
    smith_module.ensure_termination(1000 /* fuel */);
    let wasm = smith_module.to_bytes();
    let Some(mut context_reg) = <WasmiRegister as DifferentialTarget>::setup(&wasm[..]) else {
        return;
    };
    let Some(mut context_stack) = <WasmiStack as DifferentialTarget>::setup(&wasm[..]) else {
        panic!("wasmi (register) succeeded to create Context while wasmi (stack) failed");
    };
    let exported_funcs = context_reg.exported_funcs();
    for name in exported_funcs {
        let result_reg = context_reg.call(&name);
        let result_stack = context_stack.call(&name);
        if let (Err(error_reg), Err(error_stack)) = (&result_reg, &result_stack) {
            let errstr_reg = error_reg.to_string();
            let errstr_stack = error_stack.to_string();
            if errstr_reg != errstr_stack {
                panic!(
                    "wasmi (register) and wasmi (stack) fail with different error codes\n\
                    |    wasmi (register): {errstr_reg}\n\
                    |    wasmi (stack)   : {errstr_stack}",
                )
            }
        }
        if let (Ok(results_reg), Ok(results_stack)) = (&result_reg, &result_stack) {
            let results_reg = results_reg
                .iter()
                .map(FuzzValue::from)
                .collect::<Vec<FuzzValue>>();
            let results_stack = results_stack
                .iter()
                .map(FuzzValue::from)
                .collect::<Vec<FuzzValue>>();
            if results_reg != results_stack {
                let Some(mut context_wasmtime) = <Wasmtime as DifferentialTarget>::setup(&wasm[..])
                else {
                    panic!("failed to setup Wasmtime fuzzing");
                };
                let Ok(results_wasmtime) = context_wasmtime.call(&name) else {
                    panic!("failed to execute function {name} via Wasmtime")
                };
                let results_wasmtime = results_wasmtime
                    .iter()
                    .map(FuzzValue::from)
                    .collect::<Vec<FuzzValue>>();
                let text = match (
                    results_wasmtime == results_reg,
                    results_wasmtime == results_stack,
                ) {
                    (true, false) => "Wasmi (stack) disagrees with Wasmi (register) and Wasmtime",
                    (false, true) => "Wasmi (register) disagrees with Wasmi (stack) and Wasmtime",
                    (false, false) => "Wasmi (register), Wasmi (stack) and Wasmtime disagree",
                    (true, true) => unreachable!("results_reg and results_stack differ"),
                };
                println!(
                    "{text} for function execution: {name}\n\
                    |    Wasmi (register): {results_reg:?}\n\
                    |    Wasmi (stack)   : {results_stack:?}\n\
                    |    Wasmtime        : {results_wasmtime:?}"
                );
                if results_wasmtime != results_reg {
                    panic!()
                }
            }
        }
    }
});

#[derive(Debug, Copy, Clone)]
pub enum FuzzValue {
    I32(i32),
    I64(i64),
    F32(F32),
    F64(F64),
}

impl PartialEq for FuzzValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::I32(lhs), Self::I32(rhs)) => lhs == rhs,
            (Self::I64(lhs), Self::I64(rhs)) => lhs == rhs,
            (Self::F32(lhs), Self::F32(rhs)) => {
                if lhs.is_nan() && rhs.is_nan() {
                    // TODO: we might want to test if NaN bits are the same.
                    return true;
                }
                lhs == rhs
            }
            (Self::F64(lhs), Self::F64(rhs)) => {
                if lhs.is_nan() && rhs.is_nan() {
                    // TODO: we might want to test if NaN bits are the same.
                    return true;
                }
                lhs == rhs
            }
            _ => false,
        }
    }
}

impl<'a> From<&'a wasmi_reg::Value> for FuzzValue {
    fn from(value: &wasmi_reg::Value) -> Self {
        match value {
            wasmi_reg::Value::I32(value) => Self::I32(*value),
            wasmi_reg::Value::I64(value) => Self::I64(*value),
            wasmi_reg::Value::F32(value) => Self::F32(*value),
            wasmi_reg::Value::F64(value) => Self::F64(*value),
            _ => panic!("unsupported value type"),
        }
    }
}

impl<'a> From<&'a wasmi_stack::Value> for FuzzValue {
    fn from(value: &wasmi_stack::Value) -> Self {
        match value {
            wasmi_stack::Value::I32(value) => Self::I32(*value),
            wasmi_stack::Value::I64(value) => Self::I64(*value),
            wasmi_stack::Value::F32(value) => Self::F32(F32::from_bits(value.to_bits())),
            wasmi_stack::Value::F64(value) => Self::F64(F64::from_bits(value.to_bits())),
            _ => panic!("unsupported value type"),
        }
    }
}

impl<'a> From<&'a wasmtime::Val> for FuzzValue {
    fn from(value: &wasmtime::Val) -> Self {
        match value {
            wasmtime::Val::I32(value) => Self::I32(*value),
            wasmtime::Val::I64(value) => Self::I64(*value),
            wasmtime::Val::F32(value) => Self::F32(F32::from_bits(*value)),
            wasmtime::Val::F64(value) => Self::F64(F64::from_bits(*value)),
            _ => panic!("unsupported value type"),
        }
    }
}
