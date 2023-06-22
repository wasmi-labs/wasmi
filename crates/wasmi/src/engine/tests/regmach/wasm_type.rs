use crate::{
    core::{UntypedValue, F32},
    engine::{
        bytecode2::{Const32, Instruction},
        ConstRef,
    },
};
use core::fmt::Display;

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
