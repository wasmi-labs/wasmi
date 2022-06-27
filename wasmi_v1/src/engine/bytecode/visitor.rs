pub use super::{ExecRegister, ExecRegisterSlice, Global, Offset};
use super::{Instruction, InstructionTypes, Target};
use crate::module::{FuncIdx, FuncTypeIdx};
use wasmi_core::TrapCode;

/// Visits the `wasmi` instruction using the given `visitor`.
///
/// Returns the result of the visitation.
pub fn visit_instr<V, T>(
    visitor: &mut V,
    instr: Instruction<T>,
) -> <V as VisitInstruction<T>>::Outcome
where
    V: VisitInstruction<T>,
    T: InstructionTypes,
{
    use Instruction as Instr;
    match instr {
        Instr::Br { target } => visitor.visit_br(target),
        Instr::BrEqz { target, condition } => visitor.visit_br_eqz(target, condition),
        Instr::BrNez { target, condition } => visitor.visit_br_nez(target, condition),
        Instr::ReturnNez { results, condition } => visitor.visit_return_nez(results, condition),
        Instr::BrTable { case, len_targets } => visitor.visit_br_table(case, len_targets),
        Instr::Trap { trap_code } => visitor.visit_trap(trap_code),
        Instr::Return { results } => visitor.visit_return(results),
        Instr::Call {
            func_idx,
            results,
            params,
        } => visitor.visit_call(func_idx, results, params),
        Instr::CallIndirect {
            func_type_idx,
            results,
            index,
            params,
        } => visitor.visit_call_indirect(func_type_idx, results, index, params),
        Instr::Copy { result, input } => visitor.visit_copy(result, input),
        Instr::Select {
            result,
            condition,
            if_true,
            if_false,
        } => visitor.visit_select(result, condition, if_true, if_false),
        Instr::GlobalGet { result, global } => visitor.visit_global_get(result, global),
        Instr::GlobalSet { global, value } => visitor.visit_global_set(global, value),
        Instr::I32Load {
            result,
            ptr,
            offset,
        } => visitor.visit_i32_load(result, ptr, offset),
        Instr::I64Load {
            result,
            ptr,
            offset,
        } => visitor.visit_i64_load(result, ptr, offset),
        Instr::F32Load {
            result,
            ptr,
            offset,
        } => visitor.visit_f32_load(result, ptr, offset),
        Instr::F64Load {
            result,
            ptr,
            offset,
        } => visitor.visit_f64_load(result, ptr, offset),
        Instr::I32Load8S {
            result,
            ptr,
            offset,
        } => visitor.visit_i32_load_8_s(result, ptr, offset),
        Instr::I32Load8U {
            result,
            ptr,
            offset,
        } => visitor.visit_i32_load_8_u(result, ptr, offset),
        Instr::I32Load16S {
            result,
            ptr,
            offset,
        } => visitor.visit_i32_load_16_s(result, ptr, offset),
        Instr::I32Load16U {
            result,
            ptr,
            offset,
        } => visitor.visit_i32_load_16_u(result, ptr, offset),
        Instr::I64Load8S {
            result,
            ptr,
            offset,
        } => visitor.visit_i64_load_8_s(result, ptr, offset),
        Instr::I64Load8U {
            result,
            ptr,
            offset,
        } => visitor.visit_i64_load_8_u(result, ptr, offset),
        Instr::I64Load16S {
            result,
            ptr,
            offset,
        } => visitor.visit_i64_load_16_s(result, ptr, offset),
        Instr::I64Load16U {
            result,
            ptr,
            offset,
        } => visitor.visit_i64_load_16_u(result, ptr, offset),
        Instr::I64Load32S {
            result,
            ptr,
            offset,
        } => visitor.visit_i64_load_32_s(result, ptr, offset),
        Instr::I64Load32U {
            result,
            ptr,
            offset,
        } => visitor.visit_i64_load_32_u(result, ptr, offset),
        Instr::I32Store { ptr, offset, value } => visitor.visit_i32_store(ptr, offset, value),
        Instr::I64Store { ptr, offset, value } => visitor.visit_i64_store(ptr, offset, value),
        Instr::F32Store { ptr, offset, value } => visitor.visit_f32_store(ptr, offset, value),
        Instr::F64Store { ptr, offset, value } => visitor.visit_f64_store(ptr, offset, value),
        Instr::I32Store8 { ptr, offset, value } => visitor.visit_i32_store_8(ptr, offset, value),
        Instr::I32Store16 { ptr, offset, value } => visitor.visit_i32_store_16(ptr, offset, value),
        Instr::I64Store8 { ptr, offset, value } => visitor.visit_i64_store_8(ptr, offset, value),
        Instr::I64Store16 { ptr, offset, value } => visitor.visit_i64_store_16(ptr, offset, value),
        Instr::I64Store32 { ptr, offset, value } => visitor.visit_i64_store_32(ptr, offset, value),
        Instr::MemorySize { result } => visitor.visit_memory_size(result),
        Instr::MemoryGrow { result, amount } => visitor.visit_memory_grow(result, amount),
        Instr::I32Eq { result, lhs, rhs } => visitor.visit_i32_eq(result, lhs, rhs),
        Instr::I32Ne { result, lhs, rhs } => visitor.visit_i32_ne(result, lhs, rhs),
        Instr::I32LtS { result, lhs, rhs } => visitor.visit_i32_lt_s(result, lhs, rhs),
        Instr::I32LtU { result, lhs, rhs } => visitor.visit_i32_lt_u(result, lhs, rhs),
        Instr::I32GtS { result, lhs, rhs } => visitor.visit_i32_gt_s(result, lhs, rhs),
        Instr::I32GtU { result, lhs, rhs } => visitor.visit_i32_gt_u(result, lhs, rhs),
        Instr::I32LeS { result, lhs, rhs } => visitor.visit_i32_le_s(result, lhs, rhs),
        Instr::I32LeU { result, lhs, rhs } => visitor.visit_i32_le_u(result, lhs, rhs),
        Instr::I32GeS { result, lhs, rhs } => visitor.visit_i32_ge_s(result, lhs, rhs),
        Instr::I32GeU { result, lhs, rhs } => visitor.visit_i32_ge_u(result, lhs, rhs),
        Instr::I64Eq { result, lhs, rhs } => visitor.visit_i64_eq(result, lhs, rhs),
        Instr::I64Ne { result, lhs, rhs } => visitor.visit_i64_ne(result, lhs, rhs),
        Instr::I64LtS { result, lhs, rhs } => visitor.visit_i64_lt_s(result, lhs, rhs),
        Instr::I64LtU { result, lhs, rhs } => visitor.visit_i64_lt_u(result, lhs, rhs),
        Instr::I64GtS { result, lhs, rhs } => visitor.visit_i64_gt_s(result, lhs, rhs),
        Instr::I64GtU { result, lhs, rhs } => visitor.visit_i64_gt_u(result, lhs, rhs),
        Instr::I64LeS { result, lhs, rhs } => visitor.visit_i64_le_s(result, lhs, rhs),
        Instr::I64LeU { result, lhs, rhs } => visitor.visit_i64_le_u(result, lhs, rhs),
        Instr::I64GeS { result, lhs, rhs } => visitor.visit_i64_ge_s(result, lhs, rhs),
        Instr::I64GeU { result, lhs, rhs } => visitor.visit_i64_ge_u(result, lhs, rhs),
        Instr::F32Eq { result, lhs, rhs } => visitor.visit_f32_eq(result, lhs, rhs),
        Instr::F32Ne { result, lhs, rhs } => visitor.visit_f32_ne(result, lhs, rhs),
        Instr::F32Lt { result, lhs, rhs } => visitor.visit_f32_lt(result, lhs, rhs),
        Instr::F32Gt { result, lhs, rhs } => visitor.visit_f32_gt(result, lhs, rhs),
        Instr::F32Le { result, lhs, rhs } => visitor.visit_f32_le(result, lhs, rhs),
        Instr::F32Ge { result, lhs, rhs } => visitor.visit_f32_ge(result, lhs, rhs),
        Instr::F64Eq { result, lhs, rhs } => visitor.visit_f64_eq(result, lhs, rhs),
        Instr::F64Ne { result, lhs, rhs } => visitor.visit_f64_ne(result, lhs, rhs),
        Instr::F64Lt { result, lhs, rhs } => visitor.visit_f64_lt(result, lhs, rhs),
        Instr::F64Gt { result, lhs, rhs } => visitor.visit_f64_gt(result, lhs, rhs),
        Instr::F64Le { result, lhs, rhs } => visitor.visit_f64_le(result, lhs, rhs),
        Instr::F64Ge { result, lhs, rhs } => visitor.visit_f64_ge(result, lhs, rhs),
        Instr::I32Clz { result, input } => visitor.visit_i32_clz(result, input),
        Instr::I32Ctz { result, input } => visitor.visit_i32_ctz(result, input),
        Instr::I32Popcnt { result, input } => visitor.visit_i32_popcnt(result, input),
        Instr::I32Add { result, lhs, rhs } => visitor.visit_i32_add(result, lhs, rhs),
        Instr::I32Sub { result, lhs, rhs } => visitor.visit_i32_sub(result, lhs, rhs),
        Instr::I32Mul { result, lhs, rhs } => visitor.visit_i32_mul(result, lhs, rhs),
        Instr::I32DivS { result, lhs, rhs } => visitor.visit_i32_div_s(result, lhs, rhs),
        Instr::I32DivU { result, lhs, rhs } => visitor.visit_i32_div_u(result, lhs, rhs),
        Instr::I32RemS { result, lhs, rhs } => visitor.visit_i32_rem_s(result, lhs, rhs),
        Instr::I32RemU { result, lhs, rhs } => visitor.visit_i32_rem_u(result, lhs, rhs),
        Instr::I32And { result, lhs, rhs } => visitor.visit_i32_and(result, lhs, rhs),
        Instr::I32Or { result, lhs, rhs } => visitor.visit_i32_or(result, lhs, rhs),
        Instr::I32Xor { result, lhs, rhs } => visitor.visit_i32_xor(result, lhs, rhs),
        Instr::I32Shl { result, lhs, rhs } => visitor.visit_i32_shl(result, lhs, rhs),
        Instr::I32ShrS { result, lhs, rhs } => visitor.visit_i32_shr_s(result, lhs, rhs),
        Instr::I32ShrU { result, lhs, rhs } => visitor.visit_i32_shr_u(result, lhs, rhs),
        Instr::I32Rotl { result, lhs, rhs } => visitor.visit_i32_rotl(result, lhs, rhs),
        Instr::I32Rotr { result, lhs, rhs } => visitor.visit_i32_rotr(result, lhs, rhs),
        Instr::I64Clz { result, input } => visitor.visit_i64_clz(result, input),
        Instr::I64Ctz { result, input } => visitor.visit_i64_ctz(result, input),
        Instr::I64Popcnt { result, input } => visitor.visit_i64_popcnt(result, input),
        Instr::I64Add { result, lhs, rhs } => visitor.visit_i64_add(result, lhs, rhs),
        Instr::I64Sub { result, lhs, rhs } => visitor.visit_i64_sub(result, lhs, rhs),
        Instr::I64Mul { result, lhs, rhs } => visitor.visit_i64_mul(result, lhs, rhs),
        Instr::I64DivS { result, lhs, rhs } => visitor.visit_i64_div_s(result, lhs, rhs),
        Instr::I64DivU { result, lhs, rhs } => visitor.visit_i64_div_u(result, lhs, rhs),
        Instr::I64RemS { result, lhs, rhs } => visitor.visit_i64_rem_s(result, lhs, rhs),
        Instr::I64RemU { result, lhs, rhs } => visitor.visit_i64_rem_u(result, lhs, rhs),
        Instr::I64And { result, lhs, rhs } => visitor.visit_i64_and(result, lhs, rhs),
        Instr::I64Or { result, lhs, rhs } => visitor.visit_i64_or(result, lhs, rhs),
        Instr::I64Xor { result, lhs, rhs } => visitor.visit_i64_xor(result, lhs, rhs),
        Instr::I64Shl { result, lhs, rhs } => visitor.visit_i64_shl(result, lhs, rhs),
        Instr::I64ShrS { result, lhs, rhs } => visitor.visit_i64_shr_s(result, lhs, rhs),
        Instr::I64ShrU { result, lhs, rhs } => visitor.visit_i64_shr_u(result, lhs, rhs),
        Instr::I64Rotl { result, lhs, rhs } => visitor.visit_i64_rotl(result, lhs, rhs),
        Instr::I64Rotr { result, lhs, rhs } => visitor.visit_i64_rotr(result, lhs, rhs),
        Instr::F32Abs { result, input } => visitor.visit_f32_abs(result, input),
        Instr::F32Neg { result, input } => visitor.visit_f32_neg(result, input),
        Instr::F32Ceil { result, input } => visitor.visit_f32_ceil(result, input),
        Instr::F32Floor { result, input } => visitor.visit_f32_floor(result, input),
        Instr::F32Trunc { result, input } => visitor.visit_f32_trunc(result, input),
        Instr::F32Nearest { result, input } => visitor.visit_f32_nearest(result, input),
        Instr::F32Sqrt { result, input } => visitor.visit_f32_sqrt(result, input),
        Instr::F32Add { result, lhs, rhs } => visitor.visit_f32_add(result, lhs, rhs),
        Instr::F32Sub { result, lhs, rhs } => visitor.visit_f32_sub(result, lhs, rhs),
        Instr::F32Mul { result, lhs, rhs } => visitor.visit_f32_mul(result, lhs, rhs),
        Instr::F32Div { result, lhs, rhs } => visitor.visit_f32_div(result, lhs, rhs),
        Instr::F32Min { result, lhs, rhs } => visitor.visit_f32_min(result, lhs, rhs),
        Instr::F32Max { result, lhs, rhs } => visitor.visit_f32_max(result, lhs, rhs),
        Instr::F32Copysign { result, lhs, rhs } => visitor.visit_f32_copysign(result, lhs, rhs),
        Instr::F64Abs { result, input } => visitor.visit_f64_abs(result, input),
        Instr::F64Neg { result, input } => visitor.visit_f64_neg(result, input),
        Instr::F64Ceil { result, input } => visitor.visit_f64_ceil(result, input),
        Instr::F64Floor { result, input } => visitor.visit_f64_floor(result, input),
        Instr::F64Trunc { result, input } => visitor.visit_f64_trunc(result, input),
        Instr::F64Nearest { result, input } => visitor.visit_f64_nearest(result, input),
        Instr::F64Sqrt { result, input } => visitor.visit_f64_sqrt(result, input),
        Instr::F64Add { result, lhs, rhs } => visitor.visit_f64_add(result, lhs, rhs),
        Instr::F64Sub { result, lhs, rhs } => visitor.visit_f64_sub(result, lhs, rhs),
        Instr::F64Mul { result, lhs, rhs } => visitor.visit_f64_mul(result, lhs, rhs),
        Instr::F64Div { result, lhs, rhs } => visitor.visit_f64_div(result, lhs, rhs),
        Instr::F64Min { result, lhs, rhs } => visitor.visit_f64_min(result, lhs, rhs),
        Instr::F64Max { result, lhs, rhs } => visitor.visit_f64_max(result, lhs, rhs),
        Instr::F64Copysign { result, lhs, rhs } => visitor.visit_f64_copysign(result, lhs, rhs),
        Instr::I32WrapI64 { result, input } => visitor.visit_i32_wrap_i64(result, input),
        Instr::I32TruncSF32 { result, input } => visitor.visit_i32_trunc_f32_s(result, input),
        Instr::I32TruncUF32 { result, input } => visitor.visit_i32_trunc_f32_u(result, input),
        Instr::I32TruncSF64 { result, input } => visitor.visit_i32_trunc_f64_s(result, input),
        Instr::I32TruncUF64 { result, input } => visitor.visit_i32_trunc_f64_u(result, input),
        Instr::I64ExtendSI32 { result, input } => visitor.visit_i64_extend_i32_s(result, input),
        Instr::I64ExtendUI32 { result, input } => visitor.visit_i64_extend_i32_u(result, input),
        Instr::I64TruncSF32 { result, input } => visitor.visit_i64_trunc_f32_s(result, input),
        Instr::I64TruncUF32 { result, input } => visitor.visit_i64_trunc_f32_u(result, input),
        Instr::I64TruncSF64 { result, input } => visitor.visit_i64_trunc_f64_s(result, input),
        Instr::I64TruncUF64 { result, input } => visitor.visit_i64_trunc_f64_u(result, input),
        Instr::F32ConvertSI32 { result, input } => visitor.visit_f32_convert_i32_s(result, input),
        Instr::F32ConvertUI32 { result, input } => visitor.visit_f32_convert_i32_u(result, input),
        Instr::F32ConvertSI64 { result, input } => visitor.visit_f32_convert_i64_s(result, input),
        Instr::F32ConvertUI64 { result, input } => visitor.visit_f32_convert_i64_u(result, input),
        Instr::F32DemoteF64 { result, input } => visitor.visit_f32_demote_f64(result, input),
        Instr::F64ConvertSI32 { result, input } => visitor.visit_f64_convert_i32_s(result, input),
        Instr::F64ConvertUI32 { result, input } => visitor.visit_f64_convert_i32_u(result, input),
        Instr::F64ConvertSI64 { result, input } => visitor.visit_f64_convert_i64_s(result, input),
        Instr::F64ConvertUI64 { result, input } => visitor.visit_f64_convert_i64_u(result, input),
        Instr::F64PromoteF32 { result, input } => visitor.visit_f64_promote_f32(result, input),
        Instr::I32Extend8S { result, input } => visitor.visit_i32_extend8_s(result, input),
        Instr::I32Extend16S { result, input } => visitor.visit_i32_extend16_s(result, input),
        Instr::I64Extend8S { result, input } => visitor.visit_i64_extend8_s(result, input),
        Instr::I64Extend16S { result, input } => visitor.visit_i64_extend16_s(result, input),
        Instr::I64Extend32S { result, input } => visitor.visit_i64_extend32_s(result, input),
        Instr::I32TruncSatF32S { result, input } => {
            visitor.visit_i32_trunc_sat_f32_s(result, input)
        }
        Instr::I32TruncSatF32U { result, input } => {
            visitor.visit_i32_trunc_sat_f32_u(result, input)
        }
        Instr::I32TruncSatF64S { result, input } => {
            visitor.visit_i32_trunc_sat_f64_s(result, input)
        }
        Instr::I32TruncSatF64U { result, input } => {
            visitor.visit_i32_trunc_sat_f64_u(result, input)
        }
        Instr::I64TruncSatF32S { result, input } => {
            visitor.visit_i64_trunc_sat_f32_s(result, input)
        }
        Instr::I64TruncSatF32U { result, input } => {
            visitor.visit_i64_trunc_sat_f32_u(result, input)
        }
        Instr::I64TruncSatF64S { result, input } => {
            visitor.visit_i64_trunc_sat_f64_s(result, input)
        }
        Instr::I64TruncSatF64U { result, input } => {
            visitor.visit_i64_trunc_sat_f64_u(result, input)
        }
    }
}

/// An instruction visitor.
///
/// Types implementing this trait can visit all `wasmi` instructions.
pub trait VisitInstruction<T>
where
    T: InstructionTypes,
{
    /// The result of all visitor functions.
    type Outcome;

    /// Visits the `wasmi` `br` instruction.
    fn visit_br(&mut self, target: Target) -> Self::Outcome;

    /// Visits the `wasmi` `br_eqz` instruction.
    fn visit_br_eqz(
        &mut self,
        target: Target,
        condition: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `br_nez` instruction.
    fn visit_br_nez(
        &mut self,
        target: Target,
        condition: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `return_nez` instruction.
    fn visit_return_nez(
        &mut self,
        results: <T as InstructionTypes>::ProviderSlice,
        condition: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `br_table` instruction.
    fn visit_br_table(
        &mut self,
        case: <T as InstructionTypes>::Register,
        len_targets: usize,
    ) -> Self::Outcome;

    /// Vitis the `wasmi` `trap` instruction.
    fn visit_trap(&mut self, trap_code: TrapCode) -> Self::Outcome;

    /// Visits the `wasmi` `return` instruction.
    fn visit_return(&mut self, results: <T as InstructionTypes>::ProviderSlice) -> Self::Outcome;

    /// Visits the `wasmi` `call` instruction.
    fn visit_call(
        &mut self,
        func: FuncIdx,
        results: <T as InstructionTypes>::RegisterSlice,
        params: <T as InstructionTypes>::ProviderSlice,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `call_indirect` instruction.
    fn visit_call_indirect(
        &mut self,
        func_type: FuncTypeIdx,
        results: <T as InstructionTypes>::RegisterSlice,
        index: <T as InstructionTypes>::Provider,
        params: <T as InstructionTypes>::ProviderSlice,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `copy` instruction.
    fn visit_copy(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `select` instruction.
    fn visit_select(
        &mut self,
        result: <T as InstructionTypes>::Register,
        condition: <T as InstructionTypes>::Register,
        if_true: <T as InstructionTypes>::Provider,
        if_false: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `global.get` instruction.
    fn visit_global_get(
        &mut self,
        result: <T as InstructionTypes>::Register,
        global: Global,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `global.set` instruction.
    fn visit_global_set(
        &mut self,
        global: Global,
        value: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.load` instruction.
    fn visit_i32_load(
        &mut self,
        result: <T as InstructionTypes>::Register,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.load` instruction.
    fn visit_i64_load(
        &mut self,
        result: <T as InstructionTypes>::Register,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.load` instruction.
    fn visit_f32_load(
        &mut self,
        result: <T as InstructionTypes>::Register,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.load` instruction.
    fn visit_f64_load(
        &mut self,
        result: <T as InstructionTypes>::Register,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.load_8_s` instruction.
    fn visit_i32_load_8_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.load_8_u` instruction.
    fn visit_i32_load_8_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.load_16_s` instruction.
    fn visit_i32_load_16_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.load_16_u` instruction.
    fn visit_i32_load_16_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.load_8_s` instruction.
    fn visit_i64_load_8_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.load_8_u` instruction.
    fn visit_i64_load_8_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.load_16_s` instruction.
    fn visit_i64_load_16_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.load_16_u` instruction.
    fn visit_i64_load_16_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.load_32_s` instruction.
    fn visit_i64_load_32_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.load_32_u` instruction.
    fn visit_i64_load_32_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.store` instruction.
    fn visit_i32_store(
        &mut self,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
        value: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.store` instruction.
    fn visit_i64_store(
        &mut self,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
        value: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.store` instruction.
    fn visit_f32_store(
        &mut self,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
        value: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.store` instruction.
    fn visit_f64_store(
        &mut self,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
        value: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.store_8` instruction.
    fn visit_i32_store_8(
        &mut self,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
        value: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.store_16` instruction.
    fn visit_i32_store_16(
        &mut self,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
        value: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.store_8` instruction.
    fn visit_i64_store_8(
        &mut self,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
        value: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.store_16` instruction.
    fn visit_i64_store_16(
        &mut self,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
        value: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.store_32` instruction.
    fn visit_i64_store_32(
        &mut self,
        ptr: <T as InstructionTypes>::Register,
        offset: Offset,
        value: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `memory.size` instruction.
    fn visit_memory_size(&mut self, result: <T as InstructionTypes>::Register) -> Self::Outcome;

    /// Visits the `wasmi` `memory.grow` instruction.
    fn visit_memory_grow(
        &mut self,
        result: <T as InstructionTypes>::Register,
        amount: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.eq` instruction.
    fn visit_i32_eq(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.ne` instruction.
    fn visit_i32_ne(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.lt_s` instruction.
    fn visit_i32_lt_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.lt_u` instruction.
    fn visit_i32_lt_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.gt_s` instruction.
    fn visit_i32_gt_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.gt_u` instruction.
    fn visit_i32_gt_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.le_s` instruction.
    fn visit_i32_le_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.le_u` instruction.
    fn visit_i32_le_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.ge_s` instruction.
    fn visit_i32_ge_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.ge_u` instruction.
    fn visit_i32_ge_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.eq` instruction.
    fn visit_i64_eq(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.ne` instruction.
    fn visit_i64_ne(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.lt_s` instruction.
    fn visit_i64_lt_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.lt_u` instruction.
    fn visit_i64_lt_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.gt_s` instruction.
    fn visit_i64_gt_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.gt_u` instruction.
    fn visit_i64_gt_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.le_s` instruction.
    fn visit_i64_le_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.le_u` instruction.
    fn visit_i64_le_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.ge_s` instruction.
    fn visit_i64_ge_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.ge_u` instruction.
    fn visit_i64_ge_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.eq` instruction.
    fn visit_f32_eq(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.ne` instruction.
    fn visit_f32_ne(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.lt` instruction.
    fn visit_f32_lt(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.gt` instruction.
    fn visit_f32_gt(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.le` instruction.
    fn visit_f32_le(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.ge` instruction.
    fn visit_f32_ge(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.eq` instruction.
    fn visit_f64_eq(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.ne` instruction.
    fn visit_f64_ne(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.lt` instruction.
    fn visit_f64_lt(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.gt` instruction.
    fn visit_f64_gt(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.le` instruction.
    fn visit_f64_le(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.ge` instruction.
    fn visit_f64_ge(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.clz` instruction.
    fn visit_i32_clz(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.ctz` instruction.
    fn visit_i32_ctz(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.popcnt` instruction.
    fn visit_i32_popcnt(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.add` instruction.
    fn visit_i32_add(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.sub` instruction.
    fn visit_i32_sub(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.mul` instruction.
    fn visit_i32_mul(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.div_s` instruction.
    fn visit_i32_div_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.div_u` instruction.
    fn visit_i32_div_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.rem_s` instruction.
    fn visit_i32_rem_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.rem_u` instruction.
    fn visit_i32_rem_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.and` instruction.
    fn visit_i32_and(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.or` instruction.
    fn visit_i32_or(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.xor` instruction.
    fn visit_i32_xor(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.shl` instruction.
    fn visit_i32_shl(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.shr_s` instruction.
    fn visit_i32_shr_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.shr_u` instruction.
    fn visit_i32_shr_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.rotl` instruction.
    fn visit_i32_rotl(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.rotr` instruction.
    fn visit_i32_rotr(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.clz` instruction.
    fn visit_i64_clz(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.ctz` instruction.
    fn visit_i64_ctz(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.popcnt` instruction.
    fn visit_i64_popcnt(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.add` instruction.
    fn visit_i64_add(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.sub` instruction.
    fn visit_i64_sub(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.mul` instruction.
    fn visit_i64_mul(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.div_s` instruction.
    fn visit_i64_div_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.div_u` instruction.
    fn visit_i64_div_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.rem_s` instruction.
    fn visit_i64_rem_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.rem_u` instruction.
    fn visit_i64_rem_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.and` instruction.
    fn visit_i64_and(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.or` instruction.
    fn visit_i64_or(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.xor` instruction.
    fn visit_i64_xor(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.shl` instruction.
    fn visit_i64_shl(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.shr_s` instruction.
    fn visit_i64_shr_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.shr_u` instruction.
    fn visit_i64_shr_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.rotl` instruction.
    fn visit_i64_rotl(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.rotr` instruction.
    fn visit_i64_rotr(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.abs` instruction.
    fn visit_f32_abs(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.neg` instruction.
    fn visit_f32_neg(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.ceil` instruction.
    fn visit_f32_ceil(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.floor` instruction.
    fn visit_f32_floor(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.trunc` instruction.
    fn visit_f32_trunc(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.nearest` instruction.
    fn visit_f32_nearest(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.sqrt` instruction.
    fn visit_f32_sqrt(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.add` instruction.
    fn visit_f32_add(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.sub` instruction.
    fn visit_f32_sub(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.mul` instruction.
    fn visit_f32_mul(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.div` instruction.
    fn visit_f32_div(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.min` instruction.
    fn visit_f32_min(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.max` instruction.
    fn visit_f32_max(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.copysign` instruction.
    fn visit_f32_copysign(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.abs` instruction.
    fn visit_f64_abs(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.neg` instruction.
    fn visit_f64_neg(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.ceil` instruction.
    fn visit_f64_ceil(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.floor` instruction.
    fn visit_f64_floor(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.trunc` instruction.
    fn visit_f64_trunc(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.nearest` instruction.
    fn visit_f64_nearest(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.sqrt` instruction.
    fn visit_f64_sqrt(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.add` instruction.
    fn visit_f64_add(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.sub` instruction.
    fn visit_f64_sub(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.mul` instruction.
    fn visit_f64_mul(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.div` instruction.
    fn visit_f64_div(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.min` instruction.
    fn visit_f64_min(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.max` instruction.
    fn visit_f64_max(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.copysign` instruction.
    fn visit_f64_copysign(
        &mut self,
        result: <T as InstructionTypes>::Register,
        lhs: <T as InstructionTypes>::Register,
        rhs: <T as InstructionTypes>::Provider,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.wrap_i64` instruction.
    fn visit_i32_wrap_i64(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.trunc_f32_s` instruction.
    fn visit_i32_trunc_f32_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.trunc_f32_u` instruction.
    fn visit_i32_trunc_f32_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.trunc_f64_s` instruction.
    fn visit_i32_trunc_f64_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.trunc_f64_u` instruction.
    fn visit_i32_trunc_f64_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.extend_i32_s` instruction.
    fn visit_i64_extend_i32_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.extend_i32_u` instruction.
    fn visit_i64_extend_i32_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.trunc_f32_s` instruction.
    fn visit_i64_trunc_f32_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.trunc_f32_u` instruction.
    fn visit_i64_trunc_f32_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.trunc_f64_s` instruction.
    fn visit_i64_trunc_f64_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.trunc_f64_u` instruction.
    fn visit_i64_trunc_f64_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.convert_i32_s` instruction.
    fn visit_f32_convert_i32_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.convert_i32_u` instruction.
    fn visit_f32_convert_i32_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.convert_i64_s` instruction.
    fn visit_f32_convert_i64_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.convert_i64_u` instruction.
    fn visit_f32_convert_i64_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f32.demote_f64` instruction.
    fn visit_f32_demote_f64(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.converti32_s` instruction.
    fn visit_f64_convert_i32_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.converti32_u` instruction.
    fn visit_f64_convert_i32_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.converti64_s` instruction.
    fn visit_f64_convert_i64_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.converti64_u` instruction.
    fn visit_f64_convert_i64_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `f64.promote_f32` instruction.
    fn visit_f64_promote_f32(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.extend_8_s` instruction.
    fn visit_i32_extend8_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.extend_16_s` instruction.
    fn visit_i32_extend16_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.extend_8_s` instruction.
    fn visit_i64_extend8_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.extend_16_s` instruction.
    fn visit_i64_extend16_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.extend_32_s` instruction.
    fn visit_i64_extend32_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.trunc_sat_f32_s` instruction.
    fn visit_i32_trunc_sat_f32_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.trunc_sat_f32_u` instruction.
    fn visit_i32_trunc_sat_f32_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.trunc_sat_f64_s` instruction.
    fn visit_i32_trunc_sat_f64_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i32.trunc_sat_f64_u` instruction.
    fn visit_i32_trunc_sat_f64_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.trunc_sat_f32_s` instruction.
    fn visit_i64_trunc_sat_f32_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.trunc_sat_f32_u` instruction.
    fn visit_i64_trunc_sat_f32_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.trunc_sat_f64_s` instruction.
    fn visit_i64_trunc_sat_f64_s(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;

    /// Visits the `wasmi` `i64.trunc_sat_f64_u` instruction.
    fn visit_i64_trunc_sat_f64_u(
        &mut self,
        result: <T as InstructionTypes>::Register,
        input: <T as InstructionTypes>::Register,
    ) -> Self::Outcome;
}
