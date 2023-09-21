use super::Executor;
use crate::{
    core::{TrapCode, UntypedValue},
    engine::regmach::bytecode::{BinInstr, BinInstrImm16, CopysignImmInstr, Sign},
};

#[cfg(doc)]
use crate::engine::regmach::bytecode::Instruction;

macro_rules! impl_binary {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            #[inline(always)]
            pub fn $fn_name(&mut self, instr: BinInstr) {
                self.execute_binary(instr, $op)
            }
        )*
    };
}
impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    impl_binary! {
        (Instruction::I32Add, execute_i32_add, UntypedValue::i32_add),
        (Instruction::I32Sub, execute_i32_sub, UntypedValue::i32_sub),
        (Instruction::I32Mul, execute_i32_mul, UntypedValue::i32_mul),
        (Instruction::I32And, execute_i32_and, UntypedValue::i32_and),
        (Instruction::I32Or, execute_i32_or, UntypedValue::i32_or),
        (Instruction::I32Xor, execute_i32_xor, UntypedValue::i32_xor),

        (Instruction::I64Add, execute_i64_add, UntypedValue::i64_add),
        (Instruction::I64Sub, execute_i64_sub, UntypedValue::i64_sub),
        (Instruction::I64Mul, execute_i64_mul, UntypedValue::i64_mul),
        (Instruction::I64And, execute_i64_and, UntypedValue::i64_and),
        (Instruction::I64Or, execute_i64_or, UntypedValue::i64_or),
        (Instruction::I64Xor, execute_i64_xor, UntypedValue::i64_xor),

        (Instruction::I32Shl, execute_i32_shl, UntypedValue::i32_shl),
        (Instruction::I32ShrU, execute_i32_shr_u, UntypedValue::i32_shr_u),
        (Instruction::I32ShrS, execute_i32_shr_s, UntypedValue::i32_shr_s),
        (Instruction::I32Rotl, execute_i32_rotl, UntypedValue::i32_rotl),
        (Instruction::I32Rotr, execute_i32_rotr, UntypedValue::i32_rotr),

        (Instruction::I64Shl, execute_i64_shl, UntypedValue::i64_shl),
        (Instruction::I64ShrU, execute_i64_shr_u, UntypedValue::i64_shr_u),
        (Instruction::I64ShrS, execute_i64_shr_s, UntypedValue::i64_shr_s),
        (Instruction::I64Rotl, execute_i64_rotl, UntypedValue::i64_rotl),
        (Instruction::I64Rotr, execute_i64_rotr, UntypedValue::i64_rotr),

        (Instruction::F32Add, execute_f32_add, UntypedValue::f32_add),
        (Instruction::F32Sub, execute_f32_sub, UntypedValue::f32_sub),
        (Instruction::F32Mul, execute_f32_mul, UntypedValue::f32_mul),
        (Instruction::F32Div, execute_f32_div, UntypedValue::f32_div),
        (Instruction::F32Min, execute_f32_min, UntypedValue::f32_min),
        (Instruction::F32Max, execute_f32_max, UntypedValue::f32_max),
        (Instruction::F32Copysign, execute_f32_copysign, UntypedValue::f32_copysign),

        (Instruction::F64Add, execute_f64_add, UntypedValue::f64_add),
        (Instruction::F64Sub, execute_f64_sub, UntypedValue::f64_sub),
        (Instruction::F64Mul, execute_f64_mul, UntypedValue::f64_mul),
        (Instruction::F64Div, execute_f64_div, UntypedValue::f64_div),
        (Instruction::F64Min, execute_f64_min, UntypedValue::f64_min),
        (Instruction::F64Max, execute_f64_max, UntypedValue::f64_max),
        (Instruction::F64Copysign, execute_f64_copysign, UntypedValue::f64_copysign),
    }
}

macro_rules! impl_binary_imm16 {
    ( $( ($ty:ty, Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            #[inline(always)]
            pub fn $fn_name(&mut self, instr: BinInstrImm16<$ty>) {
                self.execute_binary_imm16(instr, $op)
            }
        )*
    };
}
impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    impl_binary_imm16! {
        (i32, Instruction::I32AddImm16, execute_i32_add_imm16, UntypedValue::i32_add),
        (i32, Instruction::I32SubImm16, execute_i32_sub_imm16, UntypedValue::i32_sub),
        (i32, Instruction::I32MulImm16, execute_i32_mul_imm16, UntypedValue::i32_mul),
        (i32, Instruction::I32AndImm16, execute_i32_and_imm16, UntypedValue::i32_and),
        (i32, Instruction::I32OrImm16, execute_i32_or_imm16, UntypedValue::i32_or),
        (i32, Instruction::I32XorImm16, execute_i32_xor_imm16, UntypedValue::i32_xor),

        (i64, Instruction::I64AddImm16, execute_i64_add_imm16, UntypedValue::i64_add),
        (i64, Instruction::I64SubImm16, execute_i64_sub_imm16, UntypedValue::i64_sub),
        (i64, Instruction::I64MulImm16, execute_i64_mul_imm16, UntypedValue::i64_mul),
        (i64, Instruction::I64AndImm16, execute_i64_and_imm16, UntypedValue::i64_and),
        (i64, Instruction::I64OrImm16, execute_i64_or_imm16, UntypedValue::i64_or),
        (i64, Instruction::I64XorImm16, execute_i64_xor_imm16, UntypedValue::i64_xor),

        (i32, Instruction::I32ShlImm, execute_i32_shl_imm, UntypedValue::i32_shl),
        (i32, Instruction::I32ShrUImm, execute_i32_shr_u_imm, UntypedValue::i32_shr_u),
        (i32, Instruction::I32ShrSImm, execute_i32_shr_s_imm, UntypedValue::i32_shr_s),
        (i32, Instruction::I32RotlImm, execute_i32_rotl_imm, UntypedValue::i32_rotl),
        (i32, Instruction::I32RotrImm, execute_i32_rotr_imm, UntypedValue::i32_rotr),

        (i64, Instruction::I64ShlImm, execute_i64_shl_imm, UntypedValue::i64_shl),
        (i64, Instruction::I64ShrUImm, execute_i64_shr_u_imm, UntypedValue::i64_shr_u),
        (i64, Instruction::I64ShrSImm, execute_i64_shr_s_imm, UntypedValue::i64_shr_s),
        (i64, Instruction::I64RotlImm, execute_i64_rotl_imm, UntypedValue::i64_rotl),
        (i64, Instruction::I64RotrImm, execute_i64_rotr_imm, UntypedValue::i64_rotr),

    }
}

macro_rules! impl_binary_imm16_rev {
    ( $( ($ty:ty, Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            #[inline(always)]
            pub fn $fn_name(&mut self, instr: BinInstrImm16<$ty>) {
                self.execute_binary_imm16_rev(instr, $op)
            }
        )*
    };
}
impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    impl_binary_imm16_rev! {
        (i32, Instruction::I32SubImm16Rev, execute_i32_sub_imm16_rev, UntypedValue::i32_sub),
        (i64, Instruction::I64SubImm16Rev, execute_i64_sub_imm16_rev, UntypedValue::i64_sub),

        (i32, Instruction::I32ShlImm16Rev, execute_i32_shl_imm16_rev, UntypedValue::i32_shl),
        (i32, Instruction::I32ShrUImm16Rev, execute_i32_shr_u_imm16_rev, UntypedValue::i32_shr_u),
        (i32, Instruction::I32ShrSImm16Rev, execute_i32_shr_s_imm16_rev, UntypedValue::i32_shr_s),
        (i32, Instruction::I32RotlImm16Rev, execute_i32_rotl_imm16_rev, UntypedValue::i32_rotl),
        (i32, Instruction::I32RotrImm16Rev, execute_i32_rotr_imm16_rev, UntypedValue::i32_rotr),

        (i64, Instruction::I64ShlImm16Rev, execute_i64_shl_imm16_rev, UntypedValue::i64_shl),
        (i64, Instruction::I64ShrUImm16Rev, execute_i64_shr_u_imm16_rev, UntypedValue::i64_shr_u),
        (i64, Instruction::I64ShrSImm16Rev, execute_i64_shr_s_imm16_rev, UntypedValue::i64_shr_s),
        (i64, Instruction::I64RotlImm16Rev, execute_i64_rotl_imm16_rev, UntypedValue::i64_rotl),
        (i64, Instruction::I64RotrImm16Rev, execute_i64_rotr_imm16_rev, UntypedValue::i64_rotr),
    }
}

macro_rules! impl_fallible_binary {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            #[inline(always)]
            pub fn $fn_name(&mut self, instr: BinInstr) -> Result<(), TrapCode> {
                self.try_execute_binary(instr, $op)
            }
        )*
    };
}
impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    impl_fallible_binary! {
        (Instruction::I32DivS, execute_i32_div_s, UntypedValue::i32_div_s),
        (Instruction::I32DivU, execute_i32_div_u, UntypedValue::i32_div_u),
        (Instruction::I32RemS, execute_i32_rem_s, UntypedValue::i32_rem_s),
        (Instruction::I32RemU, execute_i32_rem_u, UntypedValue::i32_rem_u),

        (Instruction::I64DivS, execute_i64_div_s, UntypedValue::i64_div_s),
        (Instruction::I64DivU, execute_i64_div_u, UntypedValue::i64_div_u),
        (Instruction::I64RemS, execute_i64_rem_s, UntypedValue::i64_rem_s),
        (Instruction::I64RemU, execute_i64_rem_u, UntypedValue::i64_rem_u),
    }
}

macro_rules! impl_fallible_binary_imm16 {
    ( $( ($ty:ty, Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            #[inline(always)]
            pub fn $fn_name(&mut self, instr: BinInstrImm16<$ty>) -> Result<(), TrapCode> {
                self.try_execute_binary_imm16(instr, $op)
            }
        )*
    };
}
impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    impl_fallible_binary_imm16! {
        (i32, Instruction::I32DivSImm16, execute_i32_div_s_imm16, UntypedValue::i32_div_s),
        (u32, Instruction::I32DivUImm16, execute_i32_div_u_imm16, UntypedValue::i32_div_u),
        (i32, Instruction::I32RemSImm16, execute_i32_rem_s_imm16, UntypedValue::i32_rem_s),
        (u32, Instruction::I32RemUImm16, execute_i32_rem_u_imm16, UntypedValue::i32_rem_u),

        (i64, Instruction::I64DivSImm16, execute_i64_div_s_imm16, UntypedValue::i64_div_s),
        (u64, Instruction::I64DivUImm16, execute_i64_div_u_imm16, UntypedValue::i64_div_u),
        (i64, Instruction::I64RemSImm16, execute_i64_rem_s_imm16, UntypedValue::i64_rem_s),
        (u64, Instruction::I64RemUImm16, execute_i64_rem_u_imm16, UntypedValue::i64_rem_u),
    }
}

macro_rules! impl_fallible_binary_imm16_rev {
    ( $( ($ty:ty, Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            #[inline(always)]
            pub fn $fn_name(&mut self, instr: BinInstrImm16<$ty>) -> Result<(), TrapCode> {
                self.try_execute_binary_imm16_rev(instr, $op)
            }
        )*
    };
}
impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    impl_fallible_binary_imm16_rev! {
        (i32, Instruction::I32DivSImm16Rev, execute_i32_div_s_imm16_rev, UntypedValue::i32_div_s),
        (u32, Instruction::I32DivUImm16Rev, execute_i32_div_u_imm16_rev, UntypedValue::i32_div_u),
        (i32, Instruction::I32RemSImm16Rev, execute_i32_rem_s_imm16_rev, UntypedValue::i32_rem_s),
        (u32, Instruction::I32RemUImm16Rev, execute_i32_rem_u_imm16_rev, UntypedValue::i32_rem_u),

        (i64, Instruction::I64DivSImm16Rev, execute_i64_div_s_imm16_rev, UntypedValue::i64_div_s),
        (u64, Instruction::I64DivUImm16Rev, execute_i64_div_u_imm16_rev, UntypedValue::i64_div_u),
        (i64, Instruction::I64RemSImm16Rev, execute_i64_rem_s_imm16_rev, UntypedValue::i64_rem_s),
        (u64, Instruction::I64RemUImm16Rev, execute_i64_rem_u_imm16_rev, UntypedValue::i64_rem_u),
    }
}

impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    /// Executes an [`Instruction::F32CopysignImm`].
    #[inline(always)]
    pub fn execute_f32_copysign_imm(&mut self, instr: CopysignImmInstr) {
        let lhs = self.get_register(instr.lhs);
        let rhs = match instr.rhs {
            Sign::Pos => 1.0_f32,
            Sign::Neg => -1.0_f32,
        };
        self.set_register(instr.result, UntypedValue::f32_copysign(lhs, rhs.into()));
        self.next_instr()
    }

    /// Executes an [`Instruction::F64CopysignImm`].
    #[inline(always)]
    pub fn execute_f64_copysign_imm(&mut self, instr: CopysignImmInstr) {
        let lhs = self.get_register(instr.lhs);
        let rhs = match instr.rhs {
            Sign::Pos => 1.0_f64,
            Sign::Neg => -1.0_f64,
        };
        self.set_register(instr.result, UntypedValue::f64_copysign(lhs, rhs.into()));
        self.next_instr()
    }
}
