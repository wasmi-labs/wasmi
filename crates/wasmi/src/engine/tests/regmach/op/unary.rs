use super::*;
use crate::engine::{const_pool::ConstRef, tests::regmach::driver::TranslationTest};
use std::fmt::Display;
use wasmi_core::{UntypedValue, F32};

pub trait WasmType: Copy + Display + Into<UntypedValue> {
    const NAME: &'static str;

    fn return_imm_instr(&self) -> Instruction;
}

impl WasmType for i32 {
    const NAME: &'static str = "i32";

    fn return_imm_instr(&self) -> Instruction {
        Instruction::ReturnImm32 {
            value: Const32::from_i32(*self),
        }
    }
}

impl WasmType for i64 {
    const NAME: &'static str = "i64";

    fn return_imm_instr(&self) -> Instruction {
        match i32::try_from(*self) {
            Ok(value) => Instruction::ReturnI64Imm32 {
                value: Const32::from_i32(value),
            },
            Err(_) => Instruction::ReturnImm {
                value: ConstRef::from_u32(0),
            },
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
fn unary_reg<T>(wasm_op: &str, make_instr: fn(result: Register, input: Register) -> Instruction)
where
    T: WasmType,
{
    let ty = <T as WasmType>::NAME;
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (param {ty}) (result {ty})
                local.get 0
                {ty}.{wasm_op}
            )
        )
    "#,
    ));
    let expected = [
        make_instr(Register::from_u16(1), Register::from_u16(0)),
        Instruction::ReturnReg {
            value: Register::from_u16(1),
        },
    ];
    TranslationTest::new(&wasm).expect_func(expected).run();
}

/// Asserts that the unary Wasm operator `wasm_op` translates properly to a unary `wasmi` instruction.
fn unary_imm<T>(wasm_op: &str, input: T, eval: fn(input: T) -> T)
where
    T: WasmType,
{
    let ty = <T as WasmType>::NAME;
    let wasm = wat2wasm(&format!(
        r#"
        (module
            (func (result {ty})
                {ty}.const {input}
                {ty}.{wasm_op}
            )
        )
    "#,
    ));
    let instr = <T as WasmType>::return_imm_instr(&eval(input));
    let mut testcase = TranslationTest::new(&wasm);
    match &instr {
        Instruction::ReturnImm { value } => {
            testcase.expect_const(*value, eval(input));
        }
        _ => {}
    }
    testcase.expect_func([instr]).run();
}

mod i32_clz {
    use super::*;

    #[test]
    fn reg() {
        unary_reg::<i32>("clz", Instruction::i32_clz);
    }

    #[test]
    fn imm() {
        unary_imm::<i32>("clz", 42, |input| input.leading_zeros() as _);
    }
}

mod i64_clz {
    use super::*;

    #[test]
    fn reg() {
        unary_reg::<i64>("clz", Instruction::i64_clz);
    }

    #[test]
    fn imm() {
        unary_imm::<i64>("clz", 42, |input| i64::from(input.leading_zeros()));
    }
}

mod i32_ctz {
    use super::*;

    #[test]
    fn reg() {
        unary_reg::<i32>("ctz", Instruction::i32_ctz);
    }

    #[test]
    fn imm() {
        unary_imm::<i32>("ctz", 42, |input| input.trailing_zeros() as _);
    }
}

mod i64_ctz {
    use super::*;

    #[test]
    fn reg() {
        unary_reg::<i64>("ctz", Instruction::i64_ctz);
    }

    #[test]
    fn imm() {
        unary_imm::<i64>("ctz", 42, |input| i64::from(input.trailing_zeros()));
    }
}

mod i32_popcnt {
    use super::*;

    #[test]
    fn reg() {
        unary_reg::<i32>("popcnt", Instruction::i32_popcnt);
    }

    #[test]
    fn imm() {
        unary_imm::<i32>("popcnt", 42, |input| input.count_ones() as _);
    }
}

mod i64_popcnt {
    use super::*;

    #[test]
    fn reg() {
        unary_reg::<i64>("popcnt", Instruction::i64_popcnt);
    }

    #[test]
    fn imm() {
        unary_imm::<i64>("popcnt", 42, |input| i64::from(input.count_ones()));
    }
}

mod f32_abs {
    use super::*;

    const OP_NAME: &str = "abs";

    #[test]
    fn reg() {
        unary_reg::<f32>(OP_NAME, Instruction::f32_abs);
    }

    #[test]
    fn imm() {
        unary_imm::<f32>(OP_NAME, 42.5, f32::abs);
        unary_imm::<f32>(OP_NAME, -42.5, f32::abs);
    }
}

mod f32_neg {
    use super::*;

    const OP_NAME: &str = "neg";

    #[test]
    fn reg() {
        unary_reg::<f32>(OP_NAME, Instruction::f32_neg);
    }

    #[test]
    fn imm() {
        use core::ops::Neg as _;
        unary_imm::<f32>(OP_NAME, 42.5, f32::neg);
        unary_imm::<f32>(OP_NAME, -42.5, f32::neg);
    }
}

mod f32_ceil {
    use super::*;

    const OP_NAME: &str = "ceil";

    #[test]
    fn reg() {
        unary_reg::<f32>(OP_NAME, Instruction::f32_ceil);
    }

    #[test]
    fn imm() {
        unary_imm::<f32>(OP_NAME, 42.5, f32::ceil);
        unary_imm::<f32>(OP_NAME, -42.5, f32::ceil);
    }
}

mod f32_floor {
    use super::*;

    const OP_NAME: &str = "floor";

    #[test]
    fn reg() {
        unary_reg::<f32>(OP_NAME, Instruction::f32_floor);
    }

    #[test]
    fn imm() {
        unary_imm::<f32>(OP_NAME, 42.5, f32::floor);
        unary_imm::<f32>(OP_NAME, -42.5, f32::floor);
    }
}

mod f32_trunc {
    use super::*;

    const OP_NAME: &str = "trunc";

    #[test]
    fn reg() {
        unary_reg::<f32>(OP_NAME, Instruction::f32_trunc);
    }

    #[test]
    fn imm() {
        unary_imm::<f32>(OP_NAME, 42.5, f32::trunc);
        unary_imm::<f32>(OP_NAME, -42.5, f32::trunc);
    }
}

mod f32_nearest {
    use super::*;
    use wasmi_core::UntypedValue;

    const OP_NAME: &str = "nearest";

    /// We simply use the `f32_nearest` implementation from the `wasmi_core` crate.
    ///
    /// # Note
    ///
    /// Rust currently does not ship with a proper rounding function for floats
    /// that has the same behavior as mandated by the WebAssembly specification.
    /// There is an issue to add a proper `round_ties_even` to Rust and we should
    /// use it once it is stabilized.
    ///
    /// More information here: https://github.com/rust-lang/rust/issues/96710
    fn f32_nearest(input: f32) -> f32 {
        f32::from(UntypedValue::f32_nearest(UntypedValue::from(input)))
    }

    #[test]
    fn reg() {
        unary_reg::<f32>(OP_NAME, Instruction::f32_nearest);
    }

    #[test]
    fn imm() {
        unary_imm::<f32>(OP_NAME, 42.5, f32_nearest);
        unary_imm::<f32>(OP_NAME, -42.5, f32_nearest);
    }
}

mod f32_sqrt {
    use super::*;

    const OP_NAME: &str = "sqrt";

    #[test]
    fn reg() {
        unary_reg::<f32>(OP_NAME, Instruction::f32_sqrt);
    }

    #[test]
    fn imm() {
        unary_imm::<f32>(OP_NAME, 42.5, f32::sqrt);
        unary_imm::<f32>(OP_NAME, -42.5, f32::sqrt);
    }
}

mod f64_abs {
    use super::*;

    const OP_NAME: &str = "abs";

    #[test]
    fn reg() {
        unary_reg::<f64>(OP_NAME, Instruction::f64_abs);
    }

    #[test]
    fn imm() {
        unary_imm::<f64>(OP_NAME, 42.5, f64::abs);
        unary_imm::<f64>(OP_NAME, -42.5, f64::abs);
    }
}

mod f64_neg {
    use super::*;

    const OP_NAME: &str = "neg";

    #[test]
    fn reg() {
        unary_reg::<f64>(OP_NAME, Instruction::f64_neg);
    }

    #[test]
    fn imm() {
        use core::ops::Neg as _;
        unary_imm::<f64>(OP_NAME, 42.5, f64::neg);
        unary_imm::<f64>(OP_NAME, -42.5, f64::neg);
    }
}

mod f64_ceil {
    use super::*;

    const OP_NAME: &str = "ceil";

    #[test]
    fn reg() {
        unary_reg::<f64>(OP_NAME, Instruction::f64_ceil);
    }

    #[test]
    fn imm() {
        unary_imm::<f64>(OP_NAME, 42.5, f64::ceil);
        unary_imm::<f64>(OP_NAME, -42.5, f64::ceil);
    }
}

mod f64_floor {
    use super::*;

    const OP_NAME: &str = "floor";

    #[test]
    fn reg() {
        unary_reg::<f64>(OP_NAME, Instruction::f64_floor);
    }

    #[test]
    fn imm() {
        unary_imm::<f64>(OP_NAME, 42.5, f64::floor);
        unary_imm::<f64>(OP_NAME, -42.5, f64::floor);
    }
}

mod f64_trunc {
    use super::*;

    const OP_NAME: &str = "trunc";

    #[test]
    fn reg() {
        unary_reg::<f64>(OP_NAME, Instruction::f64_trunc);
    }

    #[test]
    fn imm() {
        unary_imm::<f64>(OP_NAME, 42.5, f64::trunc);
        unary_imm::<f64>(OP_NAME, -42.5, f64::trunc);
    }
}

mod f64_nearest {
    use super::*;
    use wasmi_core::UntypedValue;

    const OP_NAME: &str = "nearest";

    /// We simply use the `f32_nearest` implementation from the `wasmi_core` crate.
    ///
    /// # Note
    ///
    /// Rust currently does not ship with a proper rounding function for floats
    /// that has the same behavior as mandated by the WebAssembly specification.
    /// There is an issue to add a proper `round_ties_even` to Rust and we should
    /// use it once it is stabilized.
    ///
    /// More information here: https://github.com/rust-lang/rust/issues/96710
    fn f64_nearest(input: f64) -> f64 {
        f64::from(UntypedValue::f64_nearest(UntypedValue::from(input)))
    }

    #[test]
    fn reg() {
        unary_reg::<f64>(OP_NAME, Instruction::f64_nearest);
    }

    #[test]
    fn imm() {
        unary_imm::<f64>(OP_NAME, 42.5, f64_nearest);
        unary_imm::<f64>(OP_NAME, -42.5, f64_nearest);
    }
}

mod f64_sqrt {
    use super::*;

    const OP_NAME: &str = "sqrt";

    #[test]
    fn reg() {
        unary_reg::<f64>(OP_NAME, Instruction::f64_sqrt);
    }

    #[test]
    fn imm() {
        unary_imm::<f64>(OP_NAME, 42.5, f64::sqrt);
        unary_imm::<f64>(OP_NAME, -42.5, f64::sqrt);
    }
}
