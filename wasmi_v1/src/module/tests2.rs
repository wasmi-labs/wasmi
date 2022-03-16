use std::fmt::Display;

use wasmi_core::{Value, F32, F64};

use super::*;
use crate::{
    engine::{DedupProviderSlice, Instr, Target},
    engine2::{ExecInstruction, Provider, Register, RegisterEntry},
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
    E: IntoIterator<Item = ExecInstruction>,
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

fn assert_func_bodies_for_module<E, T>(module: &Module, expected: E)
where
    E: IntoIterator<Item = T>,
    T: IntoIterator<Item = ExecInstruction>,
    <T as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let engine = module.engine();
    for ((func_type, func_body), expected) in module.internal_funcs().zip(expected) {
        assert_func_body(engine, func_type, func_body, expected);
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
    T: IntoIterator<Item = ExecInstruction>,
    <T as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let wasm_bytes = wasm_bytes.as_ref();
    let module = create_module(wasm_bytes);
    let engine = module.engine();
    for ((func_type, func_body), expected) in module.internal_funcs().zip(expected) {
        assert_func_body(engine, func_type, func_body, expected);
    }
}

/// Tests compilation of a no-op function.
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
    let expected = [ExecInstruction::Return {
        results: DedupProviderSlice::empty(),
    }];
    assert_func_bodies(&wasm, [expected]);
}

#[test]
fn add_registers() {
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
        ExecInstruction::I32Add {
            result: Register::from_inner(2),
            lhs: Register::from_inner(0),
            rhs: Register::from_inner(1).into(),
        },
        ExecInstruction::Return {
            results: DedupProviderSlice::new(0, 1),
        },
    ];
    assert_func_bodies(&wasm, [expected]);
}

#[test]
fn add_register_and_const() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.add
            )
        )
    "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let rhs = engine.alloc_const(1_i32);
    let expected = [
        ExecInstruction::I32Add {
            result: Register::from_inner(1),
            lhs: Register::from_inner(0),
            rhs: rhs.into(),
        },
        ExecInstruction::Return {
            results: DedupProviderSlice::new(0, 1),
        },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// Due to commutativity of the `add` instruction we can swap operands.
#[test]
fn add_const_and_register() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (result i32)
                i32.const 1
                local.get 0
                i32.add
            )
        )
    "#,
    );
    let module = create_module(&wasm[..]);
    let engine = module.engine();
    let rhs = engine.alloc_const(1_i32);
    let expected = [
        ExecInstruction::I32Add {
            result: Register::from_inner(1),
            lhs: Register::from_inner(0),
            rhs: rhs.into(),
        },
        ExecInstruction::Return {
            results: DedupProviderSlice::new(0, 1),
        },
    ];
    assert_func_bodies_for_module(&module, [expected]);
}

/// Tests compilation of all commutative binary Wasm instructions.
///
/// # Note
///
/// This test specializes on cases where both inputs are constant values.
/// In this case the `wasmi` compiler will directly evaluate the results.
///
/// This includes the following Wasm instructions:
///
/// - `{i32, i64, f32, f64}.add`
/// - `{i32, i64, f32, f64}.mul`
/// - `{i32, i64}.and`
/// - `{i32, i64}.or`
/// - `{i32, i64}.xor`
/// - `{f32, f64}.min`
/// - `{f32, f64}.max`
#[test]
fn commutative_consts() {
    fn run_test<T, E>(ty: &str, wasm_op: &str, lhs: T, rhs: T, exec_op: E)
    where
        T: Display + Into<RegisterEntry>,
        E: FnOnce(T, T) -> T,
    {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (result {ty})
                    {ty}.const {lhs}
                    {ty}.const {rhs}
                    {ty}.{wasm_op}
                )
            )
        "#,
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = engine.alloc_const(exec_op(lhs, rhs).into());
        let results = engine.alloc_provider_slice([Provider::from(result)]);
        let expected = [ExecInstruction::Return { results }];
        assert_func_bodies(&wasm, [expected]);
    }

    run_test::<i32, _>("i32", "add", 1, 2, |lhs, rhs| lhs.wrapping_add(rhs));
    run_test::<i64, _>("i64", "add", 1, 2, |lhs, rhs| lhs.wrapping_add(rhs));
    run_test::<i32, _>("i32", "mul", 1, 2, |lhs, rhs| lhs.wrapping_mul(rhs));
    run_test::<i64, _>("i64", "mul", 1, 2, |lhs, rhs| lhs.wrapping_mul(rhs));
    run_test::<i32, _>("i32", "and", 1, 2, |lhs, rhs| lhs & rhs);
    run_test::<i64, _>("i64", "and", 1, 2, |lhs, rhs| lhs & rhs);
    run_test::<i32, _>("i32", "or", 1, 2, |lhs, rhs| lhs | rhs);
    run_test::<i64, _>("i64", "or", 1, 2, |lhs, rhs| lhs | rhs);
    run_test::<i32, _>("i32", "xor", 1, 2, |lhs, rhs| lhs ^ rhs);
    run_test::<i64, _>("i64", "xor", 1, 2, |lhs, rhs| lhs ^ rhs);

    run_test::<f32, _>("f32", "add", 1.0, 2.0, |lhs, rhs| {
        (F32::from(lhs) + F32::from(rhs)).into()
    });
    run_test::<f64, _>("f64", "add", 1.0, 2.0, |lhs, rhs| {
        (F64::from(lhs) + F64::from(rhs)).into()
    });
    run_test::<f32, _>("f32", "mul", 1.0, 2.0, |lhs, rhs| {
        (F32::from(lhs) * F32::from(rhs)).into()
    });
    run_test::<f64, _>("f64", "mul", 1.0, 2.0, |lhs, rhs| {
        (F64::from(lhs) * F64::from(rhs)).into()
    });
    run_test::<f32, _>("f32", "min", 1.0, 2.0, |lhs, rhs| {
        F32::from(lhs).min(F32::from(rhs)).into()
    });
    run_test::<f64, _>("f64", "min", 1.0, 2.0, |lhs, rhs| {
        F64::from(lhs).min(F64::from(rhs)).into()
    });
    run_test::<f32, _>("f32", "max", 1.0, 2.0, |lhs, rhs| {
        F32::from(lhs).max(F32::from(rhs)).into()
    });
    run_test::<f64, _>("f64", "max", 1.0, 2.0, |lhs, rhs| {
        F64::from(lhs).max(F64::from(rhs)).into()
    });
}

/// Tests translation of Wasm `{i32,i64}.eqz` functions.
///
/// # Note
///
/// This tests asserts correct compilation of register inputs.
#[test]
fn cmp_zero_register() {
    fn run_test<T, F>(ty: &str, make_op: F)
    where
        T: Default + Into<RegisterEntry>,
        F: FnOnce(Register, Register, Provider) -> ExecInstruction,
    {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {ty}) (result i32)
                    local.get 0
                    {ty}.eqz
                )
            )
        "#,
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let rhs = engine.alloc_const(T::default());
        let result = Register::from_inner(1);
        let results = engine.alloc_provider_slice([Provider::from_register(result)]);
        let expected = [
            make_op(result, Register::from_inner(0), rhs.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies(&wasm, [expected]);
    }

    run_test::<i32, _>("i32", |result, lhs, rhs| ExecInstruction::I32Eq {
        result,
        lhs,
        rhs,
    });
    run_test::<i64, _>("i64", |result, lhs, rhs| ExecInstruction::I64Eq {
        result,
        lhs,
        rhs,
    });
}

/// Tests translation of Wasm `{i32,i64}.eqz` functions.
///
/// # Note
///
/// This tests asserts compile time evaluation of constant value inputs.
#[test]
fn cmp_zero_const() {
    fn run_test<T, F>(ty: &str, value: T, exec_op: F)
    where
        T: Default + Display + Into<RegisterEntry>,
        F: FnOnce(T) -> bool,
    {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (result i32)
                    {ty}.const {value}
                    {ty}.eqz
                )
            )
        "#,
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = engine.alloc_const(exec_op(value));
        let results = engine.alloc_provider_slice([Provider::from(result)]);
        let expected = [ExecInstruction::Return { results }];
        assert_func_bodies(&wasm, [expected]);
    }

    run_test("i32", 1, |input: i32| input == 0);
    run_test("i64", 1, |input: i64| input == 0);
}

/// Tests translation of all Wasm comparison functions.
///
/// # Note
///
/// In this test all Wasm functions have 2 registers (`local.get`) as inputs.
/// This is one of the simple cases to cover.
#[test]
fn cmp_registers() {
    fn run_test<F>(ty: &str, wasm_op: &str, make_op: F)
    where
        F: FnOnce(Register, Register, Provider) -> ExecInstruction,
    {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {ty}) (param {ty}) (result i32)
                    local.get 0
                    local.get 1
                    {ty}.{wasm_op}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = Register::from_inner(2);
        let results = engine.alloc_provider_slice([Provider::from_register(result)]);
        let expected = [
            make_op(
                result,
                Register::from_inner(0),
                Register::from_inner(1).into(),
            ),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies(&wasm, [expected]);
    }
    run_test("i32", "lt_s", |result, lhs, rhs| ExecInstruction::I32LtS {
        result,
        lhs,
        rhs,
    });
    run_test("i32", "lt_u", |result, lhs, rhs| ExecInstruction::I32LtU {
        result,
        lhs,
        rhs,
    });
    run_test("i32", "gt_s", |result, lhs, rhs| ExecInstruction::I32GtS {
        result,
        lhs,
        rhs,
    });
    run_test("i32", "gt_u", |result, lhs, rhs| ExecInstruction::I32GtU {
        result,
        lhs,
        rhs,
    });
    run_test("i64", "lt_s", |result, lhs, rhs| ExecInstruction::I64LtS {
        result,
        lhs,
        rhs,
    });
    run_test("i64", "lt_u", |result, lhs, rhs| ExecInstruction::I64LtU {
        result,
        lhs,
        rhs,
    });
    run_test("i64", "gt_s", |result, lhs, rhs| ExecInstruction::I64GtS {
        result,
        lhs,
        rhs,
    });
    run_test("i64", "gt_u", |result, lhs, rhs| ExecInstruction::I64GtU {
        result,
        lhs,
        rhs,
    });

    run_test("f32", "lt", |result, lhs, rhs| ExecInstruction::F32Lt {
        result,
        lhs,
        rhs,
    });
    run_test("f32", "le", |result, lhs, rhs| ExecInstruction::F32Le {
        result,
        lhs,
        rhs,
    });
    run_test("f32", "gt", |result, lhs, rhs| ExecInstruction::F32Gt {
        result,
        lhs,
        rhs,
    });
    run_test("f32", "ge", |result, lhs, rhs| ExecInstruction::F32Ge {
        result,
        lhs,
        rhs,
    });

    run_test("f64", "lt", |result, lhs, rhs| ExecInstruction::F64Lt {
        result,
        lhs,
        rhs,
    });
    run_test("f64", "le", |result, lhs, rhs| ExecInstruction::F64Le {
        result,
        lhs,
        rhs,
    });
    run_test("f64", "gt", |result, lhs, rhs| ExecInstruction::F64Gt {
        result,
        lhs,
        rhs,
    });
    run_test("f64", "ge", |result, lhs, rhs| ExecInstruction::F64Ge {
        result,
        lhs,
        rhs,
    });
}

/// Tests translation of all Wasm comparison functions.
///
/// # Note
///
/// In this test all Wasm functions have 1 register (`local.get`)
/// and a constant value (`i32.const`) as inputs.
///
/// This is one of the simple cases to cover.
#[test]
fn cmp_register_and_const() {
    fn run_test<T, F>(ty: &str, wasm_op: &str, value: T, make_op: F)
    where
        T: Display + Into<RegisterEntry>,
        F: FnOnce(Register, Register, Provider) -> ExecInstruction,
    {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {ty}) (result i32)
                    local.get 0
                    {ty}.const {value}
                    {ty}.{wasm_op}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = Register::from_inner(1);
        let results = engine.alloc_provider_slice([Provider::from_register(result)]);
        let rhs = engine.alloc_const(value);
        let expected = [
            make_op(result, Register::from_inner(0), rhs.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies(&wasm, [expected]);
    }
    run_test("i32", "lt_s", 1_i32, |result, lhs, rhs| {
        ExecInstruction::I32LtS { result, lhs, rhs }
    });
    run_test("i32", "lt_u", 1_i32, |result, lhs, rhs| {
        ExecInstruction::I32LtU { result, lhs, rhs }
    });
    run_test("i32", "gt_s", 1_i32, |result, lhs, rhs| {
        ExecInstruction::I32GtS { result, lhs, rhs }
    });
    run_test("i32", "gt_u", 1_i32, |result, lhs, rhs| {
        ExecInstruction::I32GtU { result, lhs, rhs }
    });
    run_test("i64", "lt_s", 1_i32, |result, lhs, rhs| {
        ExecInstruction::I64LtS { result, lhs, rhs }
    });
    run_test("i64", "lt_u", 1_i32, |result, lhs, rhs| {
        ExecInstruction::I64LtU { result, lhs, rhs }
    });
    run_test("i64", "gt_s", 1_i32, |result, lhs, rhs| {
        ExecInstruction::I64GtS { result, lhs, rhs }
    });
    run_test("i64", "gt_u", 1_i32, |result, lhs, rhs| {
        ExecInstruction::I64GtU { result, lhs, rhs }
    });

    run_test("f32", "lt", 1.0_f32, |result, lhs, rhs| {
        ExecInstruction::F32Lt { result, lhs, rhs }
    });
    run_test("f32", "le", 1.0_f32, |result, lhs, rhs| {
        ExecInstruction::F32Le { result, lhs, rhs }
    });
    run_test("f32", "gt", 1.0_f32, |result, lhs, rhs| {
        ExecInstruction::F32Gt { result, lhs, rhs }
    });
    run_test("f32", "ge", 1.0_f32, |result, lhs, rhs| {
        ExecInstruction::F32Ge { result, lhs, rhs }
    });

    run_test("f64", "lt", 1.0_f64, |result, lhs, rhs| {
        ExecInstruction::F64Lt { result, lhs, rhs }
    });
    run_test("f64", "le", 1.0_f64, |result, lhs, rhs| {
        ExecInstruction::F64Le { result, lhs, rhs }
    });
    run_test("f64", "gt", 1.0_f64, |result, lhs, rhs| {
        ExecInstruction::F64Gt { result, lhs, rhs }
    });
    run_test("f64", "ge", 1.0_f64, |result, lhs, rhs| {
        ExecInstruction::F64Ge { result, lhs, rhs }
    });
}

/// Tests translation of all Wasm comparison functions.
///
/// # Note
///
/// In this test all Wasm functions have 1 register (`local.get`)
/// and a constant value (`i32.const`) as inputs.
///
/// This is generally non-trivial to handle but comparison functions
/// make it easy to swap the operands by switching to the reversed comparison
/// instruction, e.g. switching from `less-than` to `greater-than`.
#[test]
fn cmp_const_and_register() {
    fn run_test<T, F>(ty: &str, wasm_op: &str, value: T, make_op: F)
    where
        T: Display + Into<RegisterEntry>,
        F: FnOnce(Register, Register, Provider) -> ExecInstruction,
    {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {ty}) (result i32)
                    {ty}.const {value}
                    local.get 0
                    {ty}.{wasm_op}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = Register::from_inner(1);
        let results = engine.alloc_provider_slice([Provider::from_register(result)]);
        let rhs = engine.alloc_const(value);
        let expected = [
            make_op(result, Register::from_inner(0), rhs.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies(&wasm, [expected]);
    }
    run_test("i32", "lt_s", 1_i32, |result, lhs, rhs| {
        ExecInstruction::I32GtS { result, lhs, rhs }
    });
    run_test("i32", "lt_u", 1_i32, |result, lhs, rhs| {
        ExecInstruction::I32GtU { result, lhs, rhs }
    });
    run_test("i32", "gt_s", 1_i32, |result, lhs, rhs| {
        ExecInstruction::I32LtS { result, lhs, rhs }
    });
    run_test("i32", "gt_u", 1_i32, |result, lhs, rhs| {
        ExecInstruction::I32LtU { result, lhs, rhs }
    });
    run_test("i64", "lt_s", 1_i32, |result, lhs, rhs| {
        ExecInstruction::I64GtS { result, lhs, rhs }
    });
    run_test("i64", "lt_u", 1_i32, |result, lhs, rhs| {
        ExecInstruction::I64GtU { result, lhs, rhs }
    });
    run_test("i64", "gt_s", 1_i32, |result, lhs, rhs| {
        ExecInstruction::I64LtS { result, lhs, rhs }
    });
    run_test("i64", "gt_u", 1_i32, |result, lhs, rhs| {
        ExecInstruction::I64LtU { result, lhs, rhs }
    });

    run_test("f32", "lt", 1.0_f32, |result, lhs, rhs| {
        ExecInstruction::F32Gt { result, lhs, rhs }
    });
    run_test("f32", "le", 1.0_f32, |result, lhs, rhs| {
        ExecInstruction::F32Ge { result, lhs, rhs }
    });
    run_test("f32", "gt", 1.0_f32, |result, lhs, rhs| {
        ExecInstruction::F32Lt { result, lhs, rhs }
    });
    run_test("f32", "ge", 1.0_f32, |result, lhs, rhs| {
        ExecInstruction::F32Le { result, lhs, rhs }
    });

    run_test("f64", "lt", 1.0_f64, |result, lhs, rhs| {
        ExecInstruction::F64Gt { result, lhs, rhs }
    });
    run_test("f64", "le", 1.0_f64, |result, lhs, rhs| {
        ExecInstruction::F64Ge { result, lhs, rhs }
    });
    run_test("f64", "gt", 1.0_f64, |result, lhs, rhs| {
        ExecInstruction::F64Lt { result, lhs, rhs }
    });
    run_test("f64", "ge", 1.0_f64, |result, lhs, rhs| {
        ExecInstruction::F64Le { result, lhs, rhs }
    });
}

/// Tests translation of all Wasm comparison functions.
///
/// # Note
///
/// In this test all Wasm functions have 2 constant values (`i32.const`) as inputs.
///
/// In this case we can simply apply const folding to resolve the instruction
/// entirely.
#[test]
fn cmp_const_and_const() {
    fn run_test<T, E>(ty: &str, wasm_op: &str, lhs: T, rhs: T, exec_op: E)
    where
        T: Display + Into<RegisterEntry> + PartialOrd,
        E: FnOnce(T, T) -> bool,
    {
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {ty}) (result i32)
                    {ty}.const {lhs}
                    {ty}.const {rhs}
                    {ty}.{wasm_op}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = engine.alloc_const(exec_op(lhs, rhs) as i32);
        let results = engine.alloc_provider_slice([Provider::from(result)]);
        let expected = [ExecInstruction::Return { results }];
        assert_func_bodies(&wasm, [expected]);
    }
    run_test("i32", "lt_s", 1_i32, 2_i32, |l, r| l < r);
    run_test("i32", "lt_u", 1_i32, 2_i32, |l, r| l < r);
    run_test("i32", "gt_s", 1_i32, 2_i32, |l, r| l > r);
    run_test("i32", "gt_u", 1_i32, 2_i32, |l, r| l > r);
    run_test("i64", "lt_s", 1_i64, 2_i64, |l, r| l < r);
    run_test("i64", "lt_u", 1_i64, 2_i64, |l, r| l < r);
    run_test("i64", "gt_s", 1_i64, 2_i64, |l, r| l > r);
    run_test("i64", "gt_u", 1_i64, 2_i64, |l, r| l > r);

    run_test("f32", "lt", 1.0_f32, 2.0_f32, |l, r| l < r);
    run_test("f32", "le", 1.0_f32, 2.0_f32, |l, r| l <= r);
    run_test("f32", "gt", 1.0_f32, 2.0_f32, |l, r| l > r);
    run_test("f32", "ge", 1.0_f32, 2.0_f32, |l, r| l >= r);

    run_test("f64", "lt", 1.0_f64, 2.0_f64, |l, r| l < r);
    run_test("f64", "le", 1.0_f64, 2.0_f64, |l, r| l <= r);
    run_test("f64", "gt", 1.0_f64, 2.0_f64, |l, r| l > r);
    run_test("f64", "ge", 1.0_f64, 2.0_f64, |l, r| l >= r);
}
