use std::fmt::Display;

use super::*;
use crate::{
    engine::{DedupProviderSlice, Instr, Target},
    engine2::{ExecInstruction, Provider, Register, RegisterEntry, WasmType},
    Engine,
};
use wasmi_core::{Float, SignExtendFrom, Value, F32, F64};

/// Allows to create a `1` instance for a type.
pub trait One {
    /// Returns a value of `Self` that equals or represents `1` (one).
    fn one() -> Self;
}

macro_rules! impl_one_for {
    ( $( type $ty:ty = $value:literal );* $(;)? ) => {
        $(
            impl One for $ty {
                fn one() -> Self {
                    $value
                }
            }
        )*
    };
}

impl_one_for! {
    type i32 = 1_i32;
    type i64 = 1_i64;
    type f32 = 1.0_f32;
    type f64 = 1.0_f64;
}

/// Implemented by Wasm compatible types to print them into `.wat` sources.
pub trait WasmTypeName {
    /// The Wasm name of `Self`.
    const NAME: &'static str;
}

macro_rules! impl_wasm_type_name {
    ( $( type $ty:ty = $name:literal );* $(;)? ) => {
        $(
            impl WasmTypeName for $ty {
                const NAME: &'static str = $name;
            }
        )*
    };
}

impl_wasm_type_name! {
    type i32 = "i32";
    type u32 = "i32";
    type i64 = "i64";
    type u64 = "i64";
    type f32 = "f32";
    type f64 = "f64";
    type F32 = "f32";
    type F64 = "f64";
    type bool = "i32";
}

/// Creates a closure taking 3 parameters and constructing a `wasmi` instruction.
macro_rules! make_op {
    ( $name:ident ) => {{
        |result, lhs, rhs| ExecInstruction::$name { result, lhs, rhs }
    }};
}

/// Creates a closure taking 2 parameters and constructing a `wasmi` instruction.
macro_rules! make_op2 {
    ( $name:ident ) => {{
        |result, input| ExecInstruction::$name { result, input }
    }};
}

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

/// Tests compilation of all commutative binary Wasm instructions.
///
/// # Note
///
/// This test specializes on cases where both inputs are register inputs
/// (e.g. `local.get 0`).
/// This is the most trivial case to cover and simply checks that the
/// correct instruction with the correct operands is resulting.
///
/// This includes the following Wasm instructions:
///
/// - `{i32, i64, f32, f64}.eq`
/// - `{i32, i64, f32, f64}.ne`
/// - `{i32, i64, f32, f64}.add`
/// - `{i32, i64, f32, f64}.mul`
/// - `{i32, i64}.and`
/// - `{i32, i64}.or`
/// - `{i32, i64}.xor`
/// - `{f32, f64}.min`
/// - `{f32, f64}.max`
#[test]
fn commutative_registers() {
    fn run_test<T, F, R>(wasm_op: &str, make_op: F)
    where
        T: Display + WasmTypeName + Into<RegisterEntry>,
        F: FnOnce(Register, Register, Provider) -> ExecInstruction,
        R: WasmTypeName,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let output_type = <R as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {input_type}) (param {input_type}) (result {output_type})
                    local.get 0
                    local.get 1
                    {input_type}.{wasm_op}
                )
            )
        "#,
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let lhs = Register::from_inner(0);
        let rhs = Register::from_inner(1);
        let result = Register::from_inner(2);
        let results = engine.alloc_provider_slice([Provider::from_register(result)]);
        let expected = [
            make_op(result, lhs, rhs.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    fn run_test_bin<T, F>(wasm_op: &str, make_op: F)
    where
        T: Display + WasmTypeName + Into<RegisterEntry>,
        F: FnOnce(Register, Register, Provider) -> ExecInstruction,
    {
        run_test::<T, F, T>(wasm_op, make_op)
    }

    fn run_test_cmp<T, F>(wasm_op: &str, make_op: F)
    where
        T: Display + WasmTypeName + Into<RegisterEntry>,
        F: FnOnce(Register, Register, Provider) -> ExecInstruction,
    {
        run_test::<T, F, bool>(wasm_op, make_op)
    }

    run_test_cmp::<i32, _>("eq", make_op!(I32Eq));
    run_test_cmp::<i64, _>("eq", make_op!(I64Eq));
    run_test_cmp::<i32, _>("ne", make_op!(I32Ne));
    run_test_cmp::<i64, _>("ne", make_op!(I64Ne));

    run_test_bin::<i32, _>("add", make_op!(I32Add));
    run_test_bin::<i64, _>("add", make_op!(I64Add));
    run_test_bin::<i32, _>("mul", make_op!(I32Mul));
    run_test_bin::<i64, _>("mul", make_op!(I64Mul));
    run_test_bin::<i32, _>("and", make_op!(I32And));
    run_test_bin::<i64, _>("and", make_op!(I64And));
    run_test_bin::<i32, _>("or", make_op!(I32Or));
    run_test_bin::<i64, _>("or", make_op!(I64Or));
    run_test_bin::<i32, _>("xor", make_op!(I32Xor));
    run_test_bin::<i64, _>("xor", make_op!(I64Xor));

    run_test_cmp::<f32, _>("eq", make_op!(F32Eq));
    run_test_cmp::<f64, _>("eq", make_op!(F64Eq));
    run_test_cmp::<f32, _>("ne", make_op!(F32Ne));
    run_test_cmp::<f64, _>("ne", make_op!(F64Ne));

    run_test_bin::<f32, _>("add", make_op!(F32Add));
    run_test_bin::<f64, _>("add", make_op!(F64Add));
    run_test_bin::<f32, _>("mul", make_op!(F32Mul));
    run_test_bin::<f64, _>("mul", make_op!(F64Mul));
    run_test_bin::<f32, _>("min", make_op!(F32Min));
    run_test_bin::<f64, _>("min", make_op!(F64Min));
    run_test_bin::<f32, _>("max", make_op!(F32Max));
    run_test_bin::<f64, _>("max", make_op!(F64Max));
}

/// Tests compilation of all commutative binary Wasm instructions.
///
/// # Note
///
/// This test specializes on cases where one of the inputs is a constant value
/// (e.g. `i32.const 1`) and the other a register input (e.g. `local.get 0`).
/// In this case the `wasmi` compiler may swap the order of operands in order
/// to represents the `wasmi` bytecode in a more compact form.
///
/// This includes the following Wasm instructions:
///
/// - `{i32, i64, f32, f64}.eq`
/// - `{i32, i64, f32, f64}.ne`
/// - `{i32, i64, f32, f64}.add`
/// - `{i32, i64, f32, f64}.mul`
/// - `{i32, i64}.and`
/// - `{i32, i64}.or`
/// - `{i32, i64}.xor`
/// - `{f32, f64}.min`
/// - `{f32, f64}.max`
#[test]
fn commutative_const_register() {
    fn run_test<T, F, R>(wasm: &[u8], wasm_op: &str, make_op: F)
    where
        T: Display + WasmTypeName + One + Into<RegisterEntry>,
        F: FnOnce(Register, Register, Provider) -> ExecInstruction,
        R: WasmTypeName,
    {
        let one = T::one();
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let rhs = engine.alloc_const(one);
        let result = Register::from_inner(1);
        let results = engine.alloc_provider_slice([Provider::from_register(result)]);
        let expected = [
            make_op(Register::from_inner(1), Register::from_inner(0), rhs.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    fn run_test_cr<T, F, R>(wasm_op: &str, make_op: F)
    where
        T: Display + WasmTypeName + One + Into<RegisterEntry>,
        F: FnOnce(Register, Register, Provider) -> ExecInstruction,
        R: WasmTypeName,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let output_type = <R as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {input_type}) (result {output_type})
                    {input_type}.const 1
                    local.get 0
                    {input_type}.{wasm_op}
                )
            )
        "#,
        ));
        run_test::<T, F, R>(&wasm[..], wasm_op, make_op);
    }

    fn run_test_rc<T, F, R>(wasm_op: &str, make_op: F)
    where
        T: Display + WasmTypeName + One + Into<RegisterEntry>,
        F: FnOnce(Register, Register, Provider) -> ExecInstruction,
        R: WasmTypeName,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let output_type = <R as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {input_type}) (result {output_type})
                    local.get 0
                    {input_type}.const 1
                    {input_type}.{wasm_op}
                )
            )
        "#,
        ));
        run_test::<T, F, R>(&wasm[..], wasm_op, make_op);
    }

    fn run_test_bin<T, F>(wasm_op: &str, make_op: F)
    where
        T: Display + Into<RegisterEntry> + WasmTypeName + One,
        F: FnOnce(Register, Register, Provider) -> ExecInstruction + Copy,
    {
        run_test_cr::<T, F, T>(wasm_op, make_op);
        run_test_rc::<T, F, T>(wasm_op, make_op);
    }

    fn run_test_cmp<T, F>(wasm_op: &str, make_op: F)
    where
        T: Display + Into<RegisterEntry> + WasmTypeName + One,
        F: FnOnce(Register, Register, Provider) -> ExecInstruction + Copy,
    {
        run_test_cr::<T, F, bool>(wasm_op, make_op);
        run_test_rc::<T, F, bool>(wasm_op, make_op);
    }

    run_test_cmp::<i32, _>("eq", make_op!(I32Eq));
    run_test_cmp::<i64, _>("eq", make_op!(I64Eq));
    run_test_cmp::<i32, _>("ne", make_op!(I32Ne));
    run_test_cmp::<i64, _>("ne", make_op!(I64Ne));

    run_test_bin::<i32, _>("add", make_op!(I32Add));
    run_test_bin::<i64, _>("add", make_op!(I64Add));
    run_test_bin::<i32, _>("mul", make_op!(I32Mul));
    run_test_bin::<i64, _>("mul", make_op!(I64Mul));
    run_test_bin::<i32, _>("and", make_op!(I32And));
    run_test_bin::<i64, _>("and", make_op!(I64And));
    run_test_bin::<i32, _>("or", make_op!(I32Or));
    run_test_bin::<i64, _>("or", make_op!(I64Or));
    run_test_bin::<i32, _>("xor", make_op!(I32Xor));
    run_test_bin::<i64, _>("xor", make_op!(I64Xor));

    run_test_cmp::<f32, _>("eq", make_op!(F32Eq));
    run_test_cmp::<f64, _>("eq", make_op!(F64Eq));
    run_test_cmp::<f32, _>("ne", make_op!(F32Ne));
    run_test_cmp::<f64, _>("ne", make_op!(F64Ne));

    run_test_bin::<f32, _>("add", make_op!(F32Add));
    run_test_bin::<f64, _>("add", make_op!(F64Add));
    run_test_bin::<f32, _>("mul", make_op!(F32Mul));
    run_test_bin::<f64, _>("mul", make_op!(F64Mul));
    run_test_bin::<f32, _>("min", make_op!(F32Min));
    run_test_bin::<f32, _>("min", make_op!(F32Min));
    run_test_bin::<f64, _>("max", make_op!(F64Max));
    run_test_bin::<f64, _>("max", make_op!(F64Max));
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
/// - `{i32, i64, f32, f64}.eq`
/// - `{i32, i64, f32, f64}.ne`
/// - `{i32, i64, f32, f64}.add`
/// - `{i32, i64, f32, f64}.mul`
/// - `{i32, i64}.and`
/// - `{i32, i64}.or`
/// - `{i32, i64}.xor`
/// - `{f32, f64}.min`
/// - `{f32, f64}.max`
#[test]
fn commutative_consts() {
    fn run_test<T, E, R>(wasm_op: &str, lhs: T, rhs: T, exec_op: E)
    where
        T: Display + WasmTypeName,
        E: FnOnce(T, T) -> R,
        R: Into<RegisterEntry> + WasmTypeName,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let output_type = <R as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (result {output_type})
                    {input_type}.const {lhs}
                    {input_type}.const {rhs}
                    {input_type}.{wasm_op}
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

    fn run_test_bin<T, E>(wasm_op: &str, lhs: T, rhs: T, exec_op: E)
    where
        T: Display + Into<RegisterEntry> + WasmTypeName,
        E: FnOnce(T, T) -> T,
    {
        run_test::<T, E, T>(wasm_op, lhs, rhs, exec_op)
    }

    fn run_test_cmp<T, E>(wasm_op: &str, lhs: T, rhs: T, exec_op: E)
    where
        T: Display + WasmTypeName,
        E: FnOnce(T, T) -> bool,
    {
        run_test::<T, E, bool>(wasm_op, lhs, rhs, exec_op)
    }

    run_test_cmp::<i32, _>("eq", 1, 2, |lhs, rhs| lhs == rhs);
    run_test_cmp::<i64, _>("eq", 1, 2, |lhs, rhs| lhs == rhs);
    run_test_cmp::<i32, _>("ne", 1, 2, |lhs, rhs| lhs != rhs);
    run_test_cmp::<i64, _>("ne", 1, 2, |lhs, rhs| lhs != rhs);

    run_test_bin::<i32, _>("add", 1, 2, |lhs, rhs| lhs.wrapping_add(rhs));
    run_test_bin::<i64, _>("add", 1, 2, |lhs, rhs| lhs.wrapping_add(rhs));
    run_test_bin::<i32, _>("mul", 1, 2, |lhs, rhs| lhs.wrapping_mul(rhs));
    run_test_bin::<i64, _>("mul", 1, 2, |lhs, rhs| lhs.wrapping_mul(rhs));
    run_test_bin::<i32, _>("and", 1, 2, |lhs, rhs| lhs & rhs);
    run_test_bin::<i64, _>("and", 1, 2, |lhs, rhs| lhs & rhs);
    run_test_bin::<i32, _>("or", 1, 2, |lhs, rhs| lhs | rhs);
    run_test_bin::<i64, _>("or", 1, 2, |lhs, rhs| lhs | rhs);
    run_test_bin::<i32, _>("xor", 1, 2, |lhs, rhs| lhs ^ rhs);
    run_test_bin::<i64, _>("xor", 1, 2, |lhs, rhs| lhs ^ rhs);

    run_test_cmp::<f32, _>("eq", 1.0, 2.0, |lhs, rhs| F32::from(lhs) == F32::from(rhs));
    run_test_cmp::<f64, _>("eq", 1.0, 2.0, |lhs, rhs| F64::from(lhs) == F64::from(rhs));
    run_test_cmp::<f32, _>("ne", 1.0, 2.0, |lhs, rhs| F32::from(lhs) != F32::from(rhs));
    run_test_cmp::<f64, _>("ne", 1.0, 2.0, |lhs, rhs| F64::from(lhs) != F64::from(rhs));

    run_test_bin::<f32, _>("add", 1.0, 2.0, |lhs, rhs| {
        (F32::from(lhs) + F32::from(rhs)).into()
    });
    run_test_bin::<f64, _>("add", 1.0, 2.0, |lhs, rhs| {
        (F64::from(lhs) + F64::from(rhs)).into()
    });
    run_test_bin::<f32, _>("mul", 1.0, 2.0, |lhs, rhs| {
        (F32::from(lhs) * F32::from(rhs)).into()
    });
    run_test_bin::<f64, _>("mul", 1.0, 2.0, |lhs, rhs| {
        (F64::from(lhs) * F64::from(rhs)).into()
    });
    run_test_bin::<f32, _>("min", 1.0, 2.0, |lhs, rhs| {
        F32::from(lhs).min(F32::from(rhs)).into()
    });
    run_test_bin::<f64, _>("min", 1.0, 2.0, |lhs, rhs| {
        F64::from(lhs).min(F64::from(rhs)).into()
    });
    run_test_bin::<f32, _>("max", 1.0, 2.0, |lhs, rhs| {
        F32::from(lhs).max(F32::from(rhs)).into()
    });
    run_test_bin::<f64, _>("max", 1.0, 2.0, |lhs, rhs| {
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

    run_test::<i32, _>("i32", make_op!(I32Eq));
    run_test::<i64, _>("i64", make_op!(I64Eq));
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
    run_test("i32", "lt_s", make_op!(I32LtS));
    run_test("i32", "lt_u", make_op!(I32LtU));
    run_test("i32", "gt_s", make_op!(I32GtS));
    run_test("i32", "gt_u", make_op!(I32GtU));
    run_test("i64", "lt_s", make_op!(I64LtS));
    run_test("i64", "lt_u", make_op!(I64LtU));
    run_test("i64", "gt_s", make_op!(I64GtS));
    run_test("i64", "gt_u", make_op!(I64GtU));

    run_test("f32", "lt", make_op!(F32Lt));
    run_test("f32", "le", make_op!(F32Le));
    run_test("f32", "gt", make_op!(F32Gt));
    run_test("f32", "ge", make_op!(F32Ge));

    run_test("f64", "lt", make_op!(F64Lt));
    run_test("f64", "le", make_op!(F64Le));
    run_test("f64", "gt", make_op!(F64Gt));
    run_test("f64", "ge", make_op!(F64Ge));
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
    run_test("i32", "lt_s", 1_i32, make_op!(I32LtS));
    run_test("i32", "lt_u", 1_i32, make_op!(I32LtU));
    run_test("i32", "gt_s", 1_i32, make_op!(I32GtS));
    run_test("i32", "gt_u", 1_i32, make_op!(I32GtU));
    run_test("i64", "lt_s", 1_i32, make_op!(I64LtS));
    run_test("i64", "lt_u", 1_i32, make_op!(I64LtU));
    run_test("i64", "gt_s", 1_i32, make_op!(I64GtS));
    run_test("i64", "gt_u", 1_i32, make_op!(I64GtU));

    run_test("f32", "lt", 1.0_f32, make_op!(F32Lt));
    run_test("f32", "le", 1.0_f32, make_op!(F32Le));
    run_test("f32", "gt", 1.0_f32, make_op!(F32Gt));
    run_test("f32", "ge", 1.0_f32, make_op!(F32Ge));

    run_test("f64", "lt", 1.0_f64, make_op!(F64Lt));
    run_test("f64", "le", 1.0_f64, make_op!(F64Le));
    run_test("f64", "gt", 1.0_f64, make_op!(F64Gt));
    run_test("f64", "ge", 1.0_f64, make_op!(F64Ge));
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
    run_test("i32", "lt_s", 1_i32, make_op!(I32GtS));
    run_test("i32", "lt_u", 1_i32, make_op!(I32GtU));
    run_test("i32", "gt_s", 1_i32, make_op!(I32LtS));
    run_test("i32", "gt_u", 1_i32, make_op!(I32LtU));
    run_test("i64", "lt_s", 1_i32, make_op!(I64GtS));
    run_test("i64", "lt_u", 1_i32, make_op!(I64GtU));
    run_test("i64", "gt_s", 1_i32, make_op!(I64LtS));
    run_test("i64", "gt_u", 1_i32, make_op!(I64LtU));

    run_test("f32", "lt", 1.0_f32, make_op!(F32Gt));
    run_test("f32", "le", 1.0_f32, make_op!(F32Ge));
    run_test("f32", "gt", 1.0_f32, make_op!(F32Lt));
    run_test("f32", "ge", 1.0_f32, make_op!(F32Le));

    run_test("f64", "lt", 1.0_f64, make_op!(F64Gt));
    run_test("f64", "le", 1.0_f64, make_op!(F64Ge));
    run_test("f64", "gt", 1.0_f64, make_op!(F64Lt));
    run_test("f64", "ge", 1.0_f64, make_op!(F64Le));
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

/// Tests translation of all unary Wasm instructions.
///
/// # Note
///
/// In this test all Wasm functions have a register input (e.g. via `local.get`).
///
/// This tests the following Wasm instructions:
///
/// - `{i32, i64}.clz`
/// - `{i32, i64}.ctz`
/// - `{i32, i64}.popcnt`
/// - `{i32, i64}.extend_8s`
/// - `{i32, i64}.extend_16s`
/// - `i64.extend_32s`
/// - `{f32, f64}.abs`
/// - `{f32, f64}.neg`
/// - `{f32, f64}.ceil`
/// - `{f32, f64}.floor`
/// - `{f32, f64}.trunc`
/// - `{f32, f64}.nearest`
/// - `{f32, f64}.sqrt`
#[test]
fn unary_register() {
    fn run_test<T, F>(wasm_op: &str, make_op: F)
    where
        T: WasmTypeName,
        F: FnOnce(Register, Register) -> ExecInstruction,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {input_type}) (result {input_type})
                    local.get 0
                    {input_type}.{wasm_op}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = Register::from_inner(1);
        let results = engine.alloc_provider_slice([Provider::from_register(result)]);
        let input = Register::from_inner(0);
        let expected = [make_op(result, input), ExecInstruction::Return { results }];
        assert_func_bodies(&wasm, [expected]);
    }

    run_test::<i32, _>("clz", make_op2!(I32Clz));
    run_test::<i64, _>("clz", make_op2!(I64Clz));
    run_test::<i32, _>("ctz", make_op2!(I32Ctz));
    run_test::<i64, _>("ctz", make_op2!(I64Ctz));
    run_test::<i32, _>("popcnt", make_op2!(I32Popcnt));
    run_test::<i64, _>("popcnt", make_op2!(I64Popcnt));
    run_test::<i32, _>("extend8_s", make_op2!(I32Extend8S));
    run_test::<i64, _>("extend8_s", make_op2!(I64Extend8S));
    run_test::<i32, _>("extend16_s", make_op2!(I32Extend16S));
    run_test::<i64, _>("extend16_s", make_op2!(I64Extend16S));
    run_test::<i64, _>("extend32_s", make_op2!(I64Extend32S));
    run_test::<f32, _>("abs", make_op2!(F32Abs));
    run_test::<f64, _>("abs", make_op2!(F64Abs));
    run_test::<f32, _>("neg", make_op2!(F32Neg));
    run_test::<f64, _>("neg", make_op2!(F64Neg));
    run_test::<f32, _>("ceil", make_op2!(F32Ceil));
    run_test::<f64, _>("ceil", make_op2!(F64Ceil));
    run_test::<f32, _>("floor", make_op2!(F32Floor));
    run_test::<f64, _>("floor", make_op2!(F64Floor));
    run_test::<f32, _>("trunc", make_op2!(F32Trunc));
    run_test::<f64, _>("trunc", make_op2!(F64Trunc));
    run_test::<f32, _>("nearest", make_op2!(F32Nearest));
    run_test::<f64, _>("nearest", make_op2!(F64Nearest));
    run_test::<f32, _>("sqrt", make_op2!(F32Sqrt));
    run_test::<f64, _>("sqrt", make_op2!(F64Sqrt));
}

/// Tests translation of all unary Wasm instructions.
///
/// # Note
///
/// In this test all Wasm functions have a constant input (e.g. via `i32.const`).
///
/// This tests the following Wasm instructions:
///
/// - `{i32, i64}.clz`
/// - `{i32, i64}.ctz`
/// - `{i32, i64}.popcnt`
/// - `{i32, i64}.extend_8s`
/// - `{i32, i64}.extend_16s`
/// - `i64.extend_32s`
/// - `{f32, f64}.abs`
/// - `{f32, f64}.neg`
/// - `{f32, f64}.ceil`
/// - `{f32, f64}.floor`
/// - `{f32, f64}.trunc`
/// - `{f32, f64}.nearest`
/// - `{f32, f64}.sqrt`
#[test]
fn unary_const() {
    fn run_test<T, F, R>(wasm_op: &str, input: T, exec_op: F)
    where
        T: Display + WasmTypeName + Into<RegisterEntry>,
        F: FnOnce(T) -> R,
        R: Into<RegisterEntry> + WasmTypeName,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {input_type}) (result {input_type})
                    {input_type}.const {input}
                    {input_type}.{wasm_op}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let result = engine.alloc_const(exec_op(input));
        let results = engine.alloc_provider_slice([Provider::from_immediate(result)]);
        let expected = [ExecInstruction::Return { results }];
        assert_func_bodies(&wasm, [expected]);
    }

    run_test("clz", 1, i32::leading_zeros);
    run_test("clz", 1, i64::leading_zeros);
    run_test("ctz", 1, i32::trailing_zeros);
    run_test("ctz", 1, i64::trailing_zeros);
    run_test("popcnt", 1, i32::count_ones);
    run_test("popcnt", 1, i64::count_ones);
    run_test(
        "extend8_s",
        1,
        <i32 as SignExtendFrom<i8>>::sign_extend_from,
    );
    run_test(
        "extend16_s",
        1,
        <i32 as SignExtendFrom<i16>>::sign_extend_from,
    );
    run_test(
        "extend8_s",
        1,
        <i64 as SignExtendFrom<i8>>::sign_extend_from,
    );
    run_test(
        "extend16_s",
        1,
        <i64 as SignExtendFrom<i16>>::sign_extend_from,
    );
    run_test(
        "extend16_s",
        1,
        <i64 as SignExtendFrom<i32>>::sign_extend_from,
    );
    run_test("abs", 1.0, |input| F32::from(input).abs());
    run_test("abs", 1.0, |input| F64::from(input).abs());
    run_test("neg", 1.0, |input| -F32::from(input));
    run_test("neg", 1.0, |input| -F64::from(input));
    run_test("ceil", 1.0, |input| F32::from(input).ceil());
    run_test("ceil", 1.0, |input| F64::from(input).ceil());
    run_test("floor", 1.0, |input| F32::from(input).floor());
    run_test("floor", 1.0, |input| F64::from(input).floor());
    run_test("trunc", 1.0, |input| F32::from(input).trunc());
    run_test("trunc", 1.0, |input| F64::from(input).trunc());
    run_test("nearest", 1.0, |input| F32::from(input).nearest());
    run_test("nearest", 1.0, |input| F64::from(input).nearest());
    run_test("sqrt", 1.0, |input| F32::from(input).sqrt());
    run_test("sqrt", 1.0, |input| F64::from(input).sqrt());
}
