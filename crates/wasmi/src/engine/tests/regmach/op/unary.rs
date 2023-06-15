use super::*;
use crate::engine::const_pool::ConstRef;
use std::fmt::Display;
use wasmi_core::F32;

pub trait WasmType: Display {
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
    assert_func_bodies(wasm, [expected]);
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
    let expected = [<T as WasmType>::return_imm_instr(&eval(input))];
    assert_func_bodies(wasm, [expected]);
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
        unary_imm::<f32>(OP_NAME, 42.0, f32::abs);
        unary_imm::<f32>(OP_NAME, -42.0, f32::abs);
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
        unary_imm::<f64>(OP_NAME, 42.0, f64::abs);
        unary_imm::<f64>(OP_NAME, -42.0, f64::abs);
    }
}
