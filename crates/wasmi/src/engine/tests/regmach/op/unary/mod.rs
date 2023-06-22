mod conversion;
mod op;

use super::*;
use crate::engine::{const_pool::ConstRef, tests::regmach::driver::TranslationTest};
use std::fmt::Display;
use wasmi_core::{UntypedValue, F32};

/// [`Display`] wrapper for `T` where `T` is a Wasm type.
pub struct DisplayWasm<T>(T);

macro_rules! impl_from {
    ( $( $ty:ty ),* $(,)? ) => {
        $(
            impl From<$ty> for DisplayWasm<$ty> {
                fn from(value: $ty) -> Self {
                    Self(value)
                }
            }
        )*
    };
}
impl_from!(i32, i64, f32, f64);

impl Display for DisplayWasm<i32> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for DisplayWasm<i64> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

macro_rules! impl_display_for_float {
    ( $float_ty:ty ) => {
        impl Display for DisplayWasm<$float_ty> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let value = self.0;
                if value.is_nan() && value.is_sign_positive() {
                    // Special rule required because Rust and Wasm have different NaN formats.
                    return write!(f, "nan");
                }
                if value.is_nan() && value.is_sign_negative() {
                    // Special rule required because Rust and Wasm have different NaN formats.
                    return write!(f, "-nan");
                }
                write!(f, "{}", value)
            }
        }
    };
}
impl_display_for_float!(f32);
impl_display_for_float!(f64);

pub trait WasmType: Copy + Display + Into<UntypedValue> + From<UntypedValue> {
    const NAME: &'static str;

    fn return_imm_instr(&self) -> Instruction;
}

impl WasmType for i32 {
    const NAME: &'static str = "i32";

    fn return_imm_instr(&self) -> Instruction {
        Instruction::return_imm32(*self)
    }
}

impl WasmType for i64 {
    const NAME: &'static str = "i64";

    fn return_imm_instr(&self) -> Instruction {
        match i32::try_from(*self) {
            Ok(value) => Instruction::return_i64imm32(value),
            Err(_) => Instruction::return_cref(0),
        }
    }
}

impl WasmType for f32 {
    const NAME: &'static str = "f32";

    fn return_imm_instr(&self) -> Instruction {
        Instruction::ReturnImm32 {
            value: Const32::from_f32(F32::from(*self)),
        }
    }
}

impl WasmType for f64 {
    const NAME: &'static str = "f64";

    fn return_imm_instr(&self) -> Instruction {
        Instruction::ReturnImm {
            value: ConstRef::from_u32(0),
        }
    }
}

/// Asserts that the unary Wasm operator `wasm_op` translates properly to a unary `wasmi` instruction.
fn conversion_reg<I, O>(
    wasm_op: &str,
    make_instr: fn(result: Register, input: Register) -> Instruction,
) where
    I: WasmType,
    O: WasmType,
{
    let param_ty = <I as WasmType>::NAME;
    let result_ty = <O as WasmType>::NAME;
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {param_ty}) (result {result_ty})
                local.get 0
                {result_ty}.{wasm_op}
            )
        )
    "#,
    ));
    let expected = [
        make_instr(Register::from_u16(1), Register::from_u16(0)),
        Instruction::return_reg(1),
    ];
    TranslationTest::new(wasm).expect_func(expected).run();
}

/// Asserts that the unary Wasm operator `wasm_op` translates properly to a unary `wasmi` instruction.
fn unary_reg<T>(wasm_op: &str, make_instr: fn(result: Register, input: Register) -> Instruction)
where
    T: WasmType,
{
    conversion_reg::<T, T>(wasm_op, make_instr)
}

/// Asserts that the unary Wasm operator `wasm_op` translates properly to a unary `wasmi` instruction.
fn conversion_imm<I, O>(wasm_op: &str, input: I, eval: fn(input: I) -> O)
where
    I: WasmType,
    O: WasmType,
    DisplayWasm<I>: From<I> + Display,
{
    let param_ty = <I as WasmType>::NAME;
    let result_ty = <O as WasmType>::NAME;
    let wasm_input = DisplayWasm::from(input);
    let wasm: Vec<u8> = wat2wasm(&format!(
        r#"
        (module
            (func (result {result_ty})
                {param_ty}.const {wasm_input}
                {result_ty}.{wasm_op}
            )
        )
    "#,
    ));
    let instr = <O as WasmType>::return_imm_instr(&eval(input));
    let mut testcase = TranslationTest::new(wasm);
    if let Instruction::ReturnImm { value } = &instr {
        testcase.expect_const(*value, eval(input));
    }
    testcase.expect_func([instr]).run();
}

/// Asserts that the unary Wasm operator `wasm_op` translates properly to a unary `wasmi` instruction.
fn unary_imm<T>(wasm_op: &str, input: T, eval: fn(input: T) -> T)
where
    T: WasmType,
    DisplayWasm<T>: From<T> + Display,
{
    conversion_imm::<T, T>(wasm_op, input, eval)
}
