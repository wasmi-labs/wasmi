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
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let expected_instructions = expected_instructions.into_iter();
    let len_expected = expected_instructions.len();
    for (index, actual, expected) in
        expected_instructions
            .into_iter()
            .enumerate()
            .map(|(index, expected)| {
                (
                    index,
                    engine.resolve_inst(func_body, index).unwrap_or_else(|| {
                        panic!("encountered missing instruction at position {}", index)
                    }),
                    expected,
                )
            })
    {
        assert_eq!(
            actual,
            expected,
            "encountered instruction mismatch for {} at position {}",
            engine.resolve_func_type(func_type, Clone::clone),
            index
        );
    }
    if let Some(unexpected) = engine.resolve_inst(func_body, len_expected) {
        panic!(
            "encountered unexpected instruction at position {}: {:?}",
            len_expected, unexpected,
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
    <T as IntoIterator>::IntoIter: ExactSizeIterator,
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
                local.get 0
            )
        )
    "#,
    );
    let expected = [
        Instruction::local_get(1),
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
                local.get 0
                local.get 1
                drop
            )
        )
    "#,
    );
    let expected = [
        Instruction::local_get(2),
        Instruction::local_get(2),
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
        Instruction::local_get(2),
        Instruction::local_get(2),
        Instruction::Drop,
        Instruction::Drop,
        Instruction::Return(DropKeep::new(2, 0)),
    ];
    assert_func_bodies(&wasm, [expected]);
}

#[test]
fn explicit_return() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (result i32)
                local.get 0
                return
            )
        )
    "#,
    );
    let expected = [
        Instruction::local_get(1),
        Instruction::Return(DropKeep::new(1, 1)),
    ];
    assert_func_bodies(&wasm, [expected]);
}

#[test]
fn simple_add() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (param i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
        )
    "#,
    );
    let expected = [
        Instruction::local_get(2),
        Instruction::local_get(2),
        Instruction::I32Add,
        Instruction::Return(DropKeep::new(2, 1)),
    ];
    assert_func_bodies(&wasm, [expected]);
}

#[test]
fn simple_mul_add() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (param i32) (result i32)
                local.get 0
                local.get 1
                local.get 0
                local.get 1
                i32.add
                i32.add
                i32.mul
            )
        )
    "#,
    );
    let expected = [
        Instruction::local_get(2),
        Instruction::local_get(2),
        Instruction::local_get(4),
        Instruction::local_get(4),
        Instruction::I32Add,
        Instruction::I32Add,
        Instruction::I32Mul,
        Instruction::Return(DropKeep::new(2, 1)),
    ];
    assert_func_bodies(&wasm, [expected]);
}

#[test]
fn drop_locals() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32)
                (local i32)
                local.get 0
                local.set 1
            )
        )
    "#,
    );
    let expected = [
        Instruction::local_get(2),
        Instruction::local_set(1),
        Instruction::Return(DropKeep::new(2, 0)),
    ];
    assert_func_bodies(&wasm, [expected]);
}

macro_rules! target {
    ( $inst_idx:expr, drop: $drop:expr, keep: $keep:expr ) => {
        Target::new(
            InstructionIdx::from_usize($inst_idx),
            DropKeep::new($drop, $keep),
        )
    };
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
    let expected = [
        /* 0 */ Instruction::constant(1),
        /* 1 */ Instruction::BrIfEqz(target!(4, drop: 0, keep: 0)),
        /* 2 */ Instruction::constant(2),
        /* 3 */ Instruction::Return(DropKeep::new(1, 1)),
        /* 4 */ Instruction::constant(3),
        /* 5 */ Instruction::Return(DropKeep::new(1, 1)),
    ];
    assert_func_bodies(&wasm, [expected]);
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
                    local.set 0
                else
                    i32.const 3
                    local.set 0
                end
            )
        )
    "#,
    );
    let expected = [
        Instruction::constant(1),
        Instruction::BrIfEqz(target!(5, drop: 0, keep: 0)),
        Instruction::constant(2),
        Instruction::local_set(1),
        Instruction::Br(target!(7, drop: 0, keep: 0)),
        Instruction::constant(3),
        Instruction::local_set(1),
        Instruction::Return(DropKeep::new(1, 0)),
    ];
    assert_func_bodies(&wasm, [expected]);
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
    let expected = [
        /* 0 */ Instruction::constant(1),
        /* 1 */ Instruction::BrIfEqz(target!(4, drop: 0, keep: 0)),
        /* 2 */ Instruction::constant(2),
        /* 3 */ Instruction::Br(target!(5, drop: 0, keep: 0)),
        /* 4 */ Instruction::constant(3),
        /* 5 */ Instruction::Drop,
        /* 6 */ Instruction::Return(DropKeep::new(0, 0)),
    ];
    assert_func_bodies(&wasm, [expected]);
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
    let expected = [
        /*  0 */ Instruction::constant(1),
        /*  1 */ Instruction::BrIfEqz(target!(8, drop: 0, keep: 0)),
        /*  2 */ Instruction::constant(1),
        /*  3 */ Instruction::constant(1),
        /*  4 */ Instruction::BrIfNez(target!(9, drop: 0, keep: 1)),
        /*  5 */ Instruction::Drop,
        /*  6 */ Instruction::constant(2),
        /*  7 */ Instruction::Br(target!(9, drop: 0, keep: 0)),
        /*  8 */ Instruction::constant(3),
        /*  9 */ Instruction::Drop,
        /* 10 */ Instruction::Return(DropKeep::new(0, 0)),
    ];
    assert_func_bodies(&wasm, [expected]);
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
    let expected = [
        /*  0 */ Instruction::constant(1),
        /*  1 */ Instruction::BrIfEqz(target!(4, drop: 0, keep: 0)),
        /*  2 */ Instruction::constant(1),
        /*  3 */ Instruction::Br(target!(9, drop: 0, keep: 0)),
        /*  4 */ Instruction::constant(2),
        /*  5 */ Instruction::constant(1),
        /*  6 */ Instruction::BrIfNez(target!(9, drop: 0, keep: 1)),
        /*  7 */ Instruction::Drop,
        /*  8 */ Instruction::constant(3),
        /*  9 */ Instruction::Drop,
        /* 10 */ Instruction::Return(DropKeep::new(0, 0)),
    ];
    assert_func_bodies(&wasm, [expected]);
}

#[test]
fn if_else_both_unreachable_before_end() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (result i32)
                local.get 0
                if (result i32)
                    i32.const 1
                    return
                    i32.const 100 ;; unreachable
                else
                    i32.const 2
                    return
                    i32.const 200 ;; unreachable
                end
                drop
                i32.const 3
            )
        )
    "#,
    );
    let expected = [
        /* 0 */ Instruction::local_get(1),
        /* 1 */ Instruction::BrIfEqz(target!(4, drop: 0, keep: 0)),
        /* 2 */ Instruction::constant(1),
        /* 3 */ Instruction::Return(DropKeep::new(1, 1)),
        /* 4 */ Instruction::constant(2),
        /* 5 */ Instruction::Return(DropKeep::new(1, 1)),
        /* 6 */ Instruction::Drop,
        /* 7 */ Instruction::constant(3),
        /* 8 */ Instruction::Return(DropKeep::new(1, 1)),
    ];
    assert_func_bodies(&wasm, [expected]);
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
    let expected = [
        Instruction::constant(1),
        Instruction::BrIfNez(target!(0, drop: 0, keep: 0)),
        Instruction::constant(2),
        Instruction::Drop,
        Instruction::Return(DropKeep::new(0, 0)),
    ];
    assert_func_bodies(&wasm, [expected]);
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
    let expected = [Instruction::Return(DropKeep::new(0, 0))];
    assert_func_bodies(&wasm, [expected]);
}

#[test]
fn spec_as_br_if_value_cond() {
    let wasm = wat2wasm(
        r#"
            (func (export "as-br_if-value-cond") (result i32)
                (block (result i32)
                    (drop
                        (br_if 0
                            (i32.const 6)
                            (br_table 0 0
                                (i32.const 9)
                                (i32.const 0)
                            )
                        )
                    )
                    (i32.const 7)
                )
            )
    "#,
    );
    let expected = [
        /* 0 */ Instruction::constant(6),
        /* 1 */ Instruction::constant(9),
        /* 2 */ Instruction::constant(0),
        /* 3 */ Instruction::BrTable { len_targets: 2 },
        /* 4 */ Instruction::Br(target!(6, drop: 1, keep: 1)),
        /* 5 */ Instruction::Br(target!(6, drop: 1, keep: 1)),
        /* 6 */ Instruction::Return(DropKeep::new(0, 1)),
    ];
    assert_func_bodies(&wasm, [expected]);
}

#[test]
fn br_table() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                block $1
                    loop $2
                        i32.const 0
                        br_table $2 $1
                    end
                end
            )
        )
    "#,
    );
    let expected = [
        /* 0 */ Instruction::constant(0),
        /* 1 */ Instruction::BrTable { len_targets: 2 },
        /* 2 */ Instruction::Br(target!(0, drop: 0, keep: 0)),
        /* 3 */ Instruction::Br(target!(4, drop: 0, keep: 0)),
        /* 4 */ Instruction::Return(DropKeep::new(0, 0)),
    ];
    assert_func_bodies(&wasm, [expected]);
}

#[test]
fn br_table_returns_result() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                block $1 (result i32)
                    block $2 (result i32)
                        i32.const 0
                        i32.const 1
                        br_table $2 $1
                    end
                    unreachable
                end
                drop
            )
        )
    "#,
    );
    let expected = [
        /* 0 */ Instruction::constant(0),
        /* 1 */ Instruction::constant(1),
        /* 2 */ Instruction::BrTable { len_targets: 2 },
        /* 3 */ Instruction::Br(target!(5, drop: 0, keep: 1)),
        /* 4 */ Instruction::Br(target!(6, drop: 0, keep: 1)),
        /* 5 */ Instruction::Unreachable,
        /* 6 */ Instruction::Drop,
        /* 7 */ Instruction::Return(DropKeep::new(0, 0)),
    ];
    assert_func_bodies(&wasm, [expected]);
}

#[test]
fn wabt_example() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (result i32)
                block $exit
                    get_local 0
                    br_if $exit
                    i32.const 1
                    return
                end
                i32.const 2
                return
            )
        )
    "#,
    );
    let expected = [
        /* 0 */ Instruction::local_get(1),
        /* 1 */ Instruction::BrIfNez(target!(4, drop: 0, keep: 0)),
        /* 2 */ Instruction::constant(1),
        /* 3 */ Instruction::Return(DropKeep::new(1, 1)),
        /* 4 */ Instruction::constant(2),
        /* 5 */ Instruction::Return(DropKeep::new(1, 1)),
    ];
    assert_func_bodies(&wasm, [expected]);
}

#[test]
fn br_return() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                br 0
                return
            )
        )
    "#,
    );
    let expected = [Instruction::Return(DropKeep::new(0, 0))];
    assert_func_bodies(&wasm, [expected]);
}

#[test]
fn br_if_return() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32)
                local.get 0
                br_if 0
                return
            )
        )
    "#,
    );
    let expected = [
        Instruction::local_get(1),
        Instruction::ReturnIfNez(DropKeep::new(1, 0)),
        Instruction::Return(DropKeep::new(1, 0)),
    ];
    assert_func_bodies(&wasm, [expected]);
}

#[test]
fn br_table_return() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32)
                block $1
                    block $0
                        local.get 0
                        br_table $0 $1 2
                    end
                end
                return
            )
        )
    "#,
    );
    let expected = [
        /* 0 */ Instruction::local_get(1),
        /* 1 */ Instruction::BrTable { len_targets: 3 },
        /* 2 */ Instruction::Br(target!(5, drop: 0, keep: 0)),
        /* 3 */ Instruction::Br(target!(5, drop: 0, keep: 0)),
        /* 4 */ Instruction::Return(DropKeep::new(1, 0)),
        /* 5 */ Instruction::Return(DropKeep::new(1, 0)),
    ];
    assert_func_bodies(&wasm, [expected]);
}
