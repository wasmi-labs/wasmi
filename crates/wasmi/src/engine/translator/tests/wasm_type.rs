use crate::core::ValType;

use crate::{
    core::UntypedVal,
    ir::{Const32, Instruction, Reg},
};
use core::fmt::Display;

pub trait WasmTy: Copy + Display + From<u16> + Into<UntypedVal> + From<UntypedVal> {
    const NAME: &'static str;
    const VALUE_TYPE: ValType;

    fn return_imm_instr(&self) -> Instruction;
}

impl WasmTy for u32 {
    const NAME: &'static str = "i32";
    const VALUE_TYPE: ValType = ValType::I32;

    fn return_imm_instr(&self) -> Instruction {
        Instruction::return_imm32(*self)
    }
}

impl WasmTy for i32 {
    const NAME: &'static str = "i32";
    const VALUE_TYPE: ValType = ValType::I32;

    fn return_imm_instr(&self) -> Instruction {
        Instruction::return_imm32(*self)
    }
}

impl WasmTy for u64 {
    const NAME: &'static str = "i64";
    const VALUE_TYPE: ValType = ValType::I64;

    fn return_imm_instr(&self) -> Instruction {
        match <Const32<i64>>::try_from(*self as i64).ok() {
            Some(value) => Instruction::return_i64imm32(value),
            None => Instruction::return_reg(Reg::from(-1)),
        }
    }
}

impl WasmTy for i64 {
    const NAME: &'static str = "i64";
    const VALUE_TYPE: ValType = ValType::I64;

    fn return_imm_instr(&self) -> Instruction {
        match <Const32<i64>>::try_from(*self).ok() {
            Some(value) => Instruction::return_i64imm32(value),
            None => Instruction::return_reg(Reg::from(-1)),
        }
    }
}

impl WasmTy for f32 {
    const NAME: &'static str = "f32";
    const VALUE_TYPE: ValType = ValType::F32;

    fn return_imm_instr(&self) -> Instruction {
        Instruction::return_imm32(*self)
    }
}

impl WasmTy for f64 {
    const NAME: &'static str = "f64";
    const VALUE_TYPE: ValType = ValType::F64;

    fn return_imm_instr(&self) -> Instruction {
        match <Const32<f64>>::try_from(*self).ok() {
            Some(value) => Instruction::return_f64imm32(value),
            None => Instruction::return_reg(Reg::from(-1)),
        }
    }
}
