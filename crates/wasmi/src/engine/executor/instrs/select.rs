use super::{Executor, InstructionPtr, UntypedValueExt};
use crate::{
    core::{wasm, ReadAs, UntypedVal},
    engine::utils::unreachable_unchecked,
    ir::{Const16, Op, Slot},
};

impl Executor<'_> {
    /// Fetches two [`Slot`]s.
    fn fetch_register_2(&self) -> (Slot, Slot) {
        let mut addr: InstructionPtr = self.ip;
        addr.add(1);
        match *addr.get() {
            Op::Slot2 { regs: [reg0, reg1] } => (reg0, reg1),
            unexpected => {
                // Safety: Wasmi translation guarantees that [`Op::Slot2`] exists.
                unsafe { unreachable_unchecked!("expected `Op::Slot2` but found {unexpected:?}") }
            }
        }
    }

    /// Executes a fused `cmp`+`select` instruction.
    #[inline(always)]
    fn execute_cmp_select_impl<T>(
        &mut self,
        result: Slot,
        lhs: Slot,
        rhs: Slot,
        f: fn(T, T) -> bool,
    ) where
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
        result: Slot,
        lhs: Slot,
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
}

macro_rules! impl_cmp_select_for {
    (
        $(
            (Op::$doc_name:ident, $fn_name:ident, $op:expr)
        ),* $(,)?
    ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($doc_name), "`].")]
            pub fn $fn_name(&mut self, result: Slot, lhs: Slot, rhs: Slot) {
                self.execute_cmp_select_impl(result, lhs, rhs, $op)
            }
        )*
    };
}

macro_rules! impl_cmp_select_imm_rhs_for {
    (
        $(
            ($ty:ty, Op::$doc_name:ident, $fn_name:ident, $op:expr)
        ),* $(,)?
    ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($doc_name), "`].")]
            pub fn $fn_name(&mut self, result: Slot, lhs: Slot, rhs: Const16<$ty>) {
                self.execute_cmp_select_imm_rhs_impl::<$ty>(result, lhs, rhs, $op)
            }
        )*
    };
}

impl Executor<'_> {
    impl_cmp_select_for! {
        (Op::SelectI32Eq, execute_select_i32_eq, wasm::i32_eq),
        (Op::SelectI32LtS, execute_select_i32_lt_s, wasm::i32_lt_s),
        (Op::SelectI32LtU, execute_select_i32_lt_u, wasm::i32_lt_u),
        (Op::SelectI32LeS, execute_select_i32_le_s, wasm::i32_le_s),
        (Op::SelectI32LeU, execute_select_i32_le_u, wasm::i32_le_u),
        (Op::SelectI32And, execute_select_i32_and, <i32 as UntypedValueExt>::and),
        (Op::SelectI32Or, execute_select_i32_or, <i32 as UntypedValueExt>::or),
        (Op::SelectI64Eq, execute_select_i64_eq, wasm::i64_eq),
        (Op::SelectI64LtS, execute_select_i64_lt_s, wasm::i64_lt_s),
        (Op::SelectI64LtU, execute_select_i64_lt_u, wasm::i64_lt_u),
        (Op::SelectI64LeS, execute_select_i64_le_s, wasm::i64_le_s),
        (Op::SelectI64LeU, execute_select_i64_le_u, wasm::i64_le_u),
        (Op::SelectI64And, execute_select_i64_and, <i64 as UntypedValueExt>::and),
        (Op::SelectI64Or, execute_select_i64_or, <i64 as UntypedValueExt>::or),
        (Op::SelectF32Eq, execute_select_f32_eq, wasm::f32_eq),
        (Op::SelectF32Lt, execute_select_f32_lt, wasm::f32_lt),
        (Op::SelectF32Le, execute_select_f32_le, wasm::f32_le),
        (Op::SelectF64Eq, execute_select_f64_eq, wasm::f64_eq),
        (Op::SelectF64Lt, execute_select_f64_lt, wasm::f64_lt),
        (Op::SelectF64Le, execute_select_f64_le, wasm::f64_le),
    }

    impl_cmp_select_imm_rhs_for! {
        (i32, Op::SelectI32EqImm16, execute_select_i32_eq_imm16, wasm::i32_eq),
        (i32, Op::SelectI32LtSImm16Rhs, execute_select_i32_lt_s_imm16_rhs, wasm::i32_lt_s),
        (u32, Op::SelectI32LtUImm16Rhs, execute_select_i32_lt_u_imm16_rhs, wasm::i32_lt_u),
        (i32, Op::SelectI32LeSImm16Rhs, execute_select_i32_le_s_imm16_rhs, wasm::i32_le_s),
        (u32, Op::SelectI32LeUImm16Rhs, execute_select_i32_le_u_imm16_rhs, wasm::i32_le_u),
        (i32, Op::SelectI32AndImm16, execute_select_i32_and_imm16, UntypedValueExt::and),
        (i32, Op::SelectI32OrImm16, execute_select_i32_or_imm16, UntypedValueExt::or),
        (i64, Op::SelectI64EqImm16, execute_select_i64_eq_imm16, wasm::i64_eq),
        (i64, Op::SelectI64LtSImm16Rhs, execute_select_i64_lt_s_imm16_rhs, wasm::i64_lt_s),
        (u64, Op::SelectI64LtUImm16Rhs, execute_select_i64_lt_u_imm16_rhs, wasm::i64_lt_u),
        (i64, Op::SelectI64LeSImm16Rhs, execute_select_i64_le_s_imm16_rhs, wasm::i64_le_s),
        (u64, Op::SelectI64LeUImm16Rhs, execute_select_i64_le_u_imm16_rhs, wasm::i64_le_u),
        (i64, Op::SelectI64AndImm16, execute_select_i64_and_imm16, UntypedValueExt::and),
        (i64, Op::SelectI64OrImm16, execute_select_i64_or_imm16, UntypedValueExt::or),
    }
}
