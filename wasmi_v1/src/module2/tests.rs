use super::*;
use crate::{
    engine::{
        bytecode::{Instruction, LocalIdx},
        DropKeep,
        InstructionIdx,
        Target,
    },
    Engine,
};

/// Converts the `wat` string source into `wasm` encoded byte.
fn wat2wasm(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).unwrap()
}

/// Compiles the `wasm` encoded bytes into a [`Module`].
///
/// # Panics
///
/// If an error occurred upon module compilation, validation or translation.
fn create_module(bytes: &[u8]) -> Module {
    let engine = Engine::default();
    Module::new(&engine, bytes).unwrap()
}

/// Asserts that the given `func_body` consists of the expected instructions.
///
/// # Panics
///
/// If there is an instruction mismatch between the actual instructions in
/// `func_body` and the `expected_instructions`.
fn assert_func_body<E>(
    engine: &Engine,
    func_type: DedupFuncType,
    func_body: FuncBody,
    expected_instructions: E,
) where
    E: IntoIterator<Item = Instruction>,
{
    for (index, actual, expected) in expected_instructions
        .into_iter()
        .enumerate()
        .map(|(index, expected)| (index, engine.resolve_inst(func_body, index), expected))
    {
        assert_eq!(
            actual,
            expected,
            "encountered instruction mismatch for {} at position {}",
            engine.resolve_func_type(func_type, Clone::clone),
            index
        );
    }
}

/// Asserts that the given `wasm` bytes yield functions with expected instructions.
///
/// # Panics
///
/// If any of the yielded functions consists of instruction different from the
/// expected instructions for that function.
fn assert_func_bodies<E, T>(wasm_bytes: impl AsRef<[u8]>, expected: E)
where
    E: IntoIterator<Item = T>,
    T: IntoIterator<Item = Instruction>,
{
    let wasm_bytes = wasm_bytes.as_ref();
    let module = create_module(wasm_bytes);
    let engine = module.engine();
    for ((func_type, func_body), expected) in module.internal_funcs().zip(expected) {
        assert_func_body(engine, func_type, func_body, expected);
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
    let expected = [Instruction::Return(DropKeep::new(0, 0))];
    assert_func_bodies(&wasm, [expected]);
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
    let expected = [
        Instruction::constant(0),
        Instruction::Return(DropKeep::new(0, 1)),
    ];
    assert_func_bodies(&wasm, [expected]);
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
    let expected = [Instruction::Return(DropKeep::new(1, 0))];
    assert_func_bodies(&wasm, [expected]);
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
    let expected = [
        Instruction::GetLocal {
            local_depth: LocalIdx::from(1),
        },
        Instruction::Return(DropKeep::new(1, 1)),
    ];
    assert_func_bodies(&wasm, [expected]);
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
    let expected = [
        Instruction::GetLocal {
            local_depth: LocalIdx::from(2),
        },
        Instruction::GetLocal {
            local_depth: LocalIdx::from(2),
        },
        Instruction::Drop,
        Instruction::Return(DropKeep::new(2, 1)),
    ];
    assert_func_bodies(&wasm, [expected]);
}

#[test]
fn get_local_3() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (param i32)
                local.get 0
                local.get 1
                drop
                drop
            )
        )
    "#,
    );
    let expected = [
        Instruction::GetLocal {
            local_depth: LocalIdx::from(2),
        },
        Instruction::GetLocal {
            local_depth: LocalIdx::from(2),
        },
        Instruction::Drop,
        Instruction::Drop,
        Instruction::Return(DropKeep::new(2, 0)),
    ];
    assert_func_bodies(&wasm, [expected]);
}
