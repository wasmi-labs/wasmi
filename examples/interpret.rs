// In this example we execute a contract funciton exported as "_call"

extern crate wasmi;

use std::{env::args, fs::File};
use wasmi::{ImportsBuilder, Module, ModuleInstance, NopExternals, RuntimeValue};

fn load_from_file(filename: &str) -> Module {
    use std::io::prelude::*;
    let mut file = File::open(filename).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    Module::from_buffer(buf).unwrap()
}

fn main() {
    let args: Vec<_> = args().collect();
    if args.len() != 3 {
        println!("Usage: {} <wasm file> <arg>", args[0]);
        println!("    wasm file should contain exported `_call` function with single I32 argument");
        return;
    }

    // Here we load module using dedicated for this purpose
    // `load_from_file` function (which works only with modules)
    let module = load_from_file(&args[1]);

    // Intialize deserialized module. It adds module into It expects 3 parameters:
    // - a name for the module
    // - a module declaration
    // - "main" module doesn't import native module(s) this is why we don't need to provide external native modules here
    // This test shows how to implement native module https://github.com/NikVolf/parity-wasm/blob/master/src/interpreter/tests/basics.rs#L197
    let main = ModuleInstance::new(&module, &ImportsBuilder::default())
        .expect("Failed to instantiate module")
        .run_start(&mut NopExternals)
        .expect("Failed to run start function in module");

    // The argument should be parsable as a valid integer
    let argument: i32 = args[2].parse().expect("Integer argument required");

    // "_call" export of function to be executed with an i32 argument and prints the result of execution
    println!(
        "Result: {:?}",
        main.invoke_export("_call", &[RuntimeValue::I32(argument)], &mut NopExternals)
    );
}
