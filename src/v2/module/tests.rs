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

fn assert_single_func_body(wasm: &[u8], expected: &[Instruction]) {
    let (engine, func_bodies) = compile(&wasm);
    assert_eq!(func_bodies.len(), 1);
    assert_func_body(&engine, func_bodies[0], expected);
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
    assert_single_func_body(&wasm, &[Instruction::Return(DropKeep::new(0, 0))]);
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
    assert_single_func_body(
        &wasm,
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
    assert_single_func_body(&wasm, &[Instruction::Return(DropKeep::new(1, 0))]);
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
    assert_single_func_body(
        &wasm,
        &[
            Instruction::GetLocal(LocalIdx::from(1)),
            Instruction::Return(DropKeep::new(1, 1)),
        ],
    );
}

#[test]
fn get_local_2() {
    let wasm = wat2wasm(
        r#"
		(module
			(func (export "call") (param i32) (param i32) (result i32)
				get_local 0
                get_local 1
                drop
			)
		)
	"#,
    );
    assert_single_func_body(
        &wasm,
        &[
            Instruction::GetLocal(LocalIdx::from(2)),
            Instruction::GetLocal(LocalIdx::from(2)),
            Instruction::Drop,
            Instruction::Return(DropKeep::new(2, 1)),
        ],
    );
}

#[test]
fn explicit_return() {
    let wasm = wat2wasm(
        r#"
		(module
			(func (export "call") (param i32) (result i32)
				get_local 0
				return
			)
		)
	"#,
    );
    assert_single_func_body(
        &wasm,
        &[
            Instruction::GetLocal(LocalIdx::from(1)),
            Instruction::Return(DropKeep::new(1, 1)),
            Instruction::Return(DropKeep::new(1, 1)),
        ],
    );
}

#[test]
fn add_params() {
    let wasm = wat2wasm(
        r#"
		(module
			(func (export "call") (param i32) (param i32) (result i32)
				get_local 0
				get_local 1
				i32.add
			)
		)
	"#,
    );
    assert_single_func_body(
        &wasm,
        &[
            // This is tricky. Locals are now loaded from the stack. The load
            // happens from address relative of the current stack pointer. The first load
            // takes the value below the previous one (i.e the second argument) and then, it increments
            // the stack pointer. And then the same thing hapens with the value below the previous one
            // (which happens to be the value loaded by the first get_local).
            Instruction::GetLocal(LocalIdx::from(2)),
            Instruction::GetLocal(LocalIdx::from(2)),
            Instruction::I32Add,
            Instruction::Return(DropKeep::new(2, 1)),
        ],
    );
}
