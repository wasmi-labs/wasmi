// Test-only code importing std for no-std testing
extern crate std;

use crate::memory_units::Pages;
use crate::{
    Error, FuncRef, GlobalDescriptor, GlobalInstance, GlobalRef, ImportsBuilder, MemoryDescriptor,
    MemoryInstance, MemoryRef, Module, ModuleImportResolver, ModuleInstance, NopExternals,
    RuntimeValue, Signature, TableDescriptor, TableInstance, TableRef,
};
use alloc::vec::Vec;
use std::fs::File;

struct Env {
    table_base: GlobalRef,
    memory_base: GlobalRef,
    memory: MemoryRef,
    table: TableRef,
}

impl Env {
    fn new() -> Env {
        Env {
            table_base: GlobalInstance::alloc(RuntimeValue::I32(0), false),
            memory_base: GlobalInstance::alloc(RuntimeValue::I32(0), false),
            memory: MemoryInstance::alloc(Pages(256), None).unwrap(),
            table: TableInstance::alloc(64, None).unwrap(),
        }
    }
}

impl ModuleImportResolver for Env {
    fn resolve_func(&self, _field_name: &str, _func_type: &Signature) -> Result<FuncRef, Error> {
        Err(Error::Instantiation(
            "env module doesn't provide any functions".into(),
        ))
    }

    fn resolve_global(
        &self,
        field_name: &str,
        _global_type: &GlobalDescriptor,
    ) -> Result<GlobalRef, Error> {
        match field_name {
            "tableBase" => Ok(self.table_base.clone()),
            "memoryBase" => Ok(self.memory_base.clone()),
            _ => Err(Error::Instantiation(format!(
                "env module doesn't provide global '{}'",
                field_name
            ))),
        }
    }

    fn resolve_memory(
        &self,
        field_name: &str,
        _memory_type: &MemoryDescriptor,
    ) -> Result<MemoryRef, Error> {
        match field_name {
            "memory" => Ok(self.memory.clone()),
            _ => Err(Error::Instantiation(format!(
                "env module doesn't provide memory '{}'",
                field_name
            ))),
        }
    }

    fn resolve_table(
        &self,
        field_name: &str,
        _table_type: &TableDescriptor,
    ) -> Result<TableRef, Error> {
        match field_name {
            "table" => Ok(self.table.clone()),
            _ => Err(Error::Instantiation(format!(
                "env module doesn't provide table '{}'",
                field_name
            ))),
        }
    }
}

fn load_from_file(filename: &str) -> Module {
    use std::io::prelude::*;
    let mut file = File::open(filename).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let wasm_buf = ::wabt::wat2wasm(&buf).unwrap();
    Module::from_buffer(wasm_buf).unwrap()
}

#[test]
fn interpreter_inc_i32() {
    // Name of function contained in WASM file (note the leading underline)
    const FUNCTION_NAME: &str = "_inc_i32";
    // The WASM file containing the module and function
    const WASM_FILE: &str = &"res/fixtures/inc_i32.wast";

    let module = load_from_file(WASM_FILE);

    let env = Env::new();

    let instance = ModuleInstance::new(&module, &ImportsBuilder::new().with_resolver("env", &env))
        .expect("Failed to instantiate module")
        .assert_no_start();

    let i32_val = 42;
    // the functions expects a single i32 parameter
    let args = &[RuntimeValue::I32(i32_val)];
    let exp_retval = Some(RuntimeValue::I32(i32_val + 1));

    let retval = instance
        .invoke_export(FUNCTION_NAME, args, &mut NopExternals)
        .expect("");
    assert_eq!(exp_retval, retval);
}

#[test]
fn interpreter_accumulate_u8() {
    // Name of function contained in WASM file (note the leading underline)
    const FUNCTION_NAME: &str = "_accumulate_u8";
    // The WASM file containing the module and function
    const WASM_FILE: &str = &"res/fixtures/accumulate_u8.wast";
    // The octet sequence being accumulated
    const BUF: &[u8] = &[9, 8, 7, 6, 5, 4, 3, 2, 1];

    // Load the module-structure from wasm-file and add to program
    let module = load_from_file(WASM_FILE);

    let env = Env::new();
    let instance = ModuleInstance::new(&module, &ImportsBuilder::new().with_resolver("env", &env))
        .expect("Failed to instantiate module")
        .assert_no_start();

    let env_memory = env.memory;

    // Place the octet-sequence at index 0 in linear memory
    let offset: u32 = 0;
    let _ = env_memory.set(offset, BUF);

    // Set up the function argument list and invoke the function
    let args = &[
        RuntimeValue::I32(BUF.len() as i32),
        RuntimeValue::I32(offset as i32),
    ];
    let retval = instance
        .invoke_export(FUNCTION_NAME, args, &mut NopExternals)
        .expect("Failed to execute function");

    // For verification, repeat accumulation using native code
    let accu = BUF.iter().fold(0_i32, |a, b| a + *b as i32);
    let exp_retval: Option<RuntimeValue> = Some(RuntimeValue::I32(accu));

    // Verify calculation from WebAssembly runtime is identical to expected result
    assert_eq!(exp_retval, retval);
}
