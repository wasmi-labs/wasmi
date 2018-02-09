
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs::File;
use std::collections::HashMap;

use wasmi::{
    Error as InterpreterError, Externals, FuncRef,
    GlobalInstance, GlobalRef, ImportResolver, ImportsBuilder,
    MemoryInstance, MemoryRef, ModuleImportResolver, ModuleInstance,
    ModuleRef, RuntimeValue, TableInstance, TableRef, ValueType,
    Module, Signature, MemoryDescriptor, Trap,
    TableDescriptor, GlobalDescriptor, FuncInstance, RuntimeArgs,
};
use wasmi::memory_units::Pages;
use wabt::spec::{Action, Visitor as ScriptVisitor, Value, run_spec};

fn spec_to_runtime_value(value: Value) -> RuntimeValue {
	match value {
		Value::I32(v) => RuntimeValue::I32(v),
		Value::I64(v) => RuntimeValue::I64(v),
		Value::F32(v) => RuntimeValue::F32(v),
		Value::F64(v) => RuntimeValue::F64(v),
	}
}

fn runtime_value_to_spec(value: RuntimeValue) -> Value {
	match value {
		RuntimeValue::I32(v) => Value::I32(v),
		RuntimeValue::I64(v) => Value::I64(v),
		RuntimeValue::F32(v) => Value::F32(v),
		RuntimeValue::F64(v) => Value::F64(v),
	}
}

#[derive(Debug)]
enum Error {
    Load(String),
    Start(Trap),
    Interpreter(InterpreterError),
}

impl From<InterpreterError> for Error {
    fn from(e: InterpreterError) -> Error {
        Error::Interpreter(e)
    }
}

struct SpecModule {
    table: TableRef,
    memory: MemoryRef,
    global_i32: GlobalRef,
    global_i64: GlobalRef,
    global_f32: GlobalRef,
    global_f64: GlobalRef,
}

impl SpecModule {
    fn new() -> Self {
        SpecModule {
            table: TableInstance::alloc(10, Some(20)).unwrap(),
            memory: MemoryInstance::alloc(Pages(1), Some(Pages(2))).unwrap(),
            global_i32: GlobalInstance::alloc(RuntimeValue::I32(666), false),
            global_i64: GlobalInstance::alloc(RuntimeValue::I64(666), false),
            global_f32: GlobalInstance::alloc(RuntimeValue::F32(666.0), false),
            global_f64: GlobalInstance::alloc(RuntimeValue::F64(666.0), false),
        }
    }
}

const PRINT_FUNC_INDEX: usize = 0;

impl Externals for SpecModule {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            PRINT_FUNC_INDEX => {
                println!("print: {:?}", args);
                Ok(None)
            }
            _ => panic!("SpecModule doesn't provide function at index {}", index),
        }
    }
}

impl ModuleImportResolver for SpecModule {
    fn resolve_func(
        &self,
        field_name: &str,
        func_type: &Signature,
    ) -> Result<FuncRef, InterpreterError> {
        if field_name == "print" {
            if func_type.return_type().is_some() {
                return Err(InterpreterError::Instantiation(
                    "Function `print` have unit return type".into(),
                ));
            }

            let func = FuncInstance::alloc_host(func_type.clone(), PRINT_FUNC_INDEX);
            return Ok(func);
        }

        Err(InterpreterError::Instantiation(
            format!("Unknown host func import {}", field_name),
        ))
    }

    fn resolve_global(
        &self,
        field_name: &str,
        global_type: &GlobalDescriptor,
    ) -> Result<GlobalRef, InterpreterError> {
        if field_name == "global" {
            return match global_type.value_type() {
                ValueType::I32 => Ok(self.global_i32.clone()),
                ValueType::I64 => Ok(self.global_i64.clone()),
                ValueType::F32 => Ok(self.global_f32.clone()),
                ValueType::F64 => Ok(self.global_f64.clone()),
            };
        }

        Err(InterpreterError::Instantiation(
            format!("Unknown host global import {}", field_name),
        ))
    }

    fn resolve_memory(
        &self,
        field_name: &str,
        _memory_type: &MemoryDescriptor,
    ) -> Result<MemoryRef, InterpreterError> {
        if field_name == "memory" {
            return Ok(self.memory.clone());
        }

        Err(InterpreterError::Instantiation(
            format!("Unknown host memory import {}", field_name),
        ))
    }

    fn resolve_table(
        &self,
        field_name: &str,
        _table_type: &TableDescriptor,
    ) -> Result<TableRef, InterpreterError> {
        if field_name == "table" {
            return Ok(self.table.clone());
        }

        Err(InterpreterError::Instantiation(
            format!("Unknown host table import {}", field_name),
        ))
    }
}

struct SpecDriver {
    spec_module: SpecModule,
    instances: HashMap<String, ModuleRef>,
    last_module: Option<ModuleRef>,
}

impl SpecDriver {
    fn new() -> SpecDriver {
        SpecDriver {
            spec_module: SpecModule::new(),
            instances: HashMap::new(),
            last_module: None,
        }
    }

    fn spec_module(&mut self) -> &mut SpecModule {
        &mut self.spec_module
    }

    fn add_module(&mut self, name: Option<String>, module: ModuleRef) {
        self.last_module = Some(module.clone());
        if let Some(name) = name {
            self.instances.insert(name, module);
        }
    }

    fn module(&self, name: &str) -> Result<ModuleRef, InterpreterError> {
        self.instances.get(name).cloned().ok_or_else(|| {
            InterpreterError::Instantiation(format!("Module not registered {}", name))
        })
    }

    fn module_or_last(&self, name: Option<&str>) -> Result<ModuleRef, InterpreterError> {
        match name {
            Some(name) => self.module(name),
            None => self.last_module.clone().ok_or_else(|| {
                InterpreterError::Instantiation("No modules registered".into())
            }),
        }
    }
}

impl ImportResolver for SpecDriver {
    fn resolve_func(
        &self,
        module_name: &str,
        field_name: &str,
        func_type: &Signature,
    ) -> Result<FuncRef, InterpreterError> {
        if module_name == "spectest" {
            self.spec_module.resolve_func(field_name, func_type)
        } else {
            self.module(module_name)?
                .resolve_func(field_name, func_type)
        }
    }

    fn resolve_global(
        &self,
        module_name: &str,
        field_name: &str,
        global_type: &GlobalDescriptor,
    ) -> Result<GlobalRef, InterpreterError> {
        if module_name == "spectest" {
            self.spec_module.resolve_global(field_name, global_type)
        } else {
            self.module(module_name)?
                .resolve_global(field_name, global_type)
        }
    }

    fn resolve_memory(
        &self,
        module_name: &str,
        field_name: &str,
        memory_type: &MemoryDescriptor,
    ) -> Result<MemoryRef, InterpreterError> {
        if module_name == "spectest" {
            self.spec_module.resolve_memory(field_name, memory_type)
        } else {
            self.module(module_name)?
                .resolve_memory(field_name, memory_type)
        }
    }

    fn resolve_table(
        &self,
        module_name: &str,
        field_name: &str,
        table_type: &TableDescriptor,
    ) -> Result<TableRef, InterpreterError> {
        if module_name == "spectest" {
            self.spec_module.resolve_table(field_name, table_type)
        } else {
            self.module(module_name)?
                .resolve_table(field_name, table_type)
        }
    }
}

fn try_load_module(wasm: &[u8]) -> Result<Module, Error> {
    Module::from_buffer(wasm).map_err(|e| Error::Load(e.to_string()))
}

fn try_load(
    wasm: &[u8],
    spec_driver: &mut SpecDriver,
) -> Result<(), Error> {
    let module = try_load_module(wasm)?;
    let instance = ModuleInstance::new(&module, &ImportsBuilder::default())?;
    instance
        .run_start(spec_driver.spec_module())
        .map_err(|trap| Error::Start(trap))?;
    Ok(())
}

fn load_module(
    wasm: &[u8],
    name: &Option<String>,
    spec_driver: &mut SpecDriver,
) -> ModuleRef {
    let module =
        try_load_module(wasm).expect(&format!("Wasm failed to load"));
    let instance = ModuleInstance::new(&module, spec_driver)
        .expect("Instantiation failed")
        .run_start(spec_driver.spec_module())
        .expect("Run start failed");

    let module_name = name.clone();
    spec_driver.add_module(module_name, instance.clone());

    instance
}

fn run_action(
    program: &mut SpecDriver,
    action: &Action,
) -> Result<Option<RuntimeValue>, InterpreterError> {
    match *action {
        Action::Invoke {
            ref module,
            ref field,
            ref args,
        } => {
            let module = program.module_or_last(module.as_ref().map(|x| x.as_ref())).expect(&format!(
                "Expected program to have loaded module {:?}",
                module
            ));
            module.invoke_export(
                field,
                &args.iter().cloned().map(spec_to_runtime_value).collect::<Vec<_>>(),
                program.spec_module(),
            )
        }
        Action::Get {
            ref module,
            ref field,
            ..
        } => {
            let module = program.module_or_last(module.as_ref().map(|x| x.as_ref())).expect(&format!(
                "Expected program to have loaded module {:?}",
                module
            ));
            let global = module
                .export_by_name(&field)
                .ok_or_else(|| {
                    InterpreterError::Global(format!("Expected to have export with name {}", field))
                })?
                .as_global()
                .cloned()
                .ok_or_else(|| {
                    InterpreterError::Global(format!("Expected export {} to be a global", field))
                })?;
            Ok(Some(global.get()))
        }
    }
}

pub struct FixtureParams {
    failing: bool,
    json: String,
}

pub fn run_wast2wasm(name: &str) -> FixtureParams {
    let outdir = env::var("OUT_DIR").unwrap();

    let mut wast2wasm_path = PathBuf::from(outdir.clone());
    wast2wasm_path.push("bin");
    wast2wasm_path.push("wast2json");

    let mut json_spec_path = PathBuf::from(outdir.clone());
    json_spec_path.push(&format!("{}.json", name));

    let wast2wasm_output = Command::new(wast2wasm_path)
        .arg("-o")
        .arg(&json_spec_path)
        .arg(&format!("./wabt/third_party/testsuite/{}.wast", name))
        .output()
        .expect("Failed to execute process");

    FixtureParams {
        json: json_spec_path.to_str().unwrap().to_owned(),
        failing: {
            if !wast2wasm_output.status.success() {
                println!("wast2json error code: {}", wast2wasm_output.status);
                println!(
                    "wast2json stdout: {}",
                    String::from_utf8_lossy(&wast2wasm_output.stdout)
                );
                println!(
                    "wast2json stderr: {}",
                    String::from_utf8_lossy(&wast2wasm_output.stderr)
                );
                true
            } else {
                false
            }
        },
    }
}

struct SpecRunner {
	spec_driver: SpecDriver,
}

impl SpecRunner {
	fn assert_nans(&mut self, line: u64, action: &Action) -> Result<(), Error> {
		let result = run_action(&mut self.spec_driver, action);
		match result {
			Ok(result) => {
				for actual_result in result.into_iter().collect::<Vec<RuntimeValue>>() {
					match actual_result {
						RuntimeValue::F32(val) => if !val.is_nan() {
							panic!("Expected nan value, got {:?}", val)
						},
						RuntimeValue::F64(val) => if !val.is_nan() {
							panic!("Expected nan value, got {:?}", val)
						},
						val @ _ => {
							panic!("Expected action to return float value, got {:?}", val)
						}
					}
				}
				println!("assert_return_nan at line {} - success", line);
			}
			Err(e) => {
				panic!("Expected action to return value, got error: {:?}", e);
			}
		}
		Ok(())
	}

	fn assert_incorrect_modules(&mut self, line: u64, wasm: &[u8]) -> Result<(), Error> {
		let module_load = try_load(wasm, &mut self.spec_driver);
		match module_load {
			Ok(_) => panic!("Expected invalid module definition, got some module!"),
			Err(e) => println!("assert_invalid at line {} - success ({:?})", line, e),
		}
		Ok(())
	}
}

impl ScriptVisitor<Error> for SpecRunner {
    fn module(&mut self, line: u64, wasm: &[u8], name: Option<String>) -> Result<(), Error> {
        load_module(wasm, &name, &mut self.spec_driver);
		Ok(())
    }

    fn assert_return(&mut self, line: u64, action: &Action, expected: &[Value]) -> Result<(), Error> {
		let result = run_action(&mut self.spec_driver, action);
		match result {
			Ok(result) => {
				let spec_expected = expected.iter().cloned().map(spec_to_runtime_value).collect::<Vec<_>>();
				let actual_result = result.into_iter().collect::<Vec<RuntimeValue>>();
				for (actual_result, spec_expected) in
					actual_result.iter().zip(spec_expected.iter())
				{
					assert_eq!(actual_result.value_type(), spec_expected.value_type());
					// f32::NAN != f32::NAN
					match spec_expected {
						&RuntimeValue::F32(val) if val.is_nan() => match actual_result {
							&RuntimeValue::F32(val) => assert!(val.is_nan()),
							_ => unreachable!(), // checked above that types are same
						},
						&RuntimeValue::F64(val) if val.is_nan() => match actual_result {
							&RuntimeValue::F64(val) => assert!(val.is_nan()),
							_ => unreachable!(), // checked above that types are same
						},
						spec_expected @ _ => assert_eq!(actual_result, spec_expected),
					}
				}
				println!("assert_return at line {} - success", line);
			}
			Err(e) => {
				panic!("Expected action to return value, got error: {:?}", e);
			}
		}
        Ok(())
    }

    fn assert_return_canonical_nan(&mut self, line: u64, action: &Action) -> Result<(), Error> {
        self.assert_nans(line, action)
    }

    fn assert_return_arithmetic_nan(&mut self, line: u64, action: &Action) -> Result<(), Error> {
        self.assert_nans(line, action)
    }

    fn assert_exhaustion(&mut self, line: u64, action: &Action) -> Result<(), Error> {
		let result = run_action(&mut self.spec_driver, action);
		match result {
			Ok(result) => panic!("Expected exhaustion, got result: {:?}", result),
			Err(e) => println!("assert_exhaustion at line {} - success ({:?})", line, e),
		}
        Ok(())
    }

    fn assert_trap(&mut self, line: u64, action: &Action, text: &str) -> Result<(), Error> {
        Ok(())
    }

    fn assert_invalid(&mut self, line: u64, wasm: &[u8], text: &str) -> Result<(), Error> {
        self.assert_incorrect_modules(line, wasm)
    }

    fn assert_malformed(&mut self, line: u64, wasm: &[u8], text: &str) -> Result<(), Error> {
        self.assert_incorrect_modules(line, wasm)
    }

    fn assert_unlinkable(&mut self, line: u64, wasm: &[u8], text: &str) -> Result<(), Error> {
        self.assert_incorrect_modules(line, wasm)
    }

    fn assert_uninstantiable(&mut self, line: u64, wasm: &[u8], text: &str) -> Result<(), Error> {
        match try_load(wasm, &mut self.spec_driver) {
			Ok(_) => panic!("Expected error running start function at line {}", line),
			Err(e) => println!("assert_uninstantiable - success ({:?})", e),
		}
		Ok(())
    }

    fn register(&mut self, line: u64, name: Option<&str>, as_name: &str) -> Result<(), Error> {
		let module = match self.spec_driver.module_or_last(name.as_ref().map(|x| x.as_ref())) {
			Ok(module) => module,
			Err(e) => panic!("No such module, at line {} - ({:?})", e, line),
		};
		self.spec_driver.add_module(Some(as_name.to_owned()), module);
        Ok(())
    }

    fn perform_action(&mut self, line: u64, action: &Action) -> Result<(), Error> {
		match run_action(&mut self.spec_driver, action) {
			Ok(_) => {}
			Err(e) => panic!("Failed to invoke action at line {}: {:?}", line, e),
		}
        Ok(())
    }
}

pub fn spec(name: &str) {
    // let tmpdir = env::var("OUT_DIR").unwrap();

    // let fixture = run_wast2wasm(name);

    // let wast2wasm_fail_expected = name.ends_with(".fail");
    // if wast2wasm_fail_expected {
    //     if !fixture.failing {
    //         panic!("wast2json expected to fail, but terminated normally");
    //     }
    //     // Failing fixture, bail out.
    //     return;
    // }

    // if fixture.failing {
    //     panic!("wast2json terminated abnormally, expected to success");
    // }

	println!("running test: {}", name);

	let mut spec_runner = SpecRunner {
		spec_driver: SpecDriver::new(),
	};
	let spec_script_path = format!("./wabt/third_party/testsuite/{}.wast", name);
	run_spec(spec_script_path, &mut spec_runner).expect("success");
}
