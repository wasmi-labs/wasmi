#![cfg(test)]

use std::collections::HashMap;

use wasmi::{
    Error as InterpreterError, Externals, FuncInstance, FuncRef, GlobalDescriptor,
    GlobalInstance, GlobalRef, ImportResolver, ImportsBuilder, MemoryDescriptor,
    MemoryInstance, MemoryRef, Module, ModuleImportResolver, ModuleInstance, ModuleRef,
    RuntimeArgs, RuntimeValue, Signature, TableDescriptor, TableInstance, TableRef, Trap,
};
use wasmi::memory_units::Pages;
use wabt::script::{self, Action, Command, CommandKind, ScriptParser, Value};

fn spec_to_runtime_value(value: Value) -> RuntimeValue {
    match value {
        Value::I32(v) => RuntimeValue::I32(v),
        Value::I64(v) => RuntimeValue::I64(v),
        Value::F32(v) => RuntimeValue::F32(v),
        Value::F64(v) => RuntimeValue::F64(v),
    }
}

#[derive(Debug)]
enum Error {
    Load(String),
    Start(Trap),
    Script(script::Error),
    Interpreter(InterpreterError),
}

impl From<InterpreterError> for Error {
    fn from(e: InterpreterError) -> Error {
        Error::Interpreter(e)
    }
}

impl From<script::Error> for Error {
    fn from(e: script::Error) -> Error {
        Error::Script(e)
    }
}

struct SpecModule {
    table: TableRef,
    memory: MemoryRef,
    global_i32: GlobalRef,
    global_f32: GlobalRef,
    global_f64: GlobalRef,
}

impl SpecModule {
    fn new() -> Self {
        SpecModule {
            table: TableInstance::alloc(10, Some(20)).unwrap(),
            memory: MemoryInstance::alloc(Pages(1), Some(Pages(2))).unwrap(),
            global_i32: GlobalInstance::alloc(RuntimeValue::I32(666), false),
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
		let index = match field_name {
			"print" => PRINT_FUNC_INDEX,
			"print_i32" => PRINT_FUNC_INDEX,
			"print_i32_f32" => PRINT_FUNC_INDEX,
			"print_f64_f64" => PRINT_FUNC_INDEX,
			"print_f32" => PRINT_FUNC_INDEX,
			"print_f64" => PRINT_FUNC_INDEX,
			_ => {
				return Err(InterpreterError::Instantiation(format!(
					"Unknown host func import {}",
					field_name
				)));
			}
		};

		if func_type.return_type().is_some() {
			return Err(InterpreterError::Instantiation(
				"Function `print_` have unit return type".into(),
			));
		}

        let func = FuncInstance::alloc_host(func_type.clone(), index);
		return Ok(func);
    }

    fn resolve_global(
        &self,
        field_name: &str,
        _global_type: &GlobalDescriptor,
    ) -> Result<GlobalRef, InterpreterError> {
		match field_name {
			"global_i32" => Ok(self.global_i32.clone()),
			"global_f32" => Ok(self.global_f32.clone()),
			"global_f64" => Ok(self.global_f64.clone()),
			_ => Err(InterpreterError::Instantiation(format!(
				"Unknown host global import {}",
				field_name
			)))
		}
    }

    fn resolve_memory(
        &self,
        field_name: &str,
        _memory_type: &MemoryDescriptor,
    ) -> Result<MemoryRef, InterpreterError> {
        if field_name == "memory" {
            return Ok(self.memory.clone());
        }

        Err(InterpreterError::Instantiation(format!(
            "Unknown host memory import {}",
            field_name
        )))
    }

    fn resolve_table(
        &self,
        field_name: &str,
        _table_type: &TableDescriptor,
    ) -> Result<TableRef, InterpreterError> {
        if field_name == "table" {
            return Ok(self.table.clone());
        }

        Err(InterpreterError::Instantiation(format!(
            "Unknown host table import {}",
            field_name
        )))
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
            None => self.last_module
                .clone()
                .ok_or_else(|| InterpreterError::Instantiation("No modules registered".into())),
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

fn try_load(wasm: &[u8], spec_driver: &mut SpecDriver) -> Result<(), Error> {
    let module = try_load_module(wasm)?;
    let instance = ModuleInstance::new(&module, &ImportsBuilder::default())?;
    instance
        .run_start(spec_driver.spec_module())
        .map_err(|trap| Error::Start(trap))?;
    Ok(())
}

fn load_module(wasm: &[u8], name: &Option<String>, spec_driver: &mut SpecDriver) -> Result<ModuleRef, Error> {
    let module = try_load_module(wasm)?;
    let instance = ModuleInstance::new(&module, spec_driver)
        .map_err(|e| Error::Load(e.to_string()))?
        .run_start(spec_driver.spec_module())
        .map_err(|trap| Error::Start(trap))?;

    let module_name = name.clone();
    spec_driver.add_module(module_name, instance.clone());

    Ok(instance)
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
            let module = program
                .module_or_last(module.as_ref().map(|x| x.as_ref()))
                .expect(&format!(
                    "Expected program to have loaded module {:?}",
                    module
                ));
            module.invoke_export(
                field,
                &args.iter()
                    .cloned()
                    .map(spec_to_runtime_value)
                    .collect::<Vec<_>>(),
                program.spec_module(),
            )
        }
        Action::Get {
            ref module,
            ref field,
            ..
        } => {
            let module = program
                .module_or_last(module.as_ref().map(|x| x.as_ref()))
                .expect(&format!(
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

pub fn spec(name: &str) {
    println!("running test: {}", name);
    try_spec(name).expect("Failed to run spec");
}

fn try_spec(name: &str) -> Result<(), Error> {
    let mut spec_driver = SpecDriver::new();
    let spec_script_path = format!("tests/spec/testsuite/{}.wast", name);
    let mut parser = ScriptParser::from_file(spec_script_path).expect("Can't read spec script");
    while let Some(Command { kind, line }) = parser.next()? {
		println!("Line {}:", line);
        match kind {
            CommandKind::Module { name, module, .. } => {
                load_module(&module.into_vec()?, &name, &mut spec_driver).expect("Failed to load module");
            }
            CommandKind::AssertReturn { action, expected } => {
                let result = run_action(&mut spec_driver, &action);
                match result {
                    Ok(result) => {
                        let spec_expected = expected
                            .iter()
                            .cloned()
                            .map(spec_to_runtime_value)
                            .collect::<Vec<_>>();
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
            }
            CommandKind::AssertReturnCanonicalNan { action }
            | CommandKind::AssertReturnArithmeticNan { action } => {
                let result = run_action(&mut spec_driver, &action);
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
            }
            CommandKind::AssertExhaustion { action, .. } => {
                let result = run_action(&mut spec_driver, &action);
                match result {
                    Ok(result) => panic!("Expected exhaustion, got result: {:?}", result),
                    Err(e) => println!("assert_exhaustion at line {} - success ({:?})", line, e),
                }
            }
            CommandKind::AssertTrap { action, .. } => {
                let result = run_action(&mut spec_driver, &action);
                match result {
                    Ok(result) => {
                        panic!(
                            "Expected action to result in a trap, got result: {:?}",
                            result
                        );
                    }
                    Err(e) => {
                        println!("assert_trap at line {} - success ({:?})", line, e);
                    }
                }
            }
            CommandKind::AssertInvalid { module, .. }
            | CommandKind::AssertMalformed { module, .. }
            | CommandKind::AssertUnlinkable { module, .. } => {
                let module_load = try_load(&module.into_vec()?, &mut spec_driver);
                match module_load {
                    Ok(_) => panic!("Expected invalid module definition, got some module!"),
                    Err(e) => println!("assert_invalid at line {} - success ({:?})", line, e),
                }
            }
            CommandKind::AssertUninstantiable { module, .. } => {
                match try_load(&module.into_vec()?, &mut spec_driver) {
                    Ok(_) => panic!("Expected error running start function at line {}", line),
                    Err(e) => println!("assert_uninstantiable - success ({:?})", e),
                }
            }
            CommandKind::Register { name, as_name, .. } => {
                let module = match spec_driver.module_or_last(name.as_ref().map(|x| x.as_ref())) {
                    Ok(module) => module,
                    Err(e) => panic!("No such module, at line {} - ({:?})", e, line),
                };
                spec_driver.add_module(Some(as_name.clone()), module);
            }
            CommandKind::PerformAction(action) => match run_action(&mut spec_driver, &action) {
                Ok(_) => {}
                Err(e) => panic!("Failed to invoke action at line {}: {:?}", line, e),
            },
        }
    }

    Ok(())
}
