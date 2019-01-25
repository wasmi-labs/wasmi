// If you are to update this code, make sure you update the example in `wasmi-derive`.

extern crate wasmi;
extern crate wasmi_derive;
#[macro_use]
extern crate assert_matches;

use std::fmt;
use wasmi::HostError;
use wasmi_derive::derive_externals;

#[derive(Debug)]
struct NoInfoError;
impl HostError for NoInfoError {}
impl fmt::Display for NoInfoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NoInfoError")
    }
}

struct NonStaticExternals<'a> {
    state: &'a mut usize,
}

#[derive_externals]
impl<'a> NonStaticExternals<'a> {
    pub fn add(&self, a: u32, b: u32) -> u32 {
        a + b
    }

    pub fn increment(&mut self) {
        *self.state += 1;
    }

    pub fn traps(&self) -> Result<(), NoInfoError> {
        Err(NoInfoError)
    }
}

mod tests {
    extern crate wabt;

    use super::*;
    use wasmi::{ImportsBuilder, Module, ModuleInstance, RuntimeValue};

    macro_rules! gen_test {
        ($test_name:ident, $wat:expr, |$instance:ident| $verify:expr) => {
            #[test]
            fn $test_name() {
                // We don't test wat compiliation, loading or decoding.
                let wasm = &wabt::wat2wasm($wat).expect("invalid wat");
                let module = Module::from_buffer(&wasm).expect("can't load module");

                let resolver = NonStaticExternals::resolver();

                let mut imports = ImportsBuilder::new();
                imports.push_resolver("env", &resolver);
                let $instance = ModuleInstance::new(&module, &imports);

                $verify
            }
        };
    }

    gen_test! { it_works,
        r#"
        (module
            (import "env" "add" (func $add (param i32 i32) (result i32)))
            (import "env" "increment" (func $increment))
            (import "env" "traps" (func $traps))

            (export "add" (func $add))
            (export "increment" (func $increment))
            (export "traps" (func $traps))
        )
        "#,
        |not_started_instance_result| {
            let mut state = 0;
            let mut externals = NonStaticExternals {
                state: &mut state,
            };

            let instance = not_started_instance_result.unwrap().assert_no_start();
            assert_matches!(
                instance.invoke_export(
                    "traps",
                    &[],
                    &mut externals,
                ),
                Err(_)
            );

            assert_eq!(*externals.state, 0);
            assert_matches!(
                instance.invoke_export(
                    "increment",
                    &[],
                    &mut externals,
                ),
                Ok(None)
            );
            assert_eq!(*externals.state, 1);

            assert_matches!(
                instance.invoke_export(
                    "add",
                    &[RuntimeValue::I32(5), RuntimeValue::I32(2)],
                    &mut externals,
                ),
                Ok(Some(RuntimeValue::I32(7)))
            );
        }
    }

    gen_test! { wrong_signature,
        r#"
        (module
            (import "env" "add" (func $add (param i64 i32) (result i32)))
        )
        "#,
        |result| {
            match result {
                Ok(_) => panic!(),
                Err(err) => {
                    assert_eq!(
                        &format!("{:?}", err),
                        r#"Instantiation("Export add has different signature Signature { params: [I64, I32], return_type: Some(I32) }")"#,
                    );
                }
            }
        }
    }

    gen_test! { nonexistent_name,
        r#"
        (module
            (import "env" "foo" (func $foo))
        )
        "#,
        |result| {
            match result {
                Ok(_) => panic!(),
                Err(err) => {
                    assert_eq!(
                        &format!("{:?}", err),
                        r#"Instantiation("Export foo not found")"#,
                    );
                }
            }
        }
    }
}
