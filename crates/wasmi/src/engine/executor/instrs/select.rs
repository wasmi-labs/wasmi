use super::{Executor, InstructionPtr, UntypedValueCmpExt, UntypedValueExt};
use crate::{
    core::{wasm, ReadAs, UntypedVal},
    engine::utils::unreachable_unchecked,
    ir::{AnyConst32, Const16, Const32, Instruction, Reg},
};

impl<'engine> Executor<'engine> {
    /// Fetches two [`Reg`]s.
    fn fetch_register_2(&self) -> (Reg, Reg) {
        let mut addr: InstructionPtr = self.ip;
        addr.add(1);
        match *addr.get() {
            Instruction::Register2 { regs: [reg0, reg1] } => (reg0, reg1),
            unexpected => {
                // Safety: Wasmi translation guarantees that [`Instruction::Register2`] exists.
                unsafe {
                    unreachable_unchecked!(
                        "expected `Instruction::Register2` but found {unexpected:?}"
                    )
                }
            }
        }
    }

    /// Fetches a [`Reg`] and a 32-bit immediate value of type `T`.
    fn fetch_register_and_imm32<T>(&self) -> (Reg, T)
    where
        T: From<AnyConst32>,
    {
        let mut addr: InstructionPtr = self.ip;
        addr.add(1);
        match *addr.get() {
            Instruction::RegisterAndImm32 { reg, imm } => (reg, T::from(imm)),
            unexpected => {
                // Safety: Wasmi translation guarantees that [`Instruction::RegisterAndImm32`] exists.
                unsafe {
                    unreachable_unchecked!(
                        "expected `Instruction::RegisterAndImm32` but found {unexpected:?}"
                    )
                }
            }
        }
    }

    /// Executes a `select` instruction generically.
    fn execute_select_impl<L, R>(
        &mut self,
        result: Reg,
        condition: Reg,
        lhs: impl FnOnce(&Self) -> L,
        rhs: impl FnOnce(&Self) -> R,
    ) where
        L: Into<UntypedVal>,
        R: Into<UntypedVal>,
    {
        let condition: bool = self.get_register_as(condition);
        let selected = match condition {
            true => lhs(self).into(),
            false => rhs(self).into(),
        };
        self.set_register(result, selected);
        self.next_instr_at(2);
    }

    /// Executes an [`Instruction::Select`].
    pub fn execute_select(&mut self, result: Reg, lhs: Reg) {
        let (condition, rhs) = self.fetch_register_2();
        self.execute_select_impl(
            result,
            condition,
            |this| this.get_register(lhs),
            |this| this.get_register(rhs),
        )
    }

    /// Executes an [`Instruction::SelectImm32Rhs`].
    pub fn execute_select_imm32_rhs(&mut self, result: Reg, lhs: Reg) {
        let (condition, rhs) = self.fetch_register_and_imm32::<AnyConst32>();
        self.execute_select_impl(
            result,
            condition,
            |this: &Executor<'engine>| this.get_register(lhs),
            |_| u32::from(rhs),
        )
    }

    /// Executes an [`Instruction::SelectImm32Lhs`].
    pub fn execute_select_imm32_lhs(&mut self, result: Reg, lhs: AnyConst32) {
        let (condition, rhs) = self.fetch_register_2();
        self.execute_select_impl(
            result,
            condition,
            |_| u32::from(lhs),
            |this| this.get_register(rhs),
        )
    }

    /// Executes an [`Instruction::SelectImm32`].
    pub fn execute_select_imm32(&mut self, result: Reg, lhs: AnyConst32) {
        let (condition, rhs) = self.fetch_register_and_imm32::<AnyConst32>();
        self.execute_select_impl(result, condition, |_| u32::from(lhs), |_| u32::from(rhs))
    }

    /// Executes an [`Instruction::SelectI64Imm32Rhs`].
    pub fn execute_select_i64imm32_rhs(&mut self, result: Reg, lhs: Reg) {
        let (condition, rhs) = self.fetch_register_and_imm32::<i32>();
        self.execute_select_impl(
            result,
            condition,
            |this| this.get_register(lhs),
            |_| i64::from(rhs),
        )
    }

    /// Executes an [`Instruction::SelectI64Imm32Lhs`].
    pub fn execute_select_i64imm32_lhs(&mut self, result: Reg, lhs: Const32<i64>) {
        let (condition, rhs) = self.fetch_register_2();
        self.execute_select_impl(
            result,
            condition,
            |_| i64::from(lhs),
            |this| this.get_register(rhs),
        )
    }

    /// Executes an [`Instruction::SelectI64Imm32`].
    pub fn execute_select_i64imm32(&mut self, result: Reg, lhs: Const32<i64>) {
        let (condition, rhs) = self.fetch_register_and_imm32::<i32>();
        self.execute_select_impl(result, condition, |_| i64::from(lhs), |_| i64::from(rhs))
    }

    /// Executes an [`Instruction::SelectF64Imm32Rhs`].
    pub fn execute_select_f64imm32_rhs(&mut self, result: Reg, lhs: Reg) {
        let (condition, rhs) = self.fetch_register_and_imm32::<f32>();
        self.execute_select_impl(
            result,
            condition,
            |this| this.get_register(lhs),
            |_| f64::from(rhs),
        )
    }

    /// Executes an [`Instruction::SelectF64Imm32Lhs`].
    pub fn execute_select_f64imm32_lhs(&mut self, result: Reg, lhs: Const32<f64>) {
        let (condition, rhs) = self.fetch_register_2();
        self.execute_select_impl(
            result,
            condition,
            |_| f64::from(lhs),
            |this| this.get_register(rhs),
        )
    }

    /// Executes an [`Instruction::SelectF64Imm32`].
    pub fn execute_select_f64imm32(&mut self, result: Reg, lhs: Const32<f64>) {
        let (condition, rhs) = self.fetch_register_and_imm32::<f32>();
        self.execute_select_impl(result, condition, |_| f64::from(lhs), |_| f64::from(rhs))
    }

    /// Executes a fused `cmp`+`select` instruction.
    #[inline(always)]
    fn execute_cmp_select_impl<T>(&mut self, result: Reg, lhs: Reg, rhs: Reg, f: fn(T, T) -> bool)
    where
        UntypedVal: ReadAs<T>,
    {
        let (true_val, false_val) = self.fetch_register_2();
        let lhs: T = self.get_register_as(lhs);
        let rhs: T = self.get_register_as(rhs);
        let selected = self.get_register(match f(lhs, rhs) {
            true => true_val,
            false => false_val,
        });
        self.set_register(result, selected);
        self.next_instr_at(2);
    }

    /// Executes a fused `cmp`+`select` instruction with immediate `rhs` parameter.
    #[inline(always)]
    fn execute_cmp_select_imm_rhs_impl<T>(
        &mut self,
        result: Reg,
        lhs: Reg,
        rhs: Const16<T>,
        f: fn(T, T) -> bool,
    ) where
        UntypedVal: ReadAs<T>,
        T: From<Const16<T>>,
    {
        let (true_val, false_val) = self.fetch_register_2();
        let lhs: T = self.get_register_as(lhs);
        let rhs: T = rhs.into();
        let selected = self.get_register(match f(lhs, rhs) {
            true => true_val,
            false => false_val,
        });
        self.set_register(result, selected);
        self.next_instr_at(2);
    }

    /// Executes a fused `cmp`+`select` instruction with immediate `rhs` parameter.
    #[inline(always)]
    fn execute_cmp_select_imm_lhs_impl<T>(
        &mut self,
        result: Reg,
        lhs: Const16<T>,
        rhs: Reg,
        f: fn(T, T) -> bool,
    ) where
        UntypedVal: ReadAs<T>,
        T: From<Const16<T>>,
    {
        let (true_val, false_val) = self.fetch_register_2();
        let lhs: T = lhs.into();
        let rhs: T = self.get_register_as(rhs);
        let selected = self.get_register(match f(lhs, rhs) {
            true => true_val,
            false => false_val,
        });
        self.set_register(result, selected);
        self.next_instr_at(2);
    }
}

macro_rules! impl_cmp_select_for {
    (
        $(
            (Instruction::$doc_name:ident, $fn_name:ident, $op:expr)
        ),* $(,)?
    ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($doc_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Reg, rhs: Reg) {
                self.execute_cmp_select_impl(result, lhs, rhs, $op)
            }
        )*
    };
}

macro_rules! impl_cmp_select_imm_rhs_for {
    (
        $(
            ($ty:ty, Instruction::$doc_name:ident, $fn_name:ident, $op:expr)
        ),* $(,)?
    ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($doc_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Reg, rhs: Const16<$ty>) {
                self.execute_cmp_select_imm_rhs_impl::<$ty>(result, lhs, rhs, $op)
            }
        )*
    };
}

macro_rules! impl_cmp_select_imm_lhs_for {
    (
        $(
            ($ty:ty, Instruction::$doc_name:ident, $fn_name:ident, $op:expr)
        ),* $(,)?
    ) => {
        $(
            #[doc = concat!("Executes an [`Instruction::", stringify!($doc_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Const16<$ty>, rhs: Reg) {
                self.execute_cmp_select_imm_lhs_impl::<$ty>(result, lhs, rhs, $op)
            }
        )*
    };
}

impl Executor<'_> {
    impl_cmp_select_for! {
        (Instruction::SelectI32Eq, execute_select_i32_eq, wasm::i32_eq),
        (Instruction::SelectI32Ne, execute_select_i32_ne, wasm::i32_ne),
        (Instruction::SelectI32LtS, execute_select_i32_lt_s, wasm::i32_lt_s),
        (Instruction::SelectI32LtU, execute_select_i32_lt_u, wasm::i32_lt_u),
        (Instruction::SelectI32LeS, execute_select_i32_le_s, wasm::i32_le_s),
        (Instruction::SelectI32LeU, execute_select_i32_le_u, wasm::i32_le_u),
        (Instruction::SelectI32And, execute_select_i32_and, <i32 as UntypedValueExt>::and),
        (Instruction::SelectI32Or, execute_select_i32_or, <i32 as UntypedValueExt>::or),
        (Instruction::SelectI32Xor, execute_select_i32_xor, <i32 as UntypedValueExt>::xor),
        (Instruction::SelectI32Nand, execute_select_i32_nand, <i32 as UntypedValueExt>::nand),
        (Instruction::SelectI32Nor, execute_select_i32_nor, <i32 as UntypedValueExt>::nor),
        (Instruction::SelectI32Xnor, execute_select_i32_xnor, <i32 as UntypedValueExt>::xnor),
        (Instruction::SelectI64Eq, execute_select_i64_eq, wasm::i64_eq),
        (Instruction::SelectI64Ne, execute_select_i64_ne, wasm::i64_ne),
        (Instruction::SelectI64LtS, execute_select_i64_lt_s, wasm::i64_lt_s),
        (Instruction::SelectI64LtU, execute_select_i64_lt_u, wasm::i64_lt_u),
        (Instruction::SelectI64LeS, execute_select_i64_le_s, wasm::i64_le_s),
        (Instruction::SelectI64LeU, execute_select_i64_le_u, wasm::i64_le_u),
        (Instruction::SelectI64And, execute_select_i64_and, <i64 as UntypedValueExt>::and),
        (Instruction::SelectI64Or, execute_select_i64_or, <i64 as UntypedValueExt>::or),
        (Instruction::SelectI64Xor, execute_select_i64_xor, <i64 as UntypedValueExt>::xor),
        (Instruction::SelectI64Nand, execute_select_i64_nand, <i64 as UntypedValueExt>::nand),
        (Instruction::SelectI64Nor, execute_select_i64_nor, <i64 as UntypedValueExt>::nor),
        (Instruction::SelectI64Xnor, execute_select_i64_xnor, <i64 as UntypedValueExt>::xnor),
        (Instruction::SelectF32Eq, execute_select_f32_eq, wasm::f32_eq),
        (Instruction::SelectF32Ne, execute_select_f32_ne, wasm::f32_ne),
        (Instruction::SelectF32Lt, execute_select_f32_lt, wasm::f32_lt),
        (Instruction::SelectF32Le, execute_select_f32_le, wasm::f32_le),
        (Instruction::SelectF32NotLt, execute_select_f32_not_lt, <f32 as UntypedValueCmpExt>::not_lt),
        (Instruction::SelectF32NotLe, execute_select_f32_not_le, <f32 as UntypedValueCmpExt>::not_le),
        (Instruction::SelectF64Eq, execute_select_f64_eq, wasm::f64_eq),
        (Instruction::SelectF64Ne, execute_select_f64_ne, wasm::f64_ne),
        (Instruction::SelectF64Lt, execute_select_f64_lt, wasm::f64_lt),
        (Instruction::SelectF64Le, execute_select_f64_le, wasm::f64_le),
        (Instruction::SelectF64NotLt, execute_select_f64_not_lt, <f64 as UntypedValueCmpExt>::not_lt),
        (Instruction::SelectF64NotLe, execute_select_f64_not_le, <f64 as UntypedValueCmpExt>::not_le),
    }

    impl_cmp_select_imm_rhs_for! {
        (i32, Instruction::SelectI32EqImm16, execute_select_i32_eq_imm16, wasm::i32_eq),
        (i32, Instruction::SelectI32NeImm16, execute_select_i32_ne_imm16, wasm::i32_ne),
        (i32, Instruction::SelectI32LtSImm16Rhs, execute_select_i32_lt_s_imm16_rhs, wasm::i32_lt_s),
        (u32, Instruction::SelectI32LtUImm16Rhs, execute_select_i32_lt_u_imm16_rhs, wasm::i32_lt_u),
        (i32, Instruction::SelectI32LeSImm16Rhs, execute_select_i32_le_s_imm16_rhs, wasm::i32_le_s),
        (u32, Instruction::SelectI32LeUImm16Rhs, execute_select_i32_le_u_imm16_rhs, wasm::i32_le_u),
        (i32, Instruction::SelectI32AndImm16, execute_select_i32_and_imm16, UntypedValueExt::and),
        (i32, Instruction::SelectI32OrImm16, execute_select_i32_or_imm16, UntypedValueExt::or),
        (i32, Instruction::SelectI32XorImm16, execute_select_i32_xor_imm16, UntypedValueExt::xor),
        (i32, Instruction::SelectI32NandImm16, execute_select_i32_nand_imm16, UntypedValueExt::nand),
        (i32, Instruction::SelectI32NorImm16, execute_select_i32_nor_imm16, UntypedValueExt::nor),
        (i32, Instruction::SelectI32XnorImm16, execute_select_i32_xnor_imm16, UntypedValueExt::xnor),
        (i64, Instruction::SelectI64EqImm16, execute_select_i64_eq_imm16, wasm::i64_eq),
        (i64, Instruction::SelectI64NeImm16, execute_select_i64_ne_imm16, wasm::i64_ne),
        (i64, Instruction::SelectI64LtSImm16Rhs, execute_select_i64_lt_s_imm16_rhs, wasm::i64_lt_s),
        (u64, Instruction::SelectI64LtUImm16Rhs, execute_select_i64_lt_u_imm16_rhs, wasm::i64_lt_u),
        (i64, Instruction::SelectI64LeSImm16Rhs, execute_select_i64_le_s_imm16_rhs, wasm::i64_le_s),
        (u64, Instruction::SelectI64LeUImm16Rhs, execute_select_i64_le_u_imm16_rhs, wasm::i64_le_u),
        (i64, Instruction::SelectI64AndImm16, execute_select_i64_and_imm16, UntypedValueExt::and),
        (i64, Instruction::SelectI64OrImm16, execute_select_i64_or_imm16, UntypedValueExt::or),
        (i64, Instruction::SelectI64XorImm16, execute_select_i64_xor_imm16, UntypedValueExt::xor),
        (i64, Instruction::SelectI64NandImm16, execute_select_i64_nand_imm16, UntypedValueExt::nand),
        (i64, Instruction::SelectI64NorImm16, execute_select_i64_nor_imm16, UntypedValueExt::nor),
        (i64, Instruction::SelectI64XnorImm16, execute_select_i64_xnor_imm16, UntypedValueExt::xnor),
    }

    impl_cmp_select_imm_lhs_for! {
        (i32, Instruction::SelectI32LtSImm16Lhs, execute_select_i32_lt_s_imm16_lhs, wasm::i32_lt_s),
        (u32, Instruction::SelectI32LtUImm16Lhs, execute_select_i32_lt_u_imm16_lhs, wasm::i32_lt_u),
        (i32, Instruction::SelectI32LeSImm16Lhs, execute_select_i32_le_s_imm16_lhs, wasm::i32_le_s),
        (u32, Instruction::SelectI32LeUImm16Lhs, execute_select_i32_le_u_imm16_lhs, wasm::i32_le_u),
        (i64, Instruction::SelectI64LtSImm16Lhs, execute_select_i64_lt_s_imm16_lhs, wasm::i64_lt_s),
        (u64, Instruction::SelectI64LtUImm16Lhs, execute_select_i64_lt_u_imm16_lhs, wasm::i64_lt_u),
        (i64, Instruction::SelectI64LeSImm16Lhs, execute_select_i64_le_s_imm16_lhs, wasm::i64_le_s),
        (u64, Instruction::SelectI64LeUImm16Lhs, execute_select_i64_le_u_imm16_lhs, wasm::i64_le_u),
    }
}
