use super::Executor;
use crate::{
    core::{TrapCode, UntypedValue},
    engine::regmach::bytecode::{
        BinAssignInstr,
        BinAssignInstrImm,
        BinAssignInstrImm32,
        BinInstr,
        BinInstrImm16,
        CopysignImmInstr,
        Sign,
    },
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
        let rhs = instr.rhs.to_f32();
        self.set_register(instr.result, UntypedValue::f32_copysign(lhs, rhs.into()));
        self.next_instr()
    }

    /// Executes an [`Instruction::F32CopysignAssignImm`].
    #[inline(always)]
    pub fn execute_f32_copysign_assign_imm(&mut self, instr: BinAssignInstrImm<Sign>) {
        let lhs = self.get_register(instr.inout);
        let rhs = instr.rhs.to_f32();
        self.set_register(instr.inout, UntypedValue::f32_copysign(lhs, rhs.into()));
        self.next_instr()
    }

    /// Executes an [`Instruction::F64CopysignImm`].
    #[inline(always)]
    pub fn execute_f64_copysign_imm(&mut self, instr: CopysignImmInstr) {
        let lhs = self.get_register(instr.lhs);
        let rhs = instr.rhs.to_f64();
        self.set_register(instr.result, UntypedValue::f64_copysign(lhs, rhs.into()));
        self.next_instr()
    }

    /// Executes an [`Instruction::F64CopysignAssignImm`].
    #[inline(always)]
    pub fn execute_f64_copysign_assign_imm(&mut self, instr: BinAssignInstrImm<Sign>) {
        let lhs = self.get_register(instr.inout);
        let rhs = instr.rhs.to_f64();
        self.set_register(instr.inout, UntypedValue::f64_copysign(lhs, rhs.into()));
        self.next_instr()
    }
}

macro_rules! impl_binary_assign {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            #[inline(always)]
            pub fn $fn_name(&mut self, instr: BinAssignInstr) {
                self.execute_binary_assign(instr, $op)
            }
        )*
    };
}
impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    impl_binary_assign! {
        (Instruction::I32EqAssign, execute_i32_eq_assign, UntypedValue::i32_eq),
        (Instruction::I32NeAssign, execute_i32_ne_assign, UntypedValue::i32_ne),
        (Instruction::I32LtSAssign, execute_i32_lt_s_assign, UntypedValue::i32_lt_s),
        (Instruction::I32LtUAssign, execute_i32_lt_u_assign, UntypedValue::i32_lt_u),
        (Instruction::I32LeSAssign, execute_i32_le_s_assign, UntypedValue::i32_le_s),
        (Instruction::I32LeUAssign, execute_i32_le_u_assign, UntypedValue::i32_le_u),
        (Instruction::I32GtSAssign, execute_i32_gt_s_assign, UntypedValue::i32_gt_s),
        (Instruction::I32GtUAssign, execute_i32_gt_u_assign, UntypedValue::i32_gt_u),
        (Instruction::I32GeSAssign, execute_i32_ge_s_assign, UntypedValue::i32_ge_s),
        (Instruction::I32GeUAssign, execute_i32_ge_u_assign, UntypedValue::i32_ge_u),

        (Instruction::I64EqAssign, execute_i64_eq_assign, UntypedValue::i64_eq),
        (Instruction::I64NeAssign, execute_i64_ne_assign, UntypedValue::i64_ne),
        (Instruction::I64LtSAssign, execute_i64_lt_s_assign, UntypedValue::i64_lt_s),
        (Instruction::I64LtUAssign, execute_i64_lt_u_assign, UntypedValue::i64_lt_u),
        (Instruction::I64LeSAssign, execute_i64_le_s_assign, UntypedValue::i64_le_s),
        (Instruction::I64LeUAssign, execute_i64_le_u_assign, UntypedValue::i64_le_u),
        (Instruction::I64GtSAssign, execute_i64_gt_s_assign, UntypedValue::i64_gt_s),
        (Instruction::I64GtUAssign, execute_i64_gt_u_assign, UntypedValue::i64_gt_u),
        (Instruction::I64GeSAssign, execute_i64_ge_s_assign, UntypedValue::i64_ge_s),
        (Instruction::I64GeUAssign, execute_i64_ge_u_assign, UntypedValue::i64_ge_u),

        (Instruction::F32EqAssign, execute_f32_eq_assign, UntypedValue::f32_eq),
        (Instruction::F32NeAssign, execute_f32_ne_assign, UntypedValue::f32_ne),
        (Instruction::F32LtAssign, execute_f32_lt_assign, UntypedValue::f32_lt),
        (Instruction::F32LeAssign, execute_f32_le_assign, UntypedValue::f32_le),
        (Instruction::F32GtAssign, execute_f32_gt_assign, UntypedValue::f32_gt),
        (Instruction::F32GeAssign, execute_f32_ge_assign, UntypedValue::f32_ge),

        (Instruction::F64EqAssign, execute_f64_eq_assign, UntypedValue::f64_eq),
        (Instruction::F64NeAssign, execute_f64_ne_assign, UntypedValue::f64_ne),
        (Instruction::F64LtAssign, execute_f64_lt_assign, UntypedValue::f64_lt),
        (Instruction::F64LeAssign, execute_f64_le_assign, UntypedValue::f64_le),
        (Instruction::F64GtAssign, execute_f64_gt_assign, UntypedValue::f64_gt),
        (Instruction::F64GeAssign, execute_f64_ge_assign, UntypedValue::f64_ge),

        (Instruction::I32AddAssign, execute_i32_add_assign, UntypedValue::i32_add),
        (Instruction::I32SubAssign, execute_i32_sub_assign, UntypedValue::i32_sub),
        (Instruction::I32MulAssign, execute_i32_mul_assign, UntypedValue::i32_mul),
        (Instruction::I32AndAssign, execute_i32_and_assign, UntypedValue::i32_and),
        (Instruction::I32OrAssign, execute_i32_or_assign, UntypedValue::i32_or),
        (Instruction::I32XorAssign, execute_i32_xor_assign, UntypedValue::i32_xor),
        (Instruction::I32ShlAssign, execute_i32_shl_assign, UntypedValue::i32_shl),
        (Instruction::I32ShrSAssign, execute_i32_shr_s_assign, UntypedValue::i32_shr_s),
        (Instruction::I32ShrUAssign, execute_i32_shr_u_assign, UntypedValue::i32_shr_u),
        (Instruction::I32RotlAssign, execute_i32_rotl_assign, UntypedValue::i32_rotl),
        (Instruction::I32RotrAssign, execute_i32_rotr_assign, UntypedValue::i32_rotr),

        (Instruction::I64AddAssign, execute_i64_add_assign, UntypedValue::i64_add),
        (Instruction::I64SubAssign, execute_i64_sub_assign, UntypedValue::i64_sub),
        (Instruction::I64MulAssign, execute_i64_mul_assign, UntypedValue::i64_mul),
        (Instruction::I64AndAssign, execute_i64_and_assign, UntypedValue::i64_and),
        (Instruction::I64OrAssign, execute_i64_or_assign, UntypedValue::i64_or),
        (Instruction::I64XorAssign, execute_i64_xor_assign, UntypedValue::i64_xor),
        (Instruction::I64ShlAssign, execute_i64_shl_assign, UntypedValue::i64_shl),
        (Instruction::I64ShrSAssign, execute_i64_shr_s_assign, UntypedValue::i64_shr_s),
        (Instruction::I64ShrUAssign, execute_i64_shr_u_assign, UntypedValue::i64_shr_u),
        (Instruction::I64RotlAssign, execute_i64_rotl_assign, UntypedValue::i64_rotl),
        (Instruction::I64RotrAssign, execute_i64_rotr_assign, UntypedValue::i64_rotr),

        (Instruction::F32AddAssign, execute_f32_add_assign, UntypedValue::f32_add),
        (Instruction::F32SubAssign, execute_f32_sub_assign, UntypedValue::f32_sub),
        (Instruction::F32MulAssign, execute_f32_mul_assign, UntypedValue::f32_mul),
        (Instruction::F32DivAssign, execute_f32_div_assign, UntypedValue::f32_div),
        (Instruction::F32MinAssign, execute_f32_min_assign, UntypedValue::f32_min),
        (Instruction::F32MaxAssign, execute_f32_max_assign, UntypedValue::f32_max),
        (Instruction::F32CopysignAssign, execute_f32_copysign_assign, UntypedValue::f32_copysign),

        (Instruction::F64AddAssign, execute_f64_add_assign, UntypedValue::f64_add),
        (Instruction::F64SubAssign, execute_f64_sub_assign, UntypedValue::f64_sub),
        (Instruction::F64MulAssign, execute_f64_mul_assign, UntypedValue::f64_mul),
        (Instruction::F64DivAssign, execute_f64_div_assign, UntypedValue::f64_div),
        (Instruction::F64MinAssign, execute_f64_min_assign, UntypedValue::f64_min),
        (Instruction::F64MaxAssign, execute_f64_max_assign, UntypedValue::f64_max),
        (Instruction::F64CopysignAssign, execute_f64_copysign_assign, UntypedValue::f64_copysign),
    }
}

macro_rules! impl_fallible_binary_assign {
    ( $( (Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            #[inline(always)]
            pub fn $fn_name(&mut self, instr: BinAssignInstr) -> Result<(), TrapCode> {
                self.try_execute_binary_assign(instr, $op)
            }
        )*
    };
}
impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    impl_fallible_binary_assign! {
            (Instruction::I32DivSAssign, execute_i32_div_s_assign, UntypedValue::i32_div_s),
            (Instruction::I32DivUAssign, execute_i32_div_u_assign, UntypedValue::i32_div_u),
            (Instruction::I32RemSAssign, execute_i32_rem_s_assign, UntypedValue::i32_rem_s),
            (Instruction::I32RemUAssign, execute_i32_rem_u_assign, UntypedValue::i32_rem_u),

            (Instruction::I64DivSAssign, execute_i64_div_s_assign, UntypedValue::i64_div_s),
            (Instruction::I64DivUAssign, execute_i64_div_u_assign, UntypedValue::i64_div_u),
            (Instruction::I64RemSAssign, execute_i64_rem_s_assign, UntypedValue::i64_rem_s),
            (Instruction::I64RemUAssign, execute_i64_rem_u_assign, UntypedValue::i64_rem_u),
    }
}

macro_rules! impl_binary_assign_imm32 {
    ( $( ($ty:ty, Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            #[inline(always)]
            pub fn $fn_name(&mut self, instr: BinAssignInstrImm32<$ty>) {
                self.execute_binary_assign_imm32(instr, $op)
            }
        )*
    };
}
impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    impl_binary_assign_imm32! {
        (i32, Instruction::I32EqAssignImm, execute_i32_eq_assign_imm, UntypedValue::i32_eq),
        (i32, Instruction::I32NeAssignImm, execute_i32_ne_assign_imm, UntypedValue::i32_ne),
        (i32, Instruction::I32LtSAssignImm, execute_i32_lt_s_assign_imm, UntypedValue::i32_lt_s),
        (u32, Instruction::I32LtUAssignImm, execute_i32_lt_u_assign_imm, UntypedValue::i32_lt_u),
        (i32, Instruction::I32LeSAssignImm, execute_i32_le_s_assign_imm, UntypedValue::i32_le_s),
        (u32, Instruction::I32LeUAssignImm, execute_i32_le_u_assign_imm, UntypedValue::i32_le_u),
        (i32, Instruction::I32GtSAssignImm, execute_i32_gt_s_assign_imm, UntypedValue::i32_gt_s),
        (u32, Instruction::I32GtUAssignImm, execute_i32_gt_u_assign_imm, UntypedValue::i32_gt_u),
        (i32, Instruction::I32GeSAssignImm, execute_i32_ge_s_assign_imm, UntypedValue::i32_ge_s),
        (u32, Instruction::I32GeUAssignImm, execute_i32_ge_u_assign_imm, UntypedValue::i32_ge_u),

        (i64, Instruction::I64EqAssignImm32, execute_i64_eq_assign_imm32, UntypedValue::i64_eq),
        (i64, Instruction::I64NeAssignImm32, execute_i64_ne_assign_imm32, UntypedValue::i64_ne),
        (i64, Instruction::I64LtSAssignImm32, execute_i64_lt_s_assign_imm32, UntypedValue::i64_lt_s),
        (u64, Instruction::I64LtUAssignImm32, execute_i64_lt_u_assign_imm32, UntypedValue::i64_lt_u),
        (i64, Instruction::I64LeSAssignImm32, execute_i64_le_s_assign_imm32, UntypedValue::i64_le_s),
        (u64, Instruction::I64LeUAssignImm32, execute_i64_le_u_assign_imm32, UntypedValue::i64_le_u),
        (i64, Instruction::I64GtSAssignImm32, execute_i64_gt_s_assign_imm32, UntypedValue::i64_gt_s),
        (u64, Instruction::I64GtUAssignImm32, execute_i64_gt_u_assign_imm32, UntypedValue::i64_gt_u),
        (i64, Instruction::I64GeSAssignImm32, execute_i64_ge_s_assign_imm32, UntypedValue::i64_ge_s),
        (u64, Instruction::I64GeUAssignImm32, execute_i64_ge_u_assign_imm32, UntypedValue::i64_ge_u),

        (f32, Instruction::F32EqAssignImm, execute_f32_eq_assign_imm, UntypedValue::f32_eq),
        (f32, Instruction::F32NeAssignImm, execute_f32_ne_assign_imm, UntypedValue::f32_ne),
        (f32, Instruction::F32LtAssignImm, execute_f32_lt_assign_imm, UntypedValue::f32_lt),
        (f32, Instruction::F32LeAssignImm, execute_f32_le_assign_imm, UntypedValue::f32_le),
        (f32, Instruction::F32GtAssignImm, execute_f32_gt_assign_imm, UntypedValue::f32_gt),
        (f32, Instruction::F32GeAssignImm, execute_f32_ge_assign_imm, UntypedValue::f32_ge),

        (f64, Instruction::F64EqAssignImm32, execute_f64_eq_assign_imm32, UntypedValue::f64_eq),
        (f64, Instruction::F64NeAssignImm32, execute_f64_ne_assign_imm32, UntypedValue::f64_ne),
        (f64, Instruction::F64LtAssignImm32, execute_f64_lt_assign_imm32, UntypedValue::f64_lt),
        (f64, Instruction::F64LeAssignImm32, execute_f64_le_assign_imm32, UntypedValue::f64_le),
        (f64, Instruction::F64GtAssignImm32, execute_f64_gt_assign_imm32, UntypedValue::f64_gt),
        (f64, Instruction::F64GeAssignImm32, execute_f64_ge_assign_imm32, UntypedValue::f64_ge),

        (i32, Instruction::I32AddAssignImm, execute_i32_add_assign_imm, UntypedValue::i32_add),
        (i32, Instruction::I32SubAssignImm, execute_i32_sub_assign_imm, UntypedValue::i32_sub),
        (i32, Instruction::I32MulAssignImm, execute_i32_mul_assign_imm, UntypedValue::i32_mul),
        (i32, Instruction::I32AndAssignImm, execute_i32_and_assign_imm, UntypedValue::i32_and),
        (i32, Instruction::I32OrAssignImm, execute_i32_or_assign_imm, UntypedValue::i32_or),
        (i32, Instruction::I32XorAssignImm, execute_i32_xor_assign_imm, UntypedValue::i32_xor),
        (i32, Instruction::I32ShlAssignImm, execute_i32_shl_assign_imm, UntypedValue::i32_shl),
        (i32, Instruction::I32ShrSAssignImm, execute_i32_shr_s_assign_imm, UntypedValue::i32_shr_s),
        (i32, Instruction::I32ShrUAssignImm, execute_i32_shr_u_assign_imm, UntypedValue::i32_shr_u),
        (i32, Instruction::I32RotlAssignImm, execute_i32_rotl_assign_imm, UntypedValue::i32_rotl),
        (i32, Instruction::I32RotrAssignImm, execute_i32_rotr_assign_imm, UntypedValue::i32_rotr),

        (i64, Instruction::I64AddAssignImm32, execute_i64_add_assign_imm32, UntypedValue::i64_add),
        (i64, Instruction::I64SubAssignImm32, execute_i64_sub_assign_imm32, UntypedValue::i64_sub),
        (i64, Instruction::I64MulAssignImm32, execute_i64_mul_assign_imm32, UntypedValue::i64_mul),
        (i64, Instruction::I64AndAssignImm32, execute_i64_and_assign_imm32, UntypedValue::i64_and),
        (i64, Instruction::I64OrAssignImm32, execute_i64_or_assign_imm32, UntypedValue::i64_or),
        (i64, Instruction::I64XorAssignImm32, execute_i64_xor_assign_imm32, UntypedValue::i64_xor),
        (i64, Instruction::I64ShlAssignImm32, execute_i64_shl_assign_imm32, UntypedValue::i64_shl),
        (i64, Instruction::I64ShrSAssignImm32, execute_i64_shr_s_assign_imm32, UntypedValue::i64_shr_s),
        (i64, Instruction::I64ShrUAssignImm32, execute_i64_shr_u_assign_imm32, UntypedValue::i64_shr_u),
        (i64, Instruction::I64RotlAssignImm32, execute_i64_rotl_assign_imm32, UntypedValue::i64_rotl),
        (i64, Instruction::I64RotrAssignImm32, execute_i64_rotr_assign_imm32, UntypedValue::i64_rotr),

        (f32, Instruction::F32AddAssignImm, execute_f32_add_assign_imm, UntypedValue::f32_add),
        (f32, Instruction::F32SubAssignImm, execute_f32_sub_assign_imm, UntypedValue::f32_sub),
        (f32, Instruction::F32MulAssignImm, execute_f32_mul_assign_imm, UntypedValue::f32_mul),
        (f32, Instruction::F32DivAssignImm, execute_f32_div_assign_imm, UntypedValue::f32_div),
        (f32, Instruction::F32MinAssignImm, execute_f32_min_assign_imm, UntypedValue::f32_min),
        (f32, Instruction::F32MaxAssignImm, execute_f32_max_assign_imm, UntypedValue::f32_max),

        (f64, Instruction::F64AddAssignImm32, execute_f64_add_assign_imm32, UntypedValue::f64_add),
        (f64, Instruction::F64SubAssignImm32, execute_f64_sub_assign_imm32, UntypedValue::f64_sub),
        (f64, Instruction::F64MulAssignImm32, execute_f64_mul_assign_imm32, UntypedValue::f64_mul),
        (f64, Instruction::F64DivAssignImm32, execute_f64_div_assign_imm32, UntypedValue::f64_div),
        (f64, Instruction::F64MinAssignImm32, execute_f64_min_assign_imm32, UntypedValue::f64_min),
        (f64, Instruction::F64MaxAssignImm32, execute_f64_max_assign_imm32, UntypedValue::f64_max),
    }
}

macro_rules! impl_fallible_binary_assign_imm32 {
    ( $( ($ty:ty, Instruction::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($var_name), "`].")]
            #[inline(always)]
            pub fn $fn_name(&mut self, instr: BinAssignInstrImm32<$ty>) -> Result<(), TrapCode> {
                self.try_execute_binary_assign_imm32(instr, $op)
            }
        )*
    };
}
impl<'ctx, 'engine> Executor<'ctx, 'engine> {
    impl_fallible_binary_assign_imm32! {
        (i32, Instruction::I32DivSAssignImm, execute_i32_div_s_assign_imm, UntypedValue::i32_div_s),
        (u32, Instruction::I32DivUAssignImm, execute_i32_div_u_assign_imm, UntypedValue::i32_div_u),
        (i32, Instruction::I32RemSAssignImm, execute_i32_rem_s_assign_imm, UntypedValue::i32_rem_s),
        (u32, Instruction::I32RemUAssignImm, execute_i32_rem_u_assign_imm, UntypedValue::i32_rem_u),

        (i64, Instruction::I64DivSAssignImm32, execute_i64_div_s_assign_imm32, UntypedValue::i64_div_s),
        (u64, Instruction::I64DivUAssignImm32, execute_i64_div_u_assign_imm32, UntypedValue::i64_div_u),
        (i64, Instruction::I64RemSAssignImm32, execute_i64_rem_s_assign_imm32, UntypedValue::i64_rem_s),
        (u64, Instruction::I64RemUAssignImm32, execute_i64_rem_u_assign_imm32, UntypedValue::i64_rem_u),
    }
}
