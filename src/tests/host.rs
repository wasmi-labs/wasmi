// Test-only code importing std for no-std testing
extern crate std;

use super::parse_wat;
use crate::memory_units::Pages;
use crate::types::ValueType;
use crate::{
    Error, Externals, FuncInstance, FuncRef, HostError, ImportsBuilder, MemoryDescriptor,
    MemoryInstance, MemoryRef, ModuleImportResolver, ModuleInstance, ModuleRef, ResumableError,
    RuntimeArgs, RuntimeValue, Signature, TableDescriptor, TableInstance, TableRef, Trap, TrapKind,
};
use alloc::boxed::Box;
use std::println;

#[derive(Debug, Clone, PartialEq)]
struct HostErrorWithCode {
    error_code: u32,
}

impl ::core::fmt::Display for HostErrorWithCode {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> Result<(), ::core::fmt::Error> {
        write!(f, "{}", self.error_code)
    }
}

impl HostError for HostErrorWithCode {}

/// Host state for the test environment.
///
/// This struct can be used as an external function executor and
/// as imports provider. This has a drawback: this struct
/// should be provided upon an instantiation of the module.
///
/// However, this limitation can be lifted by implementing `Externals`
/// and `ModuleImportResolver` traits for different structures.
/// See `defer_providing_externals` test for details.
struct TestHost {
    memory: Option<MemoryRef>,
    instance: Option<ModuleRef>,

    trap_sub_result: Option<RuntimeValue>,
}

impl TestHost {
    fn new() -> TestHost {
        TestHost {
            memory: Some(MemoryInstance::alloc(Pages(1), Some(Pages(1))).unwrap()),
            instance: None,

            trap_sub_result: None,
        }
    }
}

/// sub(a: i32, b: i32) -> i32
///
/// This function just substracts one integer from another,
/// returning the subtraction result.
const SUB_FUNC_INDEX: usize = 0;

/// err(error_code: i32) -> !
///
/// This function traps upon a call.
/// The trap have a special type - HostErrorWithCode.
const ERR_FUNC_INDEX: usize = 1;

/// inc_mem(ptr: *mut u8)
///
/// Increments value at the given address in memory. This function
/// requires attached memory.
const INC_MEM_FUNC_INDEX: usize = 2;

/// get_mem(ptr: *mut u8) -> u8
///
/// Returns value at the given address in memory. This function
/// requires attached memory.
const GET_MEM_FUNC_INDEX: usize = 3;

/// recurse<T>(val: T) -> T
///
/// If called, resolves exported function named 'recursive' from the attached
/// module instance and then calls into it with the provided argument.
/// Note that this function is polymorphic over type T.
/// This function requires attached module instance.
const RECURSE_FUNC_INDEX: usize = 4;

/// trap_sub(a: i32, b: i32) -> i32
///
/// This function is the same as sub(a, b), but it will send a Host trap which pauses the interpreter execution.
const TRAP_SUB_FUNC_INDEX: usize = 5;

impl Externals for TestHost {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            SUB_FUNC_INDEX => {
                let a: i32 = args.nth(0);
                let b: i32 = args.nth(1);

                let result: RuntimeValue = (a - b).into();

                Ok(Some(result))
            }
            ERR_FUNC_INDEX => {
                let error_code: u32 = args.nth(0);
                let error = HostErrorWithCode { error_code };
                Err(TrapKind::Host(Box::new(error)).into())
            }
            INC_MEM_FUNC_INDEX => {
                let ptr: u32 = args.nth(0);

                let memory = self
                    .memory
                    .as_ref()
                    .expect("Function 'inc_mem' expects attached memory");
                let mut buf = [0u8; 1];
                memory.get_into(ptr, &mut buf).unwrap();
                buf[0] += 1;
                memory.set(ptr, &buf).unwrap();

                Ok(None)
            }
            GET_MEM_FUNC_INDEX => {
                let ptr: u32 = args.nth(0);

                let memory = self
                    .memory
                    .as_ref()
                    .expect("Function 'get_mem' expects attached memory");
                let mut buf = [0u8; 1];
                memory.get_into(ptr, &mut buf).unwrap();

                Ok(Some(RuntimeValue::I32(buf[0] as i32)))
            }
            RECURSE_FUNC_INDEX => {
                let val = args
                    .nth_value_checked(0)
                    .expect("Exactly one argument expected");

                let instance = self
                    .instance
                    .as_ref()
                    .expect("Function 'recurse' expects attached module instance")
                    .clone();
                let result = instance
                    .invoke_export("recursive", &[val], self)
                    .expect("Failed to call 'recursive'")
                    .expect("expected to be Some");

                if val.value_type() != result.value_type() {
                    return Err(
                        TrapKind::Host(Box::new(HostErrorWithCode { error_code: 123 })).into(),
                    );
                }
                Ok(Some(result))
            }
            TRAP_SUB_FUNC_INDEX => {
                let a: i32 = args.nth(0);
                let b: i32 = args.nth(1);

                let result: RuntimeValue = (a - b).into();
                self.trap_sub_result = Some(result);
                Err(TrapKind::Host(Box::new(HostErrorWithCode { error_code: 301 })).into())
            }
            _ => panic!("env doesn't provide function at index {}", index),
        }
    }
}

impl TestHost {
    fn check_signature(&self, index: usize, signature: &Signature) -> bool {
        if index == RECURSE_FUNC_INDEX {
            // This function requires special handling because it is polymorphic.
            if signature.params().len() != 1 {
                return false;
            }
            let param_type = signature.params()[0];
            return signature.return_type() == Some(param_type);
        }

        let (params, ret_ty): (&[ValueType], Option<ValueType>) = match index {
            SUB_FUNC_INDEX => (&[ValueType::I32, ValueType::I32], Some(ValueType::I32)),
            ERR_FUNC_INDEX => (&[ValueType::I32], None),
            INC_MEM_FUNC_INDEX => (&[ValueType::I32], None),
            GET_MEM_FUNC_INDEX => (&[ValueType::I32], Some(ValueType::I32)),
            TRAP_SUB_FUNC_INDEX => (&[ValueType::I32, ValueType::I32], Some(ValueType::I32)),
            _ => return false,
        };

        signature.params() == params && signature.return_type() == ret_ty
    }
}

impl ModuleImportResolver for TestHost {
    fn resolve_func(&self, field_name: &str, signature: &Signature) -> Result<FuncRef, Error> {
        let index = match field_name {
            "sub" => SUB_FUNC_INDEX,
            "err" => ERR_FUNC_INDEX,
            "inc_mem" => INC_MEM_FUNC_INDEX,
            "get_mem" => GET_MEM_FUNC_INDEX,
            "recurse" => RECURSE_FUNC_INDEX,
            "trap_sub" => TRAP_SUB_FUNC_INDEX,
            _ => {
                return Err(Error::Instantiation(format!(
                    "Export {} not found",
                    field_name
                )));
            }
        };

        if !self.check_signature(index, signature) {
            return Err(Error::Instantiation(format!(
                "Export `{}` doesnt match expected type {:?}",
                field_name, signature
            )));
        }

        Ok(FuncInstance::alloc_host(signature.clone(), index))
    }

    fn resolve_memory(
        &self,
        field_name: &str,
        _memory_type: &MemoryDescriptor,
    ) -> Result<MemoryRef, Error> {
        Err(Error::Instantiation(format!(
            "Export {} not found",
            field_name
        )))
    }
}

#[test]
fn call_host_func() {
    let module = parse_wat(
        r#"
(module
	(import "env" "sub" (func $sub (param i32 i32) (result i32)))

	(func (export "test") (result i32)
		(call $sub
			(i32.const 5)
			(i32.const 7)
		)
	)
)
"#,
    );

    let mut env = TestHost::new();

    let instance = ModuleInstance::new(&module, &ImportsBuilder::new().with_resolver("env", &env))
        .expect("Failed to instantiate module")
        .assert_no_start();

    assert_eq!(
        instance
            .invoke_export("test", &[], &mut env)
            .expect("Failed to invoke 'test' function",),
        Some(RuntimeValue::I32(-2))
    );
}

#[test]
fn resume_call_host_func() {
    let module = parse_wat(
        r#"
(module
	(import "env" "trap_sub" (func $trap_sub (param i32 i32) (result i32)))

	(func (export "test") (result i32)
		(call $trap_sub
			(i32.const 5)
			(i32.const 7)
		)
	)
)
"#,
    );

    let mut env = TestHost::new();

    let instance = ModuleInstance::new(&module, &ImportsBuilder::new().with_resolver("env", &env))
        .expect("Failed to instantiate module")
        .assert_no_start();

    let export = instance.export_by_name("test").unwrap();
    let func_instance = export.as_func().unwrap();

    let mut invocation = FuncInstance::invoke_resumable(&func_instance, &[][..]).unwrap();
    let result = invocation.start_execution(&mut env);
    match result {
        Err(ResumableError::Trap(_)) => {}
        _ => panic!(),
    }

    assert!(invocation.is_resumable());
    let trap_sub_result = env.trap_sub_result.take();
    assert_eq!(
        invocation
            .resume_execution(trap_sub_result, &mut env)
            .expect("Failed to invoke 'test' function",),
        Some(RuntimeValue::I32(-2))
    );
}

#[test]
fn resume_call_host_func_type_mismatch() {
    fn resume_with_val(val: Option<RuntimeValue>) {
        let module = parse_wat(
            r#"
            (module
                (import "env" "trap_sub" (func $trap_sub (param i32 i32) (result i32)))

                (func (export "test") (result i32)
                    (call $trap_sub
                        (i32.const 5)
                        (i32.const 7)
                    )
                )
            )
            "#,
        );

        let mut env = TestHost::new();

        let instance =
            ModuleInstance::new(&module, &ImportsBuilder::new().with_resolver("env", &env))
                .expect("Failed to instantiate module")
                .assert_no_start();

        let export = instance.export_by_name("test").unwrap();
        let func_instance = export.as_func().unwrap();

        let mut invocation = FuncInstance::invoke_resumable(&func_instance, &[][..]).unwrap();
        let result = invocation.start_execution(&mut env);
        match result {
            Err(ResumableError::Trap(_)) => {}
            _ => panic!(),
        }

        assert!(invocation.is_resumable());
        let err = invocation.resume_execution(val, &mut env).unwrap_err();

        if let ResumableError::Trap(trap) = &err {
            if let TrapKind::UnexpectedSignature = trap.kind() {
                return;
            }
        }

        // If didn't return in the previous `match`...

        panic!(
            "Expected `ResumableError::Trap(Trap {{ kind: \
             TrapKind::UnexpectedSignature, }})`, got `{:?}`",
            err
        )
    }

    resume_with_val(None);
    resume_with_val(Some((-1i64).into()));
}

#[test]
fn host_err() {
    let module = parse_wat(
        r#"
(module
	(import "env" "err" (func $err (param i32)))

	(func (export "test")
		(call $err
			(i32.const 228)
		)
	)
)
"#,
    );

    let mut env = TestHost::new();

    let instance = ModuleInstance::new(&module, &ImportsBuilder::new().with_resolver("env", &env))
        .expect("Failed to instantiate module")
        .assert_no_start();

    let error = instance
        .invoke_export("test", &[], &mut env)
        .expect_err("`test` expected to return error");

    let error_with_code = error
        .as_host_error()
        .expect("Expected host error")
        .downcast_ref::<HostErrorWithCode>()
        .expect("Failed to downcast to expected error type");
    assert_eq!(error_with_code.error_code, 228);
}

#[test]
fn modify_mem_with_host_funcs() {
    let module = parse_wat(
        r#"
(module
	(import "env" "inc_mem" (func $inc_mem (param i32)))
	;; (import "env" "get_mem" (func $get_mem (param i32) (result i32)))

	(func (export "modify_mem")
		;; inc memory at address 12 for 4 times.
		(call $inc_mem (i32.const 12))
		(call $inc_mem (i32.const 12))
		(call $inc_mem (i32.const 12))
		(call $inc_mem (i32.const 12))
	)
)
"#,
    );

    let mut env = TestHost::new();

    let instance = ModuleInstance::new(&module, &ImportsBuilder::new().with_resolver("env", &env))
        .expect("Failed to instantiate module")
        .assert_no_start();

    instance
        .invoke_export("modify_mem", &[], &mut env)
        .expect("Failed to invoke 'test' function");

    // Check contents of memory at address 12.
    let mut buf = [0u8; 1];
    env.memory.unwrap().get_into(12, &mut buf).unwrap();

    assert_eq!(&buf, &[4]);
}

#[test]
fn pull_internal_mem_from_module() {
    let module = parse_wat(
        r#"
(module
	(import "env" "inc_mem" (func $inc_mem (param i32)))
	(import "env" "get_mem" (func $get_mem (param i32) (result i32)))

	;; declare internal memory and export it under name "mem"
	(memory (export "mem") 1 1)

	(func (export "test") (result i32)
		;; Increment value at address 1337
		(call $inc_mem (i32.const 1337))

		;; Return value at address 1337
		(call $get_mem (i32.const 1337))
	)
)
"#,
    );

    let mut env = TestHost {
        memory: None,
        instance: None,

        trap_sub_result: None,
    };

    let instance = ModuleInstance::new(&module, &ImportsBuilder::new().with_resolver("env", &env))
        .expect("Failed to instantiate module")
        .assert_no_start();

    // Get memory instance exported by name 'mem' from the module instance.
    let internal_mem = instance
        .export_by_name("mem")
        .expect("Module expected to have 'mem' export")
        .as_memory()
        .cloned()
        .expect("'mem' export should be a memory");

    env.memory = Some(internal_mem);

    assert_eq!(
        instance.invoke_export("test", &[], &mut env).unwrap(),
        Some(RuntimeValue::I32(1))
    );
}

#[test]
fn recursion() {
    let module = parse_wat(
        r#"
(module
	;; Import 'recurse' function. Upon a call it will call back inside
	;; this module, namely to function 'recursive' defined below.
	(import "env" "recurse" (func $recurse (param i64) (result i64)))

	;; Note that we import same function but with different type signature
	;; this is possible since 'recurse' is a host function and it is defined
	;; to be polymorphic.
	(import "env" "recurse" (func (param f32) (result f32)))

	(func (export "recursive") (param i64) (result i64)
		;; return arg_0 + 42;
		(i64.add
			(get_local 0)
			(i64.const 42)
		)
	)

	(func (export "test") (result i64)
		(call $recurse (i64.const 321))
	)
)
"#,
    );

    let mut env = TestHost::new();

    let instance = ModuleInstance::new(&module, &ImportsBuilder::new().with_resolver("env", &env))
        .expect("Failed to instantiate module")
        .assert_no_start();

    // Put instance into the env, because $recurse function expects
    // attached module instance.
    env.instance = Some(instance.clone());

    assert_eq!(
        instance
            .invoke_export("test", &[], &mut env)
            .expect("Failed to invoke 'test' function",),
        // 363 = 321 + 42
        Some(RuntimeValue::I64(363))
    );
}

#[test]
fn defer_providing_externals() {
    const INC_FUNC_INDEX: usize = 0;

    /// `HostImportResolver` will be passed at instantiation time.
    ///
    /// Main purpose of this struct is to statsify imports of
    /// the module being instantiated.
    struct HostImportResolver {
        mem: MemoryRef,
    }

    impl ModuleImportResolver for HostImportResolver {
        fn resolve_func(&self, field_name: &str, signature: &Signature) -> Result<FuncRef, Error> {
            if field_name != "inc" {
                return Err(Error::Instantiation(format!(
                    "Export {} not found",
                    field_name
                )));
            }
            if signature.params() != [ValueType::I32] || signature.return_type() != None {
                return Err(Error::Instantiation(format!(
                    "Export `{}` doesnt match expected type {:?}",
                    field_name, signature
                )));
            }

            Ok(FuncInstance::alloc_host(signature.clone(), INC_FUNC_INDEX))
        }

        fn resolve_memory(
            &self,
            field_name: &str,
            _memory_type: &MemoryDescriptor,
        ) -> Result<MemoryRef, Error> {
            if field_name == "mem" {
                Ok(self.mem.clone())
            } else {
                Err(Error::Instantiation(format!(
                    "Export {} not found",
                    field_name
                )))
            }
        }
    }

    /// This struct implements external functions that can be called
    /// by wasm module.
    struct HostExternals<'a> {
        acc: &'a mut u32,
    }

    impl<'a> Externals for HostExternals<'a> {
        fn invoke_index(
            &mut self,
            index: usize,
            args: RuntimeArgs,
        ) -> Result<Option<RuntimeValue>, Trap> {
            match index {
                INC_FUNC_INDEX => {
                    let a = args.nth::<u32>(0);
                    *self.acc += a;
                    Ok(None)
                }
                _ => panic!("env module doesn't provide function at index {}", index),
            }
        }
    }

    let module = parse_wat(
        r#"
(module
	;; Just to require 'mem' from 'host'.
	(import "host" "mem" (memory 1))
	(import "host" "inc" (func $inc (param i32)))

	(func (export "test")
		(call $inc (i32.const 1))
	)
)
"#,
    );

    // Create HostImportResolver with some initialized memory instance.
    // This memory instance will be provided as 'mem' export.
    let host_import_resolver = HostImportResolver {
        mem: MemoryInstance::alloc(Pages(1), Some(Pages(1))).unwrap(),
    };

    // Instantiate module with `host_import_resolver` as import resolver for "host" module.
    let instance = ModuleInstance::new(
        &module,
        &ImportsBuilder::new().with_resolver("host", &host_import_resolver),
    )
    .expect("Failed to instantiate module")
    .assert_no_start();

    let mut acc = 89;
    {
        let mut host_externals = HostExternals { acc: &mut acc };

        instance
            .invoke_export("test", &[], &mut host_externals)
            .unwrap(); // acc += 1;
        instance
            .invoke_export("test", &[], &mut host_externals)
            .unwrap(); // acc += 1;
    }
    assert_eq!(acc, 91);
}

#[test]
fn two_envs_one_externals() {
    const PRIVILEGED_FUNC_INDEX: usize = 0;
    const ORDINARY_FUNC_INDEX: usize = 1;

    struct HostExternals;

    impl Externals for HostExternals {
        fn invoke_index(
            &mut self,
            index: usize,
            _args: RuntimeArgs,
        ) -> Result<Option<RuntimeValue>, Trap> {
            match index {
                PRIVILEGED_FUNC_INDEX => {
                    println!("privileged!");
                    Ok(None)
                }
                ORDINARY_FUNC_INDEX => Ok(None),
                _ => panic!("env module doesn't provide function at index {}", index),
            }
        }
    }

    struct PrivilegedResolver;
    struct OrdinaryResolver;

    impl ModuleImportResolver for PrivilegedResolver {
        fn resolve_func(&self, field_name: &str, signature: &Signature) -> Result<FuncRef, Error> {
            let index = match field_name {
                "ordinary" => ORDINARY_FUNC_INDEX,
                "privileged" => PRIVILEGED_FUNC_INDEX,
                _ => {
                    return Err(Error::Instantiation(format!(
                        "Export {} not found",
                        field_name
                    )));
                }
            };

            Ok(FuncInstance::alloc_host(signature.clone(), index))
        }
    }

    impl ModuleImportResolver for OrdinaryResolver {
        fn resolve_func(&self, field_name: &str, signature: &Signature) -> Result<FuncRef, Error> {
            let index = match field_name {
                "ordinary" => ORDINARY_FUNC_INDEX,
                "privileged" => {
                    return Err(Error::Instantiation(
                        "'priveleged' can be imported only in privileged context".into(),
                    ));
                }
                _ => {
                    return Err(Error::Instantiation(format!(
                        "Export {} not found",
                        field_name
                    )));
                }
            };

            Ok(FuncInstance::alloc_host(signature.clone(), index))
        }
    }

    let trusted_module = parse_wat(
        r#"
(module
	;; Trusted module can import both ordinary and privileged functions.
	(import "env" "ordinary" (func $ordinary))
	(import "env" "privileged" (func $privileged))
	(func (export "do_trusted_things")
		(call $ordinary)
		(call $privileged)
	)
)
"#,
    );

    let untrusted_module = parse_wat(
        r#"
(module
	;; Untrusted module can import only ordinary functions.
	(import "env" "ordinary" (func $ordinary))
	(import "trusted" "do_trusted_things" (func $do_trusted_things))
	(func (export "test")
		(call $ordinary)
		(call $do_trusted_things)
	)
)
"#,
    );

    let trusted_instance = ModuleInstance::new(
        &trusted_module,
        &ImportsBuilder::new().with_resolver("env", &PrivilegedResolver),
    )
    .expect("Failed to instantiate module")
    .assert_no_start();

    let untrusted_instance = ModuleInstance::new(
        &untrusted_module,
        &ImportsBuilder::new()
            .with_resolver("env", &OrdinaryResolver)
            .with_resolver("trusted", &trusted_instance),
    )
    .expect("Failed to instantiate module")
    .assert_no_start();

    untrusted_instance
        .invoke_export("test", &[], &mut HostExternals)
        .expect("Failed to invoke 'test' function");
}

#[test]
fn dynamically_add_host_func() {
    const ADD_FUNC_FUNC_INDEX: usize = 0;

    struct HostExternals {
        table: TableRef,
        added_funcs: u32,
    }

    impl Externals for HostExternals {
        fn invoke_index(
            &mut self,
            index: usize,
            _args: RuntimeArgs,
        ) -> Result<Option<RuntimeValue>, Trap> {
            match index {
                ADD_FUNC_FUNC_INDEX => {
                    // Allocate indicies for the new function.
                    // host_func_index is in host index space, and first index is occupied by ADD_FUNC_FUNC_INDEX.
                    let table_index = self.added_funcs;
                    let host_func_index = table_index + 1;
                    self.added_funcs += 1;

                    let added_func = FuncInstance::alloc_host(
                        Signature::new(&[][..], Some(ValueType::I32)),
                        host_func_index as usize,
                    );
                    self.table
                        .set(table_index, Some(added_func))
                        .map_err(|_| TrapKind::TableAccessOutOfBounds)?;

                    Ok(Some(RuntimeValue::I32(table_index as i32)))
                }
                index if index as u32 <= self.added_funcs => {
                    Ok(Some(RuntimeValue::I32(index as i32)))
                }
                _ => panic!("'env' module doesn't provide function at index {}", index),
            }
        }
    }

    impl ModuleImportResolver for HostExternals {
        fn resolve_func(&self, field_name: &str, signature: &Signature) -> Result<FuncRef, Error> {
            let index = match field_name {
                "add_func" => ADD_FUNC_FUNC_INDEX,
                _ => {
                    return Err(Error::Instantiation(format!(
                        "Export {} not found",
                        field_name
                    )));
                }
            };
            Ok(FuncInstance::alloc_host(signature.clone(), index))
        }

        fn resolve_table(
            &self,
            field_name: &str,
            _table_type: &TableDescriptor,
        ) -> Result<TableRef, Error> {
            if field_name == "table" {
                Ok(self.table.clone())
            } else {
                Err(Error::Instantiation(format!(
                    "Export {} not found",
                    field_name
                )))
            }
        }
    }

    let mut host_externals = HostExternals {
        table: TableInstance::alloc(10, None).unwrap(),
        added_funcs: 0,
    };

    let module = parse_wat(
        r#"
(module
	(type $t0 (func (result i32)))
	(import "env" "add_func" (func $add_func (result i32)))
	(import "env" "table" (table 10 anyfunc))
	(func (export "test") (result i32)
		;; Call add_func but discard the result
		call $add_func
		drop

		;; Call add_func and then make an indirect call with the returned index
		call $add_func
		call_indirect (type $t0)
	)
)
"#,
    );

    let instance = ModuleInstance::new(
        &module,
        &ImportsBuilder::new().with_resolver("env", &host_externals),
    )
    .expect("Failed to instantiate module")
    .assert_no_start();

    assert_eq!(
        instance
            .invoke_export("test", &[], &mut host_externals)
            .expect("Failed to invoke 'test' function"),
        Some(RuntimeValue::I32(2))
    );
}
