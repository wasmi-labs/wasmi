use std::fmt::Display;

use super::*;
use crate::{
    engine::{DedupProviderSlice, Instr, Target},
    engine2::{ExecInstruction, Offset, Provider, Register, RegisterEntry, WasmType},
    Engine,
};
use core::ops::{Shl, Shr};
use wasmi_core::{
    ArithmeticOps,
    ExtendInto,
    Float,
    Integer,
    SignExtendFrom,
    TrapCode,
    TruncateSaturateInto,
    TryTruncateInto,
    Value,
    WrapInto,
    F32,
    F64,
};

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

/// Creates a closure for constructing a `wasmi` load instruction.
macro_rules! load_op {
    ( $name:ident ) => {{
        |result, ptr, offset| ExecInstruction::$name {
            result,
            ptr,
            offset,
        }
    }};
}

/// Creates a closure for constructing a `wasmi` store instruction.
macro_rules! store_op {
    ( $name:ident ) => {{
        |ptr, offset, value| ExecInstruction::$name { ptr, offset, value }
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
/// - `{i32, i64, f32, f64}.sub`
/// - `{i32, i64, f32, f64}.mul`
/// - `{i32, i64}.div_s`
/// - `{i32, i64}.div_u`
/// - `{i32, i64}.rem_s`
/// - `{i32, i64}.rem_u`
/// - `{i32, i64}.shl`
/// - `{i32, i64}.shr_s`
/// - `{i32, i64}.shr_u`
/// - `{i32, i64}.rotl`
/// - `{i32, i64}.rotr`
/// - `{i32, i64}.and`
/// - `{i32, i64}.or`
/// - `{i32, i64}.xor`
/// - `{f32, f64}.div`
/// - `{f32, f64}.rem`
/// - `{f32, f64}.min`
/// - `{f32, f64}.max`
/// - `{f32, f64}.copysign`
#[test]
fn binary_simple() {
    fn test_register_register<T, F, R>(wasm_op: &str, make_op: F)
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

    fn test_register_const<T, F, R>(wasm_op: &str, make_op: F)
    where
        T: Display + WasmTypeName + Into<RegisterEntry> + One,
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
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let lhs = Register::from_inner(0);
        let rhs = Provider::from_immediate(engine.alloc_const(T::one()));
        let result = Register::from_inner(1);
        let results = engine.alloc_provider_slice([Provider::from_register(result)]);
        let expected = [
            make_op(result, lhs, rhs),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    fn run_test_bin<T, F>(wasm_op: &str, make_op: F)
    where
        T: Display + WasmTypeName + Into<RegisterEntry> + One,
        F: FnOnce(Register, Register, Provider) -> ExecInstruction + Copy,
    {
        test_register_register::<T, F, T>(wasm_op, make_op);
        test_register_const::<T, F, T>(wasm_op, make_op);
    }

    fn run_test_cmp<T, F>(wasm_op: &str, make_op: F)
    where
        T: Display + WasmTypeName + Into<RegisterEntry> + One,
        F: FnOnce(Register, Register, Provider) -> ExecInstruction + Copy,
    {
        test_register_register::<T, F, bool>(wasm_op, make_op);
        test_register_const::<T, F, bool>(wasm_op, make_op);
    }

    run_test_cmp::<i32, _>("eq", make_op!(I32Eq));
    run_test_cmp::<i64, _>("eq", make_op!(I64Eq));
    run_test_cmp::<i32, _>("ne", make_op!(I32Ne));
    run_test_cmp::<i64, _>("ne", make_op!(I64Ne));

    run_test_bin::<i32, _>("add", make_op!(I32Add));
    run_test_bin::<i64, _>("add", make_op!(I64Add));
    run_test_bin::<i32, _>("sub", make_op!(I32Sub));
    run_test_bin::<i64, _>("sub", make_op!(I64Sub));
    run_test_bin::<i32, _>("mul", make_op!(I32Mul));
    run_test_bin::<i64, _>("mul", make_op!(I64Mul));
    run_test_bin::<i32, _>("div_s", make_op!(I32DivS));
    run_test_bin::<i64, _>("div_s", make_op!(I64DivS));
    run_test_bin::<i32, _>("div_u", make_op!(I32DivU));
    run_test_bin::<i64, _>("div_u", make_op!(I64DivU));
    run_test_bin::<i32, _>("rem_s", make_op!(I32RemS));
    run_test_bin::<i64, _>("rem_s", make_op!(I64RemS));
    run_test_bin::<i32, _>("rem_u", make_op!(I32RemU));
    run_test_bin::<i64, _>("rem_u", make_op!(I64RemU));
    run_test_bin::<i32, _>("shl", make_op!(I32Shl));
    run_test_bin::<i64, _>("shl", make_op!(I64Shl));
    run_test_bin::<i32, _>("shr_s", make_op!(I32ShrS));
    run_test_bin::<i64, _>("shr_s", make_op!(I64ShrS));
    run_test_bin::<i32, _>("shr_u", make_op!(I32ShrU));
    run_test_bin::<i64, _>("shr_u", make_op!(I64ShrU));
    run_test_bin::<i32, _>("rotl", make_op!(I32Rotl));
    run_test_bin::<i64, _>("rotr", make_op!(I64Rotr));
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
    run_test_bin::<f32, _>("sub", make_op!(F32Sub));
    run_test_bin::<f64, _>("sub", make_op!(F64Sub));
    run_test_bin::<f32, _>("mul", make_op!(F32Mul));
    run_test_bin::<f64, _>("mul", make_op!(F64Mul));
    run_test_bin::<f32, _>("div", make_op!(F32Div));
    run_test_bin::<f64, _>("div", make_op!(F64Div));
    run_test_bin::<f32, _>("min", make_op!(F32Min));
    run_test_bin::<f64, _>("min", make_op!(F64Min));
    run_test_bin::<f32, _>("max", make_op!(F32Max));
    run_test_bin::<f64, _>("max", make_op!(F64Max));
    run_test_bin::<f32, _>("copysign", make_op!(F32Copysign));
    run_test_bin::<f64, _>("copysign", make_op!(F64Copysign));
}

/// Tests compilation of all commutative binary Wasm instructions.
///
/// # Note
///
/// This test specializes on cases where one of the inputs is a constant value
/// (e.g. `i32.const 1`) and the other a register input (e.g. `local.get 0`).
/// In this case the `wasmi` compiler unfortunately has to insert an artificial
/// `copy` instruction in between in order to be able to properly represent
/// the underlying instruction. This is due to the fact that due to performance
/// reasons the `lhs` operand of an instruction can only be a register and
/// never an immediate value unlike the right-hand side operand. Fortunately
/// having an immediate value as the left-hand operand is quite uncommon.
///
/// This includes the following Wasm instructions:
///
/// - `{i32, i64, f32, f64}.sub`
/// - `{i32, i64}.div_s`
/// - `{i32, i64}.div_u`
/// - `{i32, i64}.rem_s`
/// - `{i32, i64}.rem_u`
/// - `{i32, i64}.shl`
/// - `{i32, i64}.shr_s`
/// - `{i32, i64}.shr_u`
/// - `{i32, i64}.rotl`
/// - `{i32, i64}.rotr`
/// - `{f32, f64}.div`
/// - `{f32, f64}.copysign`
#[test]
fn binary_const_register() {
    fn test_const_register<T, F>(wasm_op: &str, make_op: F)
    where
        T: Display + WasmTypeName + One + Into<RegisterEntry>,
        F: FnOnce(Register, Register, Provider) -> ExecInstruction,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let output_type = <T as WasmTypeName>::NAME;
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
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let input = Provider::from_immediate(engine.alloc_const(T::one()));
        let rhs = Register::from_inner(0);
        let result = Register::from_inner(1);
        let results = engine.alloc_provider_slice([Provider::from_register(result)]);
        let expected = [
            ExecInstruction::Copy { result, input },
            make_op(result, result, rhs.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    test_const_register::<i32, _>("sub", make_op!(I32Sub));
    test_const_register::<i64, _>("sub", make_op!(I64Sub));
    test_const_register::<i32, _>("div_s", make_op!(I32DivS));
    test_const_register::<i64, _>("div_s", make_op!(I64DivS));
    test_const_register::<i32, _>("div_u", make_op!(I32DivU));
    test_const_register::<i64, _>("div_u", make_op!(I64DivU));
    test_const_register::<i32, _>("rem_s", make_op!(I32RemS));
    test_const_register::<i64, _>("rem_s", make_op!(I64RemS));
    test_const_register::<i32, _>("rem_u", make_op!(I32RemU));
    test_const_register::<i64, _>("rem_u", make_op!(I64RemU));
    test_const_register::<i32, _>("shl", make_op!(I32Shl));
    test_const_register::<i64, _>("shl", make_op!(I64Shl));
    test_const_register::<i32, _>("shr_s", make_op!(I32ShrS));
    test_const_register::<i64, _>("shr_s", make_op!(I64ShrS));
    test_const_register::<i32, _>("shr_u", make_op!(I32ShrU));
    test_const_register::<i64, _>("shr_u", make_op!(I64ShrU));
    test_const_register::<i32, _>("rotl", make_op!(I32Rotl));
    test_const_register::<i64, _>("rotl", make_op!(I64Rotl));
    test_const_register::<i32, _>("rotr", make_op!(I32Rotr));
    test_const_register::<i64, _>("rotr", make_op!(I64Rotr));
    test_const_register::<f32, _>("sub", make_op!(F32Sub));
    test_const_register::<f64, _>("sub", make_op!(F64Sub));
    test_const_register::<f32, _>("div", make_op!(F32Div));
    test_const_register::<f64, _>("div", make_op!(F64Div));
    test_const_register::<f32, _>("copysign", make_op!(F32Copysign));
    test_const_register::<f64, _>("copysign", make_op!(F64Copysign));
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
fn binary_const_register_commutative() {
    fn test_const_register<T, F, R>(wasm_op: &str, make_op: F)
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
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let rhs = engine.alloc_const(T::one());
        let result = Register::from_inner(1);
        let results = engine.alloc_provider_slice([Provider::from_register(result)]);
        let expected = [
            make_op(Register::from_inner(1), Register::from_inner(0), rhs.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    fn run_test_bin<T, F>(wasm_op: &str, make_op: F)
    where
        T: Display + Into<RegisterEntry> + WasmTypeName + One,
        F: FnOnce(Register, Register, Provider) -> ExecInstruction + Copy,
    {
        test_const_register::<T, F, T>(wasm_op, make_op);
    }

    fn run_test_cmp<T, F>(wasm_op: &str, make_op: F)
    where
        T: Display + Into<RegisterEntry> + WasmTypeName + One,
        F: FnOnce(Register, Register, Provider) -> ExecInstruction + Copy,
    {
        test_const_register::<T, F, bool>(wasm_op, make_op);
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

/// The expected outcome of a fallible constant evaluation.
#[derive(Debug, Copy, Clone)]
pub enum Outcome {
    /// The instruction evaluation resulted in a proper value.
    Eval,
    /// The instruction evaluation resulted in a trap.
    Trap,
}

/// Tests compilation of all fallible binary Wasm instructions.
///
/// # Note
///
/// This test specializes on cases where both inputs are constant values.
/// In this case the `wasmi` compiler will directly evaluate the results.
///
/// This includes the following Wasm instructions:
///
/// - `{i32, i64}.div_s`
/// - `{i32, i64}.div_u`
/// - `{i32, i64}.rem_s`
/// - `{i32, i64}.rem_u`
#[test]
fn binary_const_const_fallible() {
    fn test_const_const<T, E>(wasm_op: &str, outcome: Outcome, lhs: T, rhs: T, exec_op: E)
    where
        T: Display + WasmTypeName + Into<RegisterEntry>,
        E: FnOnce(T, T) -> Result<T, TrapCode>,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let output_type = <T as WasmTypeName>::NAME;
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
        let expected = match exec_op(lhs, rhs) {
            Ok(result) => {
                assert!(matches!(outcome, Outcome::Eval));
                let result = engine.alloc_const(result.into());
                let results = engine.alloc_provider_slice([Provider::from(result)]);
                [ExecInstruction::Return { results }]
            }
            Err(trap_code) => {
                assert!(matches!(outcome, Outcome::Trap));
                [ExecInstruction::Trap { trap_code }]
            }
        };
        assert_func_bodies(&wasm, [expected]);
    }

    test_const_const::<i32, _>("div_s", Outcome::Eval, 1, 2, |lhs, rhs| lhs.div(rhs));
    test_const_const::<i32, _>("div_s", Outcome::Trap, 1, 0, |lhs, rhs| lhs.div(rhs));
    test_const_const::<i64, _>("div_s", Outcome::Eval, 1, 2, |lhs, rhs| lhs.div(rhs));
    test_const_const::<i64, _>("div_s", Outcome::Trap, 1, 0, |lhs, rhs| lhs.div(rhs));

    test_const_const::<u32, _>("div_u", Outcome::Eval, 1, 2, |lhs, rhs| lhs.div(rhs));
    test_const_const::<u32, _>("div_u", Outcome::Trap, 1, 0, |lhs, rhs| lhs.div(rhs));
    test_const_const::<u64, _>("div_u", Outcome::Eval, 1, 2, |lhs, rhs| lhs.div(rhs));
    test_const_const::<u64, _>("div_u", Outcome::Trap, 1, 0, |lhs, rhs| lhs.div(rhs));

    test_const_const::<i32, _>("rem_s", Outcome::Eval, 1, 2, |lhs, rhs| lhs.rem(rhs));
    test_const_const::<i32, _>("rem_s", Outcome::Trap, 1, 0, |lhs, rhs| lhs.rem(rhs));
    test_const_const::<i64, _>("rem_s", Outcome::Eval, 1, 2, |lhs, rhs| lhs.rem(rhs));
    test_const_const::<i64, _>("rem_s", Outcome::Trap, 1, 0, |lhs, rhs| lhs.rem(rhs));

    test_const_const::<u32, _>("rem_u", Outcome::Eval, 1, 2, |lhs, rhs| lhs.rem(rhs));
    test_const_const::<u32, _>("rem_u", Outcome::Trap, 1, 0, |lhs, rhs| lhs.rem(rhs));
    test_const_const::<u64, _>("rem_u", Outcome::Eval, 1, 2, |lhs, rhs| lhs.rem(rhs));
    test_const_const::<u64, _>("rem_u", Outcome::Trap, 1, 0, |lhs, rhs| lhs.rem(rhs));
}

/// Tests compilation of all infallible binary Wasm instructions.
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
/// - `{i32, i64, f32, f64}.sub`
/// - `{i32, i64, f32, f64}.mul`
/// - `{i32, i64}.shl`
/// - `{i32, i64}.shr_s`
/// - `{i32, i64}.shr_u`
/// - `{i32, i64}.rotl`
/// - `{i32, i64}.rotr`
/// - `{i32, i64}.and`
/// - `{i32, i64}.or`
/// - `{i32, i64}.xor`
/// - `{f32, f64}.min`
/// - `{f32, f64}.max`
/// - `{i32, i64}.copysign`
#[test]
fn binary_const_const_infallible() {
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
    run_test_bin::<i32, _>("sub", 1, 2, |lhs, rhs| lhs.wrapping_sub(rhs));
    run_test_bin::<i64, _>("sub", 1, 2, |lhs, rhs| lhs.wrapping_sub(rhs));
    run_test_bin::<i32, _>("mul", 1, 2, |lhs, rhs| lhs.wrapping_mul(rhs));
    run_test_bin::<i64, _>("mul", 1, 2, |lhs, rhs| lhs.wrapping_mul(rhs));
    run_test_bin::<i32, _>("shl", 1, 2, |lhs, rhs| lhs.shl(rhs & 0x1F));
    run_test_bin::<i64, _>("shl", 1, 2, |lhs, rhs| lhs.shl(rhs & 0x3F));
    run_test_bin::<i32, _>("shr_s", 1, 2, |lhs, rhs| lhs.shr(rhs & 0x1F));
    run_test_bin::<i64, _>("shr_s", 1, 2, |lhs, rhs| lhs.shr(rhs & 0x3F));
    run_test_bin::<u32, _>("shr_u", 1, 2, |lhs, rhs| lhs.shr(rhs & 0x1F));
    run_test_bin::<u64, _>("shr_u", 1, 2, |lhs, rhs| lhs.shr(rhs & 0x3F));
    run_test_bin::<i32, _>("rotl", 1, 2, |lhs, rhs| lhs.rotl(rhs));
    run_test_bin::<i64, _>("rotl", 1, 2, |lhs, rhs| lhs.rotl(rhs));
    run_test_bin::<i32, _>("rotr", 1, 2, |lhs, rhs| lhs.rotr(rhs));
    run_test_bin::<i64, _>("rotr", 1, 2, |lhs, rhs| lhs.rotr(rhs));
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
    run_test_bin::<f32, _>("sub", 1.0, 2.0, |lhs, rhs| {
        (F32::from(lhs) - F32::from(rhs)).into()
    });
    run_test_bin::<f64, _>("sub", 1.0, 2.0, |lhs, rhs| {
        (F64::from(lhs) - F64::from(rhs)).into()
    });
    run_test_bin::<f32, _>("mul", 1.0, 2.0, |lhs, rhs| {
        (F32::from(lhs) * F32::from(rhs)).into()
    });
    run_test_bin::<f64, _>("mul", 1.0, 2.0, |lhs, rhs| {
        (F64::from(lhs) * F64::from(rhs)).into()
    });
    run_test_bin::<f32, _>("div", 1.0, 2.0, |lhs, rhs| {
        (F32::from(lhs) / F32::from(rhs)).into()
    });
    run_test_bin::<f64, _>("div", 1.0, 2.0, |lhs, rhs| {
        (F64::from(lhs) / F64::from(rhs)).into()
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
    run_test_bin::<f32, _>("copysign", 1.0, 2.0, |lhs, rhs| {
        F32::from(lhs).copysign(F32::from(rhs)).into()
    });
    run_test_bin::<f64, _>("copysign", 1.0, 2.0, |lhs, rhs| {
        F64::from(lhs).copysign(F64::from(rhs)).into()
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
    fn test<T, R, F>(wasm_op: &str, make_op: F)
    where
        T: WasmTypeName,
        R: WasmTypeName,
        F: FnOnce(Register, Register) -> ExecInstruction,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let result_type = <R as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {input_type}) (result {result_type})
                    local.get 0
                    {result_type}.{wasm_op}
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

    fn test_unary<T, F>(wasm_op: &str, make_op: F)
    where
        T: WasmTypeName,
        F: FnOnce(Register, Register) -> ExecInstruction,
    {
        test::<T, T, F>(wasm_op, make_op)
    }

    test_unary::<i32, _>("clz", make_op2!(I32Clz));
    test_unary::<i64, _>("clz", make_op2!(I64Clz));
    test_unary::<i32, _>("ctz", make_op2!(I32Ctz));
    test_unary::<i64, _>("ctz", make_op2!(I64Ctz));
    test_unary::<i32, _>("popcnt", make_op2!(I32Popcnt));
    test_unary::<i64, _>("popcnt", make_op2!(I64Popcnt));
    test_unary::<i32, _>("extend8_s", make_op2!(I32Extend8S));
    test_unary::<i64, _>("extend8_s", make_op2!(I64Extend8S));
    test_unary::<i32, _>("extend16_s", make_op2!(I32Extend16S));
    test_unary::<i64, _>("extend16_s", make_op2!(I64Extend16S));
    test_unary::<i64, _>("extend32_s", make_op2!(I64Extend32S));
    test_unary::<f32, _>("abs", make_op2!(F32Abs));
    test_unary::<f64, _>("abs", make_op2!(F64Abs));
    test_unary::<f32, _>("neg", make_op2!(F32Neg));
    test_unary::<f64, _>("neg", make_op2!(F64Neg));
    test_unary::<f32, _>("ceil", make_op2!(F32Ceil));
    test_unary::<f64, _>("ceil", make_op2!(F64Ceil));
    test_unary::<f32, _>("floor", make_op2!(F32Floor));
    test_unary::<f64, _>("floor", make_op2!(F64Floor));
    test_unary::<f32, _>("trunc", make_op2!(F32Trunc));
    test_unary::<f64, _>("trunc", make_op2!(F64Trunc));
    test_unary::<f32, _>("nearest", make_op2!(F32Nearest));
    test_unary::<f64, _>("nearest", make_op2!(F64Nearest));
    test_unary::<f32, _>("sqrt", make_op2!(F32Sqrt));
    test_unary::<f64, _>("sqrt", make_op2!(F64Sqrt));

    fn test_conversion<From, Into, F>(wasm_op: &str, make_op: F)
    where
        From: WasmTypeName,
        Into: WasmTypeName,
        F: FnOnce(Register, Register) -> ExecInstruction,
    {
        test::<From, Into, F>(wasm_op, make_op)
    }

    test::<i64, i32, _>("wrap_i64", make_op2!(I32WrapI64));
    test::<F32, i32, _>("trunc_f32_s", make_op2!(I32TruncSF32));
    test::<F32, u32, _>("trunc_f32_u", make_op2!(I32TruncUF32));
    test::<F64, i32, _>("trunc_f64_s", make_op2!(I32TruncSF64));
    test::<F64, u32, _>("trunc_f64_u", make_op2!(I32TruncUF64));
    test::<i32, i64, _>("extend_i32_s", make_op2!(I64ExtendSI32));
    test::<u32, i64, _>("extend_i32_u", make_op2!(I64ExtendUI32));
    test::<F32, i64, _>("trunc_f32_s", make_op2!(I64TruncSF32));
    test::<F32, u64, _>("trunc_f32_u", make_op2!(I64TruncUF32));
    test::<F64, i64, _>("trunc_f64_s", make_op2!(I64TruncSF64));
    test::<F64, u64, _>("trunc_f64_u", make_op2!(I64TruncUF64));
    test::<i32, F32, _>("convert_i32_s", make_op2!(F32ConvertSI32));
    test::<u32, F32, _>("convert_i32_u", make_op2!(F32ConvertUI32));
    test::<i64, F32, _>("convert_i64_s", make_op2!(F32ConvertSI64));
    test::<u64, F32, _>("convert_i64_u", make_op2!(F32ConvertUI64));
    test::<F64, F32, _>("demote_f64", make_op2!(F32DemoteF64));
    test::<i32, F64, _>("convert_i32_s", make_op2!(F64ConvertSI32));
    test::<u32, F64, _>("convert_i32_u", make_op2!(F64ConvertUI32));
    test::<i64, F64, _>("convert_i64_s", make_op2!(F64ConvertSI64));
    test::<u64, F64, _>("convert_i64_u", make_op2!(F64ConvertUI64));
    test::<F32, F64, _>("promote_f32", make_op2!(F64PromoteF32));
    test::<F32, i32, _>("trunc_sat_f32_s", make_op2!(I32TruncSatF32S));
    test::<F32, u32, _>("trunc_sat_f32_u", make_op2!(I32TruncSatF32U));
    test::<F64, i32, _>("trunc_sat_f64_s", make_op2!(I32TruncSatF64S));
    test::<F64, u32, _>("trunc_sat_f64_u", make_op2!(I32TruncSatF64U));
    test::<F32, i64, _>("trunc_sat_f32_s", make_op2!(I64TruncSatF32S));
    test::<F32, u64, _>("trunc_sat_f32_u", make_op2!(I64TruncSatF32U));
    test::<F64, i64, _>("trunc_sat_f64_s", make_op2!(I64TruncSatF64S));
    test::<F64, u64, _>("trunc_sat_f64_u", make_op2!(I64TruncSatF64U));
}

/// Tests translation of all unary Wasm instructions.
///
/// # Note
///
/// In this test all Wasm functions have a constant input (e.g. via `i32.const`).
///
/// This tests the following unary Wasm instructions:
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
///
/// And also this tests the following Wasm conversion instructions:
///
/// - `i32.wrap_i64`
/// - `i64.extend_i32_s`
/// - `i64.extend_i32_u`
/// - `{f32, f64}.convert_i32_s`
/// - `{f32, f64}.convert_i32_u`
/// - `{f32, f64}.convert_i64_s`
/// - `{f32, f64}.convert_i64_u`
/// - `f32.demote_f64`
/// - `f64.promote_f32`
/// - `{i32, i64}.trunc_sat_f32_s`
/// - `{i32, i64}.trunc_sat_f32_u`
/// - `{i32, i64}.trunc_sat_f64_s`
/// - `{i32, i64}.trunc_sat_f64_u`
#[test]
fn unary_const_infallible() {
    fn test<T, R, F>(wasm_op: &str, input: T, exec_op: F)
    where
        T: Display + WasmTypeName + Into<RegisterEntry>,
        F: FnOnce(T) -> R,
        R: Into<RegisterEntry> + WasmTypeName,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let result_type = <R as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {input_type}) (result {result_type})
                    {input_type}.const {input}
                    {result_type}.{wasm_op}
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

    fn test_unary<T, F>(wasm_op: &str, input: T, exec_op: F)
    where
        T: Display + WasmTypeName + Into<RegisterEntry>,
        F: FnOnce(T) -> T,
    {
        test::<T, T, F>(wasm_op, input, exec_op)
    }

    test_unary("clz", 1, <i32 as Integer<i32>>::leading_zeros);
    test_unary("clz", 1, <i64 as Integer<i64>>::leading_zeros);
    test_unary("ctz", 1, <i32 as Integer<i32>>::trailing_zeros);
    test_unary("ctz", 1, <i64 as Integer<i64>>::trailing_zeros);
    test_unary("popcnt", 1, <i32 as Integer<i32>>::count_ones);
    test_unary("popcnt", 1, <i64 as Integer<i64>>::count_ones);
    test_unary(
        "extend8_s",
        1,
        <i32 as SignExtendFrom<i8>>::sign_extend_from,
    );
    test_unary(
        "extend16_s",
        1,
        <i32 as SignExtendFrom<i16>>::sign_extend_from,
    );
    test_unary(
        "extend8_s",
        1,
        <i64 as SignExtendFrom<i8>>::sign_extend_from,
    );
    test_unary(
        "extend16_s",
        1,
        <i64 as SignExtendFrom<i16>>::sign_extend_from,
    );
    test_unary(
        "extend16_s",
        1,
        <i64 as SignExtendFrom<i32>>::sign_extend_from,
    );
    test("abs", 1.0, |input| F32::from(input).abs());
    test("abs", 1.0, |input| F64::from(input).abs());
    test("neg", 1.0, |input| -F32::from(input));
    test("neg", 1.0, |input| -F64::from(input));
    test("ceil", 1.0, |input| F32::from(input).ceil());
    test("ceil", 1.0, |input| F64::from(input).ceil());
    test("floor", 1.0, |input| F32::from(input).floor());
    test("floor", 1.0, |input| F64::from(input).floor());
    test("trunc", 1.0, |input| F32::from(input).trunc());
    test("trunc", 1.0, |input| F64::from(input).trunc());
    test("nearest", 1.0, |input| F32::from(input).nearest());
    test("nearest", 1.0, |input| F64::from(input).nearest());
    test("sqrt", 1.0, |input| F32::from(input).sqrt());
    test("sqrt", 1.0, |input| F64::from(input).sqrt());

    fn test_f32<R, F>(wasm_op: &str, input: f32, exec_op: F)
    where
        F: FnOnce(F32) -> R,
        R: Into<RegisterEntry> + WasmTypeName,
    {
        test::<f32, R, _>(wasm_op, input, |input| exec_op(F32::from(input)))
    }

    fn test_f64<R, F>(wasm_op: &str, input: f64, exec_op: F)
    where
        F: FnOnce(F64) -> R,
        R: Into<RegisterEntry> + WasmTypeName,
    {
        test::<f64, R, _>(wasm_op, input, |input| exec_op(F64::from(input)))
    }

    test::<i64, i32, _>("wrap_i64", 1, <i64 as WrapInto<i32>>::wrap_into);

    test::<i32, i64, _>("extend_i32_s", 1, <i32 as ExtendInto<i64>>::extend_into);
    test::<u32, i64, _>("extend_i32_u", 1, <u32 as ExtendInto<i64>>::extend_into);

    test::<i32, F32, _>("convert_i32_s", 1, <i32 as ExtendInto<F32>>::extend_into);
    test::<u32, F32, _>("convert_i32_u", 1, <u32 as ExtendInto<F32>>::extend_into);
    test::<i64, F32, _>("convert_i64_s", 1, <i64 as WrapInto<F32>>::wrap_into);
    test::<u64, F32, _>("convert_i64_u", 1, <u64 as WrapInto<F32>>::wrap_into);
    test::<f64, f32, _>("demote_f64", 1.0, |input| input.wrap_into());
    test::<i32, F64, _>("convert_i32_s", 1, <i32 as ExtendInto<F64>>::extend_into);
    test::<u32, F64, _>("convert_i32_u", 1, <u32 as ExtendInto<F64>>::extend_into);
    test::<i64, F64, _>("convert_i64_s", 1, <i64 as ExtendInto<F64>>::extend_into);
    test::<u64, F64, _>("convert_i64_u", 1, <u64 as ExtendInto<F64>>::extend_into);
    test::<f32, f64, _>("promote_f32", 1.0, |input| input.extend_into());
    test::<f32, i32, _>(
        "trunc_sat_f32_s",
        1.0,
        <f32 as TruncateSaturateInto<i32>>::truncate_saturate_into,
    );
    test::<f32, u32, _>(
        "trunc_sat_f32_u",
        1.0,
        <f32 as TruncateSaturateInto<u32>>::truncate_saturate_into,
    );
    test::<f64, i32, _>(
        "trunc_sat_f64_s",
        1.0,
        <f64 as TruncateSaturateInto<i32>>::truncate_saturate_into,
    );
    test::<f64, u32, _>(
        "trunc_sat_f64_u",
        1.0,
        <f64 as TruncateSaturateInto<u32>>::truncate_saturate_into,
    );
    test::<f32, i64, _>(
        "trunc_sat_f32_s",
        1.0,
        <f32 as TruncateSaturateInto<i64>>::truncate_saturate_into,
    );
    test::<f32, u64, _>(
        "trunc_sat_f32_u",
        1.0,
        <f32 as TruncateSaturateInto<u64>>::truncate_saturate_into,
    );
    test::<f64, i64, _>(
        "trunc_sat_f64_s",
        1.0,
        <f64 as TruncateSaturateInto<i64>>::truncate_saturate_into,
    );
    test::<f64, u64, _>(
        "trunc_sat_f64_u",
        1.0,
        <f64 as TruncateSaturateInto<u64>>::truncate_saturate_into,
    );
}

#[test]
fn unary_const_fallible() {
    fn test<T, R, F>(wasm_op: &str, outcome: Outcome, input: T, exec_op: F)
    where
        T: Display + WasmTypeName + Into<RegisterEntry>,
        F: FnOnce(T) -> Result<R, TrapCode>,
        R: Into<RegisterEntry> + WasmTypeName,
    {
        let input_type = <T as WasmTypeName>::NAME;
        let result_type = <R as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (func (export "call") (param {input_type}) (result {result_type})
                    {input_type}.const {input}
                    {result_type}.{wasm_op}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let expected = match exec_op(input) {
            Ok(result) => {
                assert!(matches!(outcome, Outcome::Eval));
                let result = engine.alloc_const(result);
                let results = engine.alloc_provider_slice([Provider::from_immediate(result)]);
                [ExecInstruction::Return { results }]
            }
            Err(trap_code) => {
                assert!(matches!(outcome, Outcome::Trap));
                [ExecInstruction::Trap { trap_code }]
            }
        };
        assert_func_bodies(&wasm, [expected]);
    }

    fn test_f32<R, F>(wasm_op: &str, outcome: Outcome, input: f32, exec_op: F)
    where
        F: FnOnce(F32) -> Result<R, TrapCode>,
        R: Into<RegisterEntry> + WasmTypeName,
    {
        test::<f32, R, _>(wasm_op, outcome, input, |input| exec_op(F32::from(input)))
    }

    fn test_f64<R, F>(wasm_op: &str, outcome: Outcome, input: f64, exec_op: F)
    where
        F: FnOnce(F64) -> Result<R, TrapCode>,
        R: Into<RegisterEntry> + WasmTypeName,
    {
        test::<f64, R, _>(wasm_op, outcome, input, |input| exec_op(F64::from(input)))
    }

    test_f32::<i32, _>(
        "trunc_f32_s",
        Outcome::Eval,
        1.0,
        <F32 as TryTruncateInto<i32, TrapCode>>::try_truncate_into,
    );
    test_f32::<u32, _>(
        "trunc_f32_u",
        Outcome::Eval,
        1.0,
        <F32 as TryTruncateInto<u32, TrapCode>>::try_truncate_into,
    );
    test_f64::<i32, _>(
        "trunc_f64_s",
        Outcome::Eval,
        1.0,
        <F64 as TryTruncateInto<i32, TrapCode>>::try_truncate_into,
    );
    test_f64::<u32, _>(
        "trunc_f64_u",
        Outcome::Eval,
        1.0,
        <F64 as TryTruncateInto<u32, TrapCode>>::try_truncate_into,
    );
    test_f32::<i64, _>(
        "trunc_f32_s",
        Outcome::Eval,
        1.0,
        <F32 as TryTruncateInto<i64, TrapCode>>::try_truncate_into,
    );
    test_f32::<u64, _>(
        "trunc_f32_u",
        Outcome::Eval,
        1.0,
        <F32 as TryTruncateInto<u64, TrapCode>>::try_truncate_into,
    );
    test_f64::<i64, _>(
        "trunc_f64_s",
        Outcome::Eval,
        1.0,
        <F64 as TryTruncateInto<i64, TrapCode>>::try_truncate_into,
    );
    test_f64::<u64, _>(
        "trunc_f64_u",
        Outcome::Eval,
        1.0,
        <F64 as TryTruncateInto<u64, TrapCode>>::try_truncate_into,
    );

    test_f32::<i32, _>(
        "trunc_f32_s",
        Outcome::Trap,
        f32::MAX,
        <F32 as TryTruncateInto<i32, TrapCode>>::try_truncate_into,
    );
    test_f32::<u32, _>(
        "trunc_f32_u",
        Outcome::Trap,
        f32::MAX,
        <F32 as TryTruncateInto<u32, TrapCode>>::try_truncate_into,
    );
    test_f64::<i32, _>(
        "trunc_f64_s",
        Outcome::Trap,
        f64::MAX,
        <F64 as TryTruncateInto<i32, TrapCode>>::try_truncate_into,
    );
    test_f64::<u32, _>(
        "trunc_f64_u",
        Outcome::Trap,
        f64::MAX,
        <F64 as TryTruncateInto<u32, TrapCode>>::try_truncate_into,
    );
    test_f32::<i64, _>(
        "trunc_f32_s",
        Outcome::Trap,
        f32::MAX,
        <F32 as TryTruncateInto<i64, TrapCode>>::try_truncate_into,
    );
    test_f32::<u64, _>(
        "trunc_f32_u",
        Outcome::Trap,
        f32::MAX,
        <F32 as TryTruncateInto<u64, TrapCode>>::try_truncate_into,
    );
    test_f64::<i64, _>(
        "trunc_f64_s",
        Outcome::Trap,
        f64::MAX,
        <F64 as TryTruncateInto<i64, TrapCode>>::try_truncate_into,
    );
    test_f64::<u64, _>(
        "trunc_f64_u",
        Outcome::Trap,
        f64::MAX,
        <F64 as TryTruncateInto<u64, TrapCode>>::try_truncate_into,
    );
}

#[test]
fn load_from_register() {
    fn test<T, F>(load_op: &str, offset: u32, make_op: F)
    where
        T: Display + WasmTypeName + Into<RegisterEntry>,
        F: FnOnce(Register, Register, Offset) -> ExecInstruction,
    {
        let load_type = <T as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (memory 1)
                (func (export "call") (param i32) (result {load_type})
                    local.get 0
                    {load_type}.{load_op} 0 offset={offset}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let ptr = Register::from_inner(0);
        let result = Register::from_inner(1);
        let results = engine.alloc_provider_slice([Provider::from_register(result)]);
        let expected = [
            make_op(result, ptr, offset.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    test::<i32, _>("load", 42, load_op!(I32Load));
    test::<i64, _>("load", 42, load_op!(I64Load));
    test::<f32, _>("load", 42, load_op!(F32Load));
    test::<f64, _>("load", 42, load_op!(F64Load));

    test::<i32, _>("load8_s", 42, load_op!(I32Load8S));
    test::<i32, _>("load16_s", 42, load_op!(I32Load16S));
    test::<i64, _>("load8_s", 42, load_op!(I64Load8S));
    test::<i64, _>("load16_s", 42, load_op!(I64Load16S));
    test::<i64, _>("load32_s", 42, load_op!(I64Load32S));

    test::<i32, _>("load8_u", 42, load_op!(I32Load8U));
    test::<i32, _>("load16_u", 42, load_op!(I32Load16U));
    test::<i64, _>("load8_u", 42, load_op!(I64Load8U));
    test::<i64, _>("load16_u", 42, load_op!(I64Load16U));
    test::<i64, _>("load32_u", 42, load_op!(I64Load32U));
}

#[test]
fn load_from_const() {
    fn test<T, F>(load_op: &str, offset: u32, make_op: F)
    where
        T: Display + WasmTypeName + Into<RegisterEntry>,
        F: FnOnce(Register, Register, Offset) -> ExecInstruction,
    {
        let load_type = <T as WasmTypeName>::NAME;
        let wasm = wat2wasm(&format!(
            r#"
            (module
                (memory 1)
                (func (export "call") (result {load_type})
                    i32.const 100
                    {load_type}.{load_op} 0 offset={offset}
                )
            )
        "#
        ));
        let module = create_module(&wasm[..]);
        let engine = module.engine();
        let const_ptr = engine.alloc_const(100);
        let result = Register::from_inner(0);
        let results = engine.alloc_provider_slice([Provider::from_register(result)]);
        let expected = [
            ExecInstruction::Copy {
                result,
                input: const_ptr.into(),
            },
            make_op(result, result, offset.into()),
            ExecInstruction::Return { results },
        ];
        assert_func_bodies_for_module(&module, [expected]);
    }

    test::<i32, _>("load", 42, load_op!(I32Load));
    test::<i64, _>("load", 42, load_op!(I64Load));
    test::<f32, _>("load", 42, load_op!(F32Load));
    test::<f64, _>("load", 42, load_op!(F64Load));

    test::<i32, _>("load8_s", 42, load_op!(I32Load8S));
    test::<i32, _>("load16_s", 42, load_op!(I32Load16S));
    test::<i64, _>("load8_s", 42, load_op!(I64Load8S));
    test::<i64, _>("load16_s", 42, load_op!(I64Load16S));
    test::<i64, _>("load32_s", 42, load_op!(I64Load32S));

    test::<i32, _>("load8_u", 42, load_op!(I32Load8U));
    test::<i32, _>("load16_u", 42, load_op!(I32Load16U));
    test::<i64, _>("load8_u", 42, load_op!(I64Load8U));
    test::<i64, _>("load16_u", 42, load_op!(I64Load16U));
    test::<i64, _>("load32_u", 42, load_op!(I64Load32U));
}
