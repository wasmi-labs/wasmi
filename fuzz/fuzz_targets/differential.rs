#![no_main]

mod utils;

use libfuzzer_sys::fuzz_target;
use std::{collections::hash_map::RandomState, mem};
use utils::{ty_to_val, ExecConfig};
use wasm_smith::ConfiguredModule;
use wasmi as wasmi_reg;
use wasmi_reg::core::{F32, F64};

/// Names of exported items.
#[derive(Debug, Default)]
pub struct Exports {
    /// Names of exported functions.
    funcs: Vec<Box<str>>,
    /// Names of exported global variables.
    globals: Vec<Box<str>>,
    /// Names of exported linear memories.
    memories: Vec<Box<str>>,
    /// Names of exported tables.
    tables: Vec<Box<str>>,
}

/// Trait implemented by differential fuzzing backends.
trait DifferentialTarget: Sized {
    /// The value type of the backend.
    type Value;
    /// The error type of the backend.
    type Error;

    /// Sets up the store and exported functions for the backend if possible.
    fn setup(wasm: &[u8]) -> Option<Self>;

    /// Calls the exported function with `name` and returns the result.
    fn call(&mut self, name: &str) -> Result<Box<[FuzzValue]>, Self::Error>;

    /// Returns the value of the global named `name` if any.
    fn get_global(&mut self, name: &str) -> Option<FuzzValue>;

    /// Returns the bytes of the memory named `name` if any.
    fn get_memory(&mut self, name: &str) -> Option<&[u8]>;
}

/// Differential fuzzing backend for the register-machine Wasmi.
#[derive(Debug)]
struct WasmiRegister {
    store: wasmi_reg::Store<wasmi_reg::StoreLimits>,
    instance: wasmi_reg::Instance,
    params: Vec<wasmi_reg::Val>,
    results: Vec<wasmi_reg::Val>,
}

impl WasmiRegister {
    /// Returns the names of all exported items.
    pub fn exports(&self) -> Exports {
        let mut exports = Exports::default();
        for export in self.instance.exports(&self.store) {
            let name = export.name();
            let dst = match export.ty(&self.store) {
                wasmi::ExternType::Func(_) => &mut exports.funcs,
                wasmi::ExternType::Global(_) => &mut exports.globals,
                wasmi::ExternType::Memory(_) => &mut exports.memories,
                wasmi::ExternType::Table(_) => &mut exports.tables,
            };
            dst.push(name.into());
        }
        exports
    }

    fn type_to_value(ty: &wasmi_reg::core::ValType) -> wasmi_reg::Val {
        ty_to_val(ty)
    }
}

impl DifferentialTarget for WasmiRegister {
    type Value = FuzzValue;
    type Error = wasmi_reg::Error;

    fn call(&mut self, name: &str) -> Result<Box<[Self::Value]>, Self::Error> {
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
        let results = self.results.iter().map(FuzzValue::from).collect();
        Ok(results)
    }

    fn setup(wasm: &[u8]) -> Option<Self> {
        use wasmi_reg::{Config, Engine, Linker, Module, StackLimits, Store, StoreLimitsBuilder};
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
        let engine = Engine::new(&config);
        let linker = Linker::new(&engine);
        let limiter = StoreLimitsBuilder::new()
            .memory_size(1000 * 0x10000)
            .build();
        let mut store = Store::new(&engine, limiter);
        store.limiter(|lim| lim);
        let module = Module::new_streaming(store.engine(), wasm).unwrap();
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

    fn get_global(&mut self, name: &str) -> Option<FuzzValue> {
        let value = self
            .instance
            .get_global(&self.store, name)?
            .get(&self.store);
        Some(FuzzValue::from(&value))
    }

    fn get_memory(&mut self, name: &str) -> Option<&[u8]> {
        let data = self
            .instance
            .get_memory(&self.store, name)?
            .data(&self.store);
        Some(data)
    }
}

/// Differential fuzzing backend for the stack-machine Wasmi.
#[derive(Debug)]
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

    fn call(&mut self, name: &str) -> Result<Box<[FuzzValue]>, Self::Error> {
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
        let results = self.results.iter().map(FuzzValue::from).collect();
        Ok(results)
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

    fn get_global(&mut self, name: &str) -> Option<FuzzValue> {
        let value = self
            .instance
            .get_global(&self.store, name)?
            .get(&self.store);
        Some(FuzzValue::from(&value))
    }

    fn get_memory(&mut self, name: &str) -> Option<&[u8]> {
        let data = self
            .instance
            .get_memory(&self.store, name)?
            .data(&self.store);
        Some(data)
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

    fn call(&mut self, name: &str) -> Result<Box<[FuzzValue]>, Self::Error> {
        let Some(func) = self.instance.get_func(&mut self.store, name) else {
            panic!("wasmtime is missing exported function {name} that exists in wasmi (register)")
        };
        let ty = func.ty(&self.store);
        self.params.clear();
        self.results.clear();
        self.params.extend(ty.params().map(Self::type_to_value));
        self.results.extend(ty.results().map(Self::type_to_value));
        func.call(&mut self.store, &self.params[..], &mut self.results[..])?;
        let results = self.results.iter().map(FuzzValue::from).collect();
        Ok(results)
    }

    fn setup(wasm: &[u8]) -> Option<Self> {
        use wasmtime::{Config, Engine, Linker, Module, Store, StoreLimitsBuilder};
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

    fn get_global(&mut self, name: &str) -> Option<FuzzValue> {
        let value = self
            .instance
            .get_global(&mut self.store, name)?
            .get(&mut self.store);
        Some(FuzzValue::from(&value))
    }

    fn get_memory(&mut self, name: &str) -> Option<&[u8]> {
        let data = self
            .instance
            .get_memory(&mut self.store, name)?
            .data(&mut self.store);
        Some(data)
    }
}

#[allow(dead_code)] // Note: dead code analysis somehow ignores Debug impl usage.
#[derive(Debug, Default)]
pub struct UnmatchedState {
    globals: Vec<UnmatchedGlobal>,
    memories: Vec<UnmatchedMemory>,
}

impl UnmatchedState {
    fn new(
        exports: &Exports,
        wasmi_reg: &mut WasmiRegister,
        wasmi_stack: &mut WasmiStack,
        wasmtime: &mut Wasmtime,
    ) -> Self {
        let mut unmatched = Self::default();
        unmatched.extract_unmatched_globals(&exports.globals, wasmi_reg, wasmi_stack, wasmtime);
        unmatched.extract_unmatched_memories(&exports.memories, wasmi_reg, wasmi_stack, wasmtime);
        unmatched
    }

    fn is_empty(&self) -> bool {
        self.globals.is_empty() && self.memories.is_empty()
    }

    fn extract_unmatched_globals(
        &mut self,
        globals: &[Box<str>],
        wasmi_reg: &mut WasmiRegister,
        wasmi_stack: &mut WasmiStack,
        wasmtime: &mut Wasmtime,
    ) {
        for name in globals {
            let Some(value_reg) = wasmi_reg.get_global(name) else {
                panic!("missing global for Wasmi (register): {name}")
            };
            let Some(value_stack) = wasmi_stack.get_global(name) else {
                panic!("missing global for Wasmi (stack): {name}")
            };
            let Some(value_wasmtime) = wasmtime.get_global(name) else {
                panic!("missing global for Wasmtime: {name}")
            };
            if let Some(entry) = UnmatchedGlobal::new(name, value_reg, value_stack, value_wasmtime)
            {
                self.push_global(entry);
            }
        }
    }

    fn extract_unmatched_memories(
        &mut self,
        memories: &[Box<str>],
        wasmi_reg: &mut WasmiRegister,
        wasmi_stack: &mut WasmiStack,
        wasmtime: &mut Wasmtime,
    ) {
        for name in memories {
            let Some(memory_reg) = wasmi_reg.get_memory(name) else {
                panic!("missing linear memory for Wasmi (register): {name}")
            };
            let Some(memory_stack) = wasmi_stack.get_memory(name) else {
                panic!("missing linear memory for Wasmi (stack): {name}")
            };
            let Some(memory_wasmtime) = wasmtime.get_memory(name) else {
                panic!("missing linear memory for Wasmtime: {name}")
            };
            if let Some(entry) =
                UnmatchedMemory::new(name, memory_reg, memory_stack, memory_wasmtime)
            {
                self.push_memory(entry);
            }
        }
    }

    fn push_global(&mut self, global: UnmatchedGlobal) {
        self.globals.push(global);
    }

    fn push_memory(&mut self, memory: UnmatchedMemory) {
        self.memories.push(memory);
    }
}

#[allow(dead_code)] // Note: dead code analysis somehow ignores Debug impl usage.
#[derive(Debug)]
pub struct UnmatchedGlobal {
    name: Box<str>,
    value_register: FuzzValue,
    value_stack: FuzzValue,
    value_wasmtime: FuzzValue,
}

impl UnmatchedGlobal {
    /// Returns an [`UnmatchedGlobal`] if either `value_stack` or `value_wasmtime` differs from `value_register`.
    ///
    /// Returns `None` otherwise.
    pub fn new(
        name: &str,
        value_register: FuzzValue,
        value_stack: FuzzValue,
        value_wasmtime: FuzzValue,
    ) -> Option<Self> {
        if (value_register != value_stack) || (value_register != value_wasmtime) {
            return Some(Self {
                name: name.into(),
                value_register,
                value_stack,
                value_wasmtime,
            });
        }
        None
    }
}

#[allow(dead_code)] // Note: dead code analysis somehow ignores Debug impl usage.
#[derive(Debug)]
pub struct UnmatchedMemory {
    name: Box<str>,
    hash_register: u64,
    hash_stack: u64,
    hash_wasmtime: u64,
}

impl UnmatchedMemory {
    /// Returns an [`UnmatchedMemory`] if either `memory_stack` or `memory_wasmtime` differs from `memory_register`.
    ///
    /// Returns `None` otherwise.
    pub fn new(
        name: &str,
        memory_register: &[u8],
        memory_stack: &[u8],
        memory_wasmtime: &[u8],
    ) -> Option<Self> {
        use std::hash::BuildHasher as _;
        let hasher = RandomState::new();
        let hash_register = hasher.hash_one(memory_register);
        let hash_stack = hasher.hash_one(memory_stack);
        let hash_wasmtime = hasher.hash_one(memory_wasmtime);
        if (hash_register != hash_stack) || (hash_register != hash_wasmtime) {
            return Some(Self {
                name: name.into(),
                hash_register,
                hash_stack,
                hash_wasmtime,
            });
        }
        None
    }
}

#[derive(Debug)]
pub struct FuzzContext {
    wasm: Vec<u8>,
    wasmi_register: WasmiRegister,
    wasmi_stack: WasmiStack,
    exports: Exports,
}

impl FuzzContext {
    pub fn run(&mut self) {
        for name in mem::take(&mut self.exports.funcs) {
            let result_reg = self.wasmi_register.call(&name);
            let result_stack = self.wasmi_stack.call(&name);
            match (result_reg, result_stack) {
                (Err(err_reg), Err(err_stack)) => self.both_error(&name, err_reg, err_stack),
                (Ok(ok_reg), Err(err_stack)) => self.reg_ok_stack_err(&name, &ok_reg, err_stack),
                (Err(err_reg), Ok(ok_stack)) => self.reg_err_stack_ok(&name, err_reg, &ok_stack),
                (Ok(ok_reg), Ok(ok_stack)) => self.both_ok(&name, &ok_reg, &ok_stack),
            }
        }
    }

    fn unmatched_state(&mut self, wasmtime: &mut Wasmtime) -> UnmatchedState {
        UnmatchedState::new(
            &self.exports,
            &mut self.wasmi_register,
            &mut self.wasmi_stack,
            wasmtime,
        )
    }

    fn both_error(
        &mut self,
        func_name: &str,
        error_reg: <WasmiRegister as DifferentialTarget>::Error,
        error_stack: <WasmiStack as DifferentialTarget>::Error,
    ) {
        let errstr_reg = error_reg.to_string();
        let errstr_stack = error_stack.to_string();
        if errstr_reg == errstr_stack {
            // Bail out since both Wasmi (register) and Wasmi (stack) agree on the execution failure.
            return;
        }
        let Some(mut wasmtime) = <Wasmtime as DifferentialTarget>::setup(&self.wasm) else {
            panic!("failed to setup Wasmtime fuzzing backend");
        };
        let result_wasmtime = wasmtime.call(func_name);
        let unmatched_state = self.unmatched_state(&mut wasmtime);
        panic!(
            "\
            Wasmi (register) and Wasmi (stack) both fail with different error codes:\n\
            \x20   Function: {func_name:?}\n\
            \x20   Wasmi (register): {errstr_reg}\n\
            \x20   Wasmi (stack)   : {errstr_stack}\n\
            \x20   Wasmtime        : {result_wasmtime:?}\n\
            \n\
            {unmatched_state:#?}",
        )
    }

    fn reg_ok_stack_err(
        &mut self,
        func_name: &str,
        results_reg: &[FuzzValue],
        error_stack: <WasmiStack as DifferentialTarget>::Error,
    ) {
        let errstr_stack = error_stack.to_string();
        let Some(mut wasmtime) = <Wasmtime as DifferentialTarget>::setup(&self.wasm) else {
            panic!("failed to setup Wasmtime fuzzing backend");
        };
        let result_wasmtime = wasmtime.call(func_name);
        let unmatched_state = self.unmatched_state(&mut wasmtime);
        panic!(
            "\
            Wasmi (register) succeeded and Wasmi (stack) failed:\n\
            \x20   Function: {func_name:?}\n\
            \x20   Wasmi (register): {results_reg:?}\n\
            \x20   Wasmi (stack)   : {errstr_stack}\n\
            \x20   Wasmtime        : {result_wasmtime:?}\n\
            \n\
            {unmatched_state:#?}",
        )
    }

    fn reg_err_stack_ok(
        &mut self,
        func_name: &str,
        error_reg: <WasmiRegister as DifferentialTarget>::Error,
        results_stack: &[FuzzValue],
    ) {
        let errstr_reg = error_reg.to_string();
        let Some(mut wasmtime) = <Wasmtime as DifferentialTarget>::setup(&self.wasm) else {
            panic!("failed to setup Wasmtime fuzzing backend");
        };
        let results_wasmtime = wasmtime.call(func_name);
        let unmatched_state = self.unmatched_state(&mut wasmtime);
        panic!(
            "\
            Wasmi (register) failed and Wasmi (stack) succeeded:\n\
            \x20   Function: {func_name:?}\n\
            \x20   Wasmi (register): {errstr_reg}\n\
            \x20   Wasmi (stack)   : {results_stack:?}\n\
            \x20   Wasmtime        : {results_wasmtime:?}\n\
            \n\
            {unmatched_state:#?}",
        )
    }

    fn both_ok(&mut self, func_name: &str, results_reg: &[FuzzValue], results_stack: &[FuzzValue]) {
        if results_reg == results_stack {
            // Bail out since both Wasmi (register) and Wasmi (stack) agree on the execution results.
            return;
        }
        let Some(mut wasmtime) = <Wasmtime as DifferentialTarget>::setup(&self.wasm) else {
            panic!("failed to setup Wasmtime fuzzing backend");
        };
        let results_wasmtime = wasmtime.call(func_name).unwrap_or_else(|error| {
            panic!("failed to execute func ({func_name}) via Wasmtime fuzzing backend: {error}")
        });
        let text = match (
            &results_wasmtime[..] == results_reg,
            &results_wasmtime[..] == results_stack,
        ) {
            (true, false) => "Wasmi (stack) disagrees with Wasmi (register) and Wasmtime",
            (false, true) => "Wasmi (register) disagrees with Wasmi (stack) and Wasmtime",
            (false, false) => "Wasmi (register), Wasmi (stack) and Wasmtime disagree",
            (true, true) => unreachable!("results_reg and results_stack differ"),
        };
        let unmatched_state = self.unmatched_state(&mut wasmtime);
        println!(
            "{text} for function execution: {func_name:?}\n\
            \x20   Wasmi (register): {results_reg:?}\n\
            \x20   Wasmi (stack)   : {results_stack:?}\n\
            \x20   Wasmtime        : {results_wasmtime:?}\n\
            \n\
            {unmatched_state:#?}",
        );
        if &results_wasmtime[..] != results_reg || !unmatched_state.is_empty() {
            panic!()
        }
    }
}

fuzz_target!(|cfg_module: ConfiguredModule<ExecConfig>| {
    let mut smith_module = cfg_module.module;
    // Note: We cannot use built-in fuel metering of the different engines since that
    //       would introduce unwanted non-determinism with respect to fuzz testing.
    smith_module.ensure_termination(1_000 /* fuel */);
    let wasm = smith_module.to_bytes();
    let Some(wasmi_register) = <WasmiRegister as DifferentialTarget>::setup(&wasm[..]) else {
        return;
    };
    let Some(wasmi_stack) = <WasmiStack as DifferentialTarget>::setup(&wasm[..]) else {
        panic!("wasmi (register) succeeded to create Context while wasmi (stack) failed");
    };
    let exports = wasmi_register.exports();
    let mut context = FuzzContext {
        wasm,
        wasmi_register,
        wasmi_stack,
        exports,
    };
    context.run();
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

impl<'a> From<&'a wasmi_reg::Val> for FuzzValue {
    fn from(value: &wasmi_reg::Val) -> Self {
        match value {
            wasmi_reg::Val::I32(value) => Self::I32(*value),
            wasmi_reg::Val::I64(value) => Self::I64(*value),
            wasmi_reg::Val::F32(value) => Self::F32(*value),
            wasmi_reg::Val::F64(value) => Self::F64(*value),
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
