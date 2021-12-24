use super::*;
use crate::v2::{
    engine::{
        bytecode::{Instruction, LocalIdx},
        DropKeep,
    },
    Engine,
};

fn wat2wasm(wat: &str) -> Vec<u8> {
    wabt::wat2wasm(wat).unwrap()
}

fn compile(bytes: impl AsRef<[u8]>) -> (Engine, Vec<FuncBody>) {
    let engine = Engine::default();
    let module = Module::new(&engine, bytes).unwrap();
    (engine, module.func_bodies)
}

fn assert_func_body(engine: &Engine, func_body: FuncBody, expected: &[Instruction]) {
    for (index, actual, expected) in expected
        .iter()
        .enumerate()
        .map(|(index, expected)| (index, engine.resolve_inst(func_body, index), *expected))
    {
        assert_eq!(
            actual, expected,
            "encountered instruction mismatch at position {}",
            index
        );
    }
}

#[test]
fn implicit_return_no_value() {
    let wasm = wat2wasm(
        r#"
		(module
			(func (export "call")
			)
		)
	"#,
    );
    let (engine, func_bodies) = compile(&wasm);
    assert_eq!(func_bodies.len(), 1);
    assert_func_body(
        &engine,
        func_bodies[0],
        &[Instruction::Return(DropKeep::new(0, 0))],
    );
}

#[test]
fn implicit_return_with_value() {
    let wasm = wat2wasm(
        r#"
		(module
			(func (export "call") (result i32)
				i32.const 0
			)
		)
	"#,
    );
    let (engine, func_bodies) = compile(&wasm);
    assert_eq!(func_bodies.len(), 1);
    assert_func_body(
        &engine,
        func_bodies[0],
        &[
            Instruction::I32Const(0),
            Instruction::Return(DropKeep::new(0, 1)),
        ],
    );
}

#[test]
fn implicit_return_param() {
    let wasm = wat2wasm(
        r#"
		(module
			(func (export "call") (param i32)
			)
		)
	"#,
    );
    let (engine, func_bodies) = compile(&wasm);
    assert_eq!(func_bodies.len(), 1);
    assert_func_body(
        &engine,
        func_bodies[0],
        &[Instruction::Return(DropKeep::new(1, 0))],
    );
}

#[test]
fn get_local() {
    let wasm = wat2wasm(
        r#"
		(module
			(func (export "call") (param i32) (result i32)
				get_local 0
			)
		)
	"#,
    );
    let (engine, func_bodies) = compile(&wasm);
    assert_eq!(func_bodies.len(), 1);
    assert_func_body(
        &engine,
        func_bodies[0],
        &[
            Instruction::GetLocal(LocalIdx::from(1)),
            Instruction::Return(DropKeep::new(1, 1)),
        ],
    );
}
