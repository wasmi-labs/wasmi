use super::*;
use crate::v2::{
    engine::{
        bytecode::{Instruction, LocalIdx},
        DropKeep,
        InstructionIdx,
        Target,
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

#[test]
fn drop_locals() {
    let wasm = wat2wasm(
        r#"
		(module
			(func (export "call") (param i32)
				(local i32)
				get_local 0
				set_local 1
			)
		)
	"#,
    );
    assert_single_func_body(
        &wasm,
        &[
            Instruction::GetLocal(LocalIdx::from(2)),
            Instruction::SetLocal(LocalIdx::from(1)),
            Instruction::Return(DropKeep::new(2, 0)),
        ],
    );
}

#[test]
fn if_without_else() {
    let wasm = wat2wasm(
        r#"
		(module
			(func (export "call") (param i32) (result i32)
				i32.const 1
				if
					i32.const 2
					return
				end
				i32.const 3
			)
		)
	"#,
    );
    assert_single_func_body(
        &wasm,
        &[
            // 0
            Instruction::I32Const(1),
            // 1
            Instruction::BrIfEqz(Target::new(
                InstructionIdx::from_usize(4),
                DropKeep::new(0, 0),
            )),
            // 2
            Instruction::I32Const(2),
            // 3
            Instruction::Return(DropKeep::new(1, 1)),
            // 4
            Instruction::I32Const(3),
            // 5
            Instruction::Return(DropKeep::new(1, 1)),
        ],
    );
}

#[test]
fn if_else() {
    let wasm = wat2wasm(
        r#"
		(module
			(func (export "call")
				(local i32)
				i32.const 1
				if
					i32.const 2
					set_local 0
				else
					i32.const 3
					set_local 0
				end
			)
		)
	"#,
    );
    assert_single_func_body(
        &wasm,
        &[
            Instruction::I32Const(1),
            Instruction::BrIfEqz(Target::new(
                InstructionIdx::from_usize(5),
                DropKeep::new(0, 0),
            )),
            Instruction::I32Const(2),
            Instruction::SetLocal(LocalIdx::from(1)),
            Instruction::Br(Target::new(
                InstructionIdx::from_usize(7),
                DropKeep::new(0, 0),
            )),
            Instruction::I32Const(3),
            Instruction::SetLocal(LocalIdx::from(1)),
            Instruction::Return(DropKeep::new(1, 0)),
        ],
    );
}

#[test]
fn if_else_returns_result() {
    let wasm = wat2wasm(
        r#"
		(module
			(func (export "call")
				i32.const 1
				if (result i32)
					i32.const 2
				else
					i32.const 3
				end
				drop
			)
		)
	"#,
    );
    assert_single_func_body(
        &wasm,
        &[
            Instruction::I32Const(1),
            Instruction::BrIfEqz(Target::new(
                InstructionIdx::from_usize(4),
                DropKeep::new(0, 0),
            )),
            Instruction::I32Const(2),
            Instruction::Br(Target::new(
                InstructionIdx::from_usize(5),
                DropKeep::new(0, 0),
            )),
            Instruction::I32Const(3),
            Instruction::Drop,
            Instruction::Return(DropKeep::new(0, 0)),
        ],
    );
}

#[test]
fn if_else_branch_from_true_branch() {
    let wasm = wat2wasm(
        r#"
		(module
			(func (export "call")
				i32.const 1
				if (result i32)
					i32.const 1
					i32.const 1
					br_if 0
					drop
					i32.const 2
				else
					i32.const 3
				end
				drop
			)
		)
	"#,
    );
    assert_single_func_body(
        &wasm,
        &[
            Instruction::I32Const(1),
            Instruction::BrIfEqz(Target::new(
                InstructionIdx::from_usize(8),
                DropKeep::new(0, 0),
            )),
            Instruction::I32Const(1),
            Instruction::I32Const(1),
            Instruction::BrIfNez(Target::new(
                InstructionIdx::from_usize(9),
                DropKeep::new(0, 1),
            )),
            Instruction::Drop,
            Instruction::I32Const(2),
            Instruction::Br(Target::new(
                InstructionIdx::from_usize(9),
                DropKeep::new(0, 0),
            )),
            Instruction::I32Const(3),
            Instruction::Drop,
            Instruction::Return(DropKeep::new(0, 0)),
        ],
    );
}

#[test]
fn if_else_branch_from_false_branch() {
    let wasm = wat2wasm(
        r#"
		(module
			(func (export "call")
				i32.const 1
				if (result i32)
					i32.const 1
				else
					i32.const 2
					i32.const 1
					br_if 0
					drop
					i32.const 3
				end
				drop
			)
		)
	"#,
    );
    assert_single_func_body(
        &wasm,
        &[
            Instruction::I32Const(1),
            Instruction::BrIfEqz(Target::new(
                InstructionIdx::from_usize(4),
                DropKeep::new(0, 0),
            )),
            Instruction::I32Const(1),
            Instruction::Br(Target::new(
                InstructionIdx::from_usize(9),
                DropKeep::new(0, 0),
            )),
            Instruction::I32Const(2),
            Instruction::I32Const(1),
            Instruction::BrIfNez(Target::new(
                InstructionIdx::from_usize(9),
                DropKeep::new(0, 1),
            )),
            Instruction::Drop,
            Instruction::I32Const(3),
            Instruction::Drop,
            Instruction::Return(DropKeep::new(0, 0)),
        ],
    );
}

#[test]
fn loop_() {
    let wasm = wat2wasm(
        r#"
		(module
			(func (export "call")
				loop (result i32)
					i32.const 1
					br_if 0
					i32.const 2
				end
				drop
			)
		)
	"#,
    );
    assert_single_func_body(
        &wasm,
        &[
            Instruction::I32Const(1),
            Instruction::BrIfNez(Target::new(
                InstructionIdx::from_usize(0),
                DropKeep::new(0, 0),
            )),
            Instruction::I32Const(2),
            Instruction::Drop,
            Instruction::Return(DropKeep::new(0, 0)),
        ],
    );
}

#[test]
fn loop_empty() {
    let wasm = wat2wasm(
        r#"
		(module
			(func (export "call")
				loop
				end
			)
		)
	"#,
    );
    assert_single_func_body(&wasm, &[Instruction::Return(DropKeep::new(0, 0))]);
}
