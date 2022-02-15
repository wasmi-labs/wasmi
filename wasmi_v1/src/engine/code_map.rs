//! Datastructure to efficiently store function bodies and their instructions.

use super::{
    super::Index,
    bytecode::{BrTable, VisitInstruction},
    Instruction,
};
use alloc::vec::Vec;
use core::iter;

/// A reference to a Wasm function body stored in the [`CodeMap`].
#[derive(Debug, Copy, Clone)]
pub struct FuncBody(usize);

impl Index for FuncBody {
    fn into_usize(self) -> usize {
        self.0
    }

    fn from_usize(value: usize) -> Self {
        FuncBody(value)
    }
}

/// Datastructure to efficiently store Wasm function bodies.
#[derive(Debug, Default)]
pub struct CodeMap {
    /// The instructions of all allocated function bodies.
    ///
    /// By storing all `wasmi` bytecode instructions in a single
    /// allocation we avoid an indirection when calling a function
    /// compared to a solution that stores instructions of different
    /// function bodies in different allocations.
    ///
    /// Also this improves efficiency of deallocating the [`CodeMap`]
    /// and generally improves data locality.
    insts: Vec<Instruction>,
}

impl CodeMap {
    /// Returns the next [`FuncBody`] index.
    fn next_index(&self) -> FuncBody {
        FuncBody(self.insts.len())
    }

    /// Allocates a new function body to the [`CodeMap`].
    ///
    /// Returns a reference to the allocated function body that can
    /// be used with [`CodeMap::resolve`] in order to resolve its
    /// instructions.
    pub fn alloc<I>(&mut self, len_locals: usize, max_stack_height: usize, insts: I) -> FuncBody
    where
        I: IntoIterator<Item = Instruction>,
        I::IntoIter: ExactSizeIterator,
    {
        let idx = self.next_index();
        // We are inserting an artificial `unreachable` Wasm instruction
        // in between instructions of different function bodies as a small
        // safety precaution.
        let insts = insts.into_iter();
        let len_instructions = insts.len().try_into().unwrap_or_else(|error| {
            panic!(
                "encountered too many instructions (= {}) for function: {}",
                insts.len(),
                error
            )
        });
        let max_stack_height = (max_stack_height + len_locals)
            .try_into()
            .unwrap_or_else(|error| {
                panic!(
                "encountered function that requires too many stack values (= {}) for function: {}",
                max_stack_height, error
            )
            });
        let len_locals = len_locals.try_into().unwrap_or_else(|error| {
            panic!(
                "encountered too many local variables (= {}) for function: {}",
                len_locals, error
            )
        });
        let start = iter::once(Instruction::FuncBodyStart {
            len_instructions,
            len_locals,
            max_stack_height,
        });
        let end = iter::once(Instruction::FuncBodyEnd);
        self.insts.extend(start.chain(insts).chain(end));
        idx
    }

    /// Resolves the instruction of the function body.
    ///
    /// # Panics
    ///
    /// If the given `func_body` is invalid for this [`CodeMap`].
    pub fn resolve(&self, func_body: FuncBody) -> ResolvedFuncBody {
        let offset = func_body.into_usize();
        let (len_instructions, len_locals, max_stack_height) = match &self.insts[offset] {
            Instruction::FuncBodyStart {
                len_instructions,
                len_locals,
                max_stack_height,
            } => (*len_instructions, *len_locals, *max_stack_height),
            unexpected => panic!(
                "expected function start instruction but found: {:?}",
                unexpected
            ),
        };
        let len_instructions = len_instructions as usize;
        let len_locals = len_locals as usize;
        let max_stack_height = max_stack_height as usize;
        // The index of the first instruction in the function body.
        let first_inst = offset + 1;
        {
            // Assert that the end of the function instructions is
            // properly guarded with the `FuncBodyEnd` sentinel.
            //
            // This check is not needed to validate the integrity of
            // the resolution procedure and therefore the below assertion
            // is only performed in debug mode.
            let end = &self.insts[first_inst + len_instructions];
            debug_assert!(
                matches!(end, Instruction::FuncBodyEnd),
                "expected function end instruction but found: {:?}",
                end,
            );
        }
        let insts = &self.insts[first_inst..(first_inst + len_instructions)];
        ResolvedFuncBody {
            insts,
            len_locals,
            max_stack_height,
        }
    }
}

/// A resolved Wasm function body that is stored in a [`CodeMap`].
///
/// Allows to immutably access the `wasmi` instructions of a Wasm
/// function stored in the [`CodeMap`].
///
/// # Dev. Note
///
/// This does not include the [`Instruction::FuncBodyStart`] and
/// [`Instruction::FuncBodyEnd`] instructions surrounding the instructions
/// of a function body in the [`CodeMap`].
#[derive(Debug, Copy, Clone)]
pub struct ResolvedFuncBody<'a> {
    insts: &'a [Instruction],
    len_locals: usize,
    max_stack_height: usize,
}

impl ResolvedFuncBody<'_> {
    /// Returns the instruction at the given index.
    ///
    /// # Panics
    ///
    /// If there is no instruction at the given index.
    #[cfg(test)]
    pub fn get(&self, index: usize) -> Option<&Instruction> {
        self.insts.get(index)
    }

    /// Returns the amount of local variable of the function.
    pub fn len_locals(&self) -> usize {
        self.len_locals
    }

    /// Returns the amount of stack values required by the function.
    ///
    /// # Note
    ///
    /// This amount includes the amount of local variables but does
    /// _not_ include the amount of input parameters to the function.
    pub fn max_stack_height(&self) -> usize {
        self.max_stack_height
    }
}

impl<'a> ResolvedFuncBody<'a> {
    /// Visits the corresponding [`Instruction`] method from the given visitor.
    ///
    /// Returns the visitor's outcome value.
    #[inline(always)]
    pub fn visit<T>(&self, index: usize, mut visitor: T) -> T::Outcome
    where
        T: VisitInstruction,
    {
        debug_assert!(
            self.insts.get(index).is_some(),
            "expect to find instruction at index {} due to validation but found none",
            index
        );
        // # Safety
        //
        // This access is safe since all possible accesses have already been
        // checked during Wasm validation. Functions and their instructions including
        // jump addresses are immutable after Wasm function compilation and validation
        // and therefore this bounds check can be safely eliminated.
        //
        // Note that eliminating this bounds check is extremely valuable since this
        // part of the `wasmi` interpreter is part of the interpreter's hot path.
        let inst = unsafe { self.insts.get_unchecked(index) };
        match inst {
            Instruction::GetLocal { local_depth } => visitor.visit_get_local(*local_depth),
            Instruction::SetLocal { local_depth } => visitor.visit_set_local(*local_depth),
            Instruction::TeeLocal { local_depth } => visitor.visit_tee_local(*local_depth),
            Instruction::Br(target) => visitor.visit_br(*target),
            Instruction::BrIfEqz(target) => visitor.visit_br_if_eqz(*target),
            Instruction::BrIfNez(target) => visitor.visit_br_if_nez(*target),
            Instruction::ReturnIfNez(drop_keep) => visitor.visit_return_if_nez(*drop_keep),
            Instruction::BrTable { len_targets } => visitor.visit_br_table(BrTable::new(
                &self.insts[(index + 1)..(index + 1 + len_targets)],
            )),
            Instruction::Unreachable => visitor.visit_unreachable(),
            Instruction::Return(drop_keep) => visitor.visit_ret(*drop_keep),
            Instruction::Call(func) => visitor.visit_call(*func),
            Instruction::CallIndirect(signature) => visitor.visit_call_indirect(*signature),
            Instruction::Drop => visitor.visit_drop(),
            Instruction::Select => visitor.visit_select(),
            Instruction::GetGlobal(global_idx) => visitor.visit_get_global(*global_idx),
            Instruction::SetGlobal(global_idx) => visitor.visit_set_global(*global_idx),
            Instruction::I32Load(offset) => visitor.visit_i32_load(*offset),
            Instruction::I64Load(offset) => visitor.visit_i64_load(*offset),
            Instruction::F32Load(offset) => visitor.visit_f32_load(*offset),
            Instruction::F64Load(offset) => visitor.visit_f64_load(*offset),
            Instruction::I32Load8S(offset) => visitor.visit_i32_load_i8(*offset),
            Instruction::I32Load8U(offset) => visitor.visit_i32_load_u8(*offset),
            Instruction::I32Load16S(offset) => visitor.visit_i32_load_i16(*offset),
            Instruction::I32Load16U(offset) => visitor.visit_i32_load_u16(*offset),
            Instruction::I64Load8S(offset) => visitor.visit_i64_load_i8(*offset),
            Instruction::I64Load8U(offset) => visitor.visit_i64_load_u8(*offset),
            Instruction::I64Load16S(offset) => visitor.visit_i64_load_i16(*offset),
            Instruction::I64Load16U(offset) => visitor.visit_i64_load_u16(*offset),
            Instruction::I64Load32S(offset) => visitor.visit_i64_load_i32(*offset),
            Instruction::I64Load32U(offset) => visitor.visit_i64_load_u32(*offset),
            Instruction::I32Store(offset) => visitor.visit_i32_store(*offset),
            Instruction::I64Store(offset) => visitor.visit_i64_store(*offset),
            Instruction::F32Store(offset) => visitor.visit_f32_store(*offset),
            Instruction::F64Store(offset) => visitor.visit_f64_store(*offset),
            Instruction::I32Store8(offset) => visitor.visit_i32_store_8(*offset),
            Instruction::I32Store16(offset) => visitor.visit_i32_store_16(*offset),
            Instruction::I64Store8(offset) => visitor.visit_i64_store_8(*offset),
            Instruction::I64Store16(offset) => visitor.visit_i64_store_16(*offset),
            Instruction::I64Store32(offset) => visitor.visit_i64_store_32(*offset),
            Instruction::CurrentMemory => visitor.visit_current_memory(),
            Instruction::GrowMemory => visitor.visit_grow_memory(),
            Instruction::Const(bytes) => visitor.visit_const(*bytes),
            Instruction::I32Eqz => visitor.visit_i32_eqz(),
            Instruction::I32Eq => visitor.visit_i32_eq(),
            Instruction::I32Ne => visitor.visit_i32_ne(),
            Instruction::I32LtS => visitor.visit_i32_lt_s(),
            Instruction::I32LtU => visitor.visit_i32_lt_u(),
            Instruction::I32GtS => visitor.visit_i32_gt_s(),
            Instruction::I32GtU => visitor.visit_i32_gt_u(),
            Instruction::I32LeS => visitor.visit_i32_le_s(),
            Instruction::I32LeU => visitor.visit_i32_le_u(),
            Instruction::I32GeS => visitor.visit_i32_ge_s(),
            Instruction::I32GeU => visitor.visit_i32_ge_u(),
            Instruction::I64Eqz => visitor.visit_i64_eqz(),
            Instruction::I64Eq => visitor.visit_i64_eq(),
            Instruction::I64Ne => visitor.visit_i64_ne(),
            Instruction::I64LtS => visitor.visit_i64_lt_s(),
            Instruction::I64LtU => visitor.visit_i64_lt_u(),
            Instruction::I64GtS => visitor.visit_i64_gt_s(),
            Instruction::I64GtU => visitor.visit_i64_gt_u(),
            Instruction::I64LeS => visitor.visit_i64_le_s(),
            Instruction::I64LeU => visitor.visit_i64_le_u(),
            Instruction::I64GeS => visitor.visit_i64_ge_s(),
            Instruction::I64GeU => visitor.visit_i64_ge_u(),
            Instruction::F32Eq => visitor.visit_f32_eq(),
            Instruction::F32Ne => visitor.visit_f32_ne(),
            Instruction::F32Lt => visitor.visit_f32_lt(),
            Instruction::F32Gt => visitor.visit_f32_gt(),
            Instruction::F32Le => visitor.visit_f32_le(),
            Instruction::F32Ge => visitor.visit_f32_ge(),
            Instruction::F64Eq => visitor.visit_f64_eq(),
            Instruction::F64Ne => visitor.visit_f64_ne(),
            Instruction::F64Lt => visitor.visit_f64_lt(),
            Instruction::F64Gt => visitor.visit_f64_gt(),
            Instruction::F64Le => visitor.visit_f64_le(),
            Instruction::F64Ge => visitor.visit_f64_ge(),
            Instruction::I32Clz => visitor.visit_i32_clz(),
            Instruction::I32Ctz => visitor.visit_i32_ctz(),
            Instruction::I32Popcnt => visitor.visit_i32_popcnt(),
            Instruction::I32Add => visitor.visit_i32_add(),
            Instruction::I32Sub => visitor.visit_i32_sub(),
            Instruction::I32Mul => visitor.visit_i32_mul(),
            Instruction::I32DivS => visitor.visit_i32_div_s(),
            Instruction::I32DivU => visitor.visit_i32_div_u(),
            Instruction::I32RemS => visitor.visit_i32_rem_s(),
            Instruction::I32RemU => visitor.visit_i32_rem_u(),
            Instruction::I32And => visitor.visit_i32_and(),
            Instruction::I32Or => visitor.visit_i32_or(),
            Instruction::I32Xor => visitor.visit_i32_xor(),
            Instruction::I32Shl => visitor.visit_i32_shl(),
            Instruction::I32ShrS => visitor.visit_i32_shr_s(),
            Instruction::I32ShrU => visitor.visit_i32_shr_u(),
            Instruction::I32Rotl => visitor.visit_i32_rotl(),
            Instruction::I32Rotr => visitor.visit_i32_rotr(),
            Instruction::I64Clz => visitor.visit_i64_clz(),
            Instruction::I64Ctz => visitor.visit_i64_ctz(),
            Instruction::I64Popcnt => visitor.visit_i64_popcnt(),
            Instruction::I64Add => visitor.visit_i64_add(),
            Instruction::I64Sub => visitor.visit_i64_sub(),
            Instruction::I64Mul => visitor.visit_i64_mul(),
            Instruction::I64DivS => visitor.visit_i64_div_s(),
            Instruction::I64DivU => visitor.visit_i64_div_u(),
            Instruction::I64RemS => visitor.visit_i64_rem_s(),
            Instruction::I64RemU => visitor.visit_i64_rem_u(),
            Instruction::I64And => visitor.visit_i64_and(),
            Instruction::I64Or => visitor.visit_i64_or(),
            Instruction::I64Xor => visitor.visit_i64_xor(),
            Instruction::I64Shl => visitor.visit_i64_shl(),
            Instruction::I64ShrS => visitor.visit_i64_shr_s(),
            Instruction::I64ShrU => visitor.visit_i64_shr_u(),
            Instruction::I64Rotl => visitor.visit_i64_rotl(),
            Instruction::I64Rotr => visitor.visit_i64_rotr(),
            Instruction::F32Abs => visitor.visit_f32_abs(),
            Instruction::F32Neg => visitor.visit_f32_neg(),
            Instruction::F32Ceil => visitor.visit_f32_ceil(),
            Instruction::F32Floor => visitor.visit_f32_floor(),
            Instruction::F32Trunc => visitor.visit_f32_trunc(),
            Instruction::F32Nearest => visitor.visit_f32_nearest(),
            Instruction::F32Sqrt => visitor.visit_f32_sqrt(),
            Instruction::F32Add => visitor.visit_f32_add(),
            Instruction::F32Sub => visitor.visit_f32_sub(),
            Instruction::F32Mul => visitor.visit_f32_mul(),
            Instruction::F32Div => visitor.visit_f32_div(),
            Instruction::F32Min => visitor.visit_f32_min(),
            Instruction::F32Max => visitor.visit_f32_max(),
            Instruction::F32Copysign => visitor.visit_f32_copysign(),
            Instruction::F64Abs => visitor.visit_f64_abs(),
            Instruction::F64Neg => visitor.visit_f64_neg(),
            Instruction::F64Ceil => visitor.visit_f64_ceil(),
            Instruction::F64Floor => visitor.visit_f64_floor(),
            Instruction::F64Trunc => visitor.visit_f64_trunc(),
            Instruction::F64Nearest => visitor.visit_f64_nearest(),
            Instruction::F64Sqrt => visitor.visit_f64_sqrt(),
            Instruction::F64Add => visitor.visit_f64_add(),
            Instruction::F64Sub => visitor.visit_f64_sub(),
            Instruction::F64Mul => visitor.visit_f64_mul(),
            Instruction::F64Div => visitor.visit_f64_div(),
            Instruction::F64Min => visitor.visit_f64_min(),
            Instruction::F64Max => visitor.visit_f64_max(),
            Instruction::F64Copysign => visitor.visit_f64_copysign(),
            Instruction::I32WrapI64 => visitor.visit_i32_wrap_i64(),
            Instruction::I32TruncSF32 => visitor.visit_i32_trunc_f32(),
            Instruction::I32TruncUF32 => visitor.visit_u32_trunc_f32(),
            Instruction::I32TruncSF64 => visitor.visit_i32_trunc_f64(),
            Instruction::I32TruncUF64 => visitor.visit_u32_trunc_f64(),
            Instruction::I64ExtendSI32 => visitor.visit_i64_extend_i32(),
            Instruction::I64ExtendUI32 => visitor.visit_i64_extend_u32(),
            Instruction::I64TruncSF32 => visitor.visit_i64_trunc_f32(),
            Instruction::I64TruncUF32 => visitor.visit_u64_trunc_f32(),
            Instruction::I64TruncSF64 => visitor.visit_i64_trunc_f64(),
            Instruction::I64TruncUF64 => visitor.visit_u64_trunc_f64(),
            Instruction::F32ConvertSI32 => visitor.visit_f32_convert_i32(),
            Instruction::F32ConvertUI32 => visitor.visit_f32_convert_u32(),
            Instruction::F32ConvertSI64 => visitor.visit_f32_convert_i64(),
            Instruction::F32ConvertUI64 => visitor.visit_f32_convert_u64(),
            Instruction::F32DemoteF64 => visitor.visit_f32_demote_f64(),
            Instruction::F64ConvertSI32 => visitor.visit_f64_convert_i32(),
            Instruction::F64ConvertUI32 => visitor.visit_f64_convert_u32(),
            Instruction::F64ConvertSI64 => visitor.visit_f64_convert_i64(),
            Instruction::F64ConvertUI64 => visitor.visit_f64_convert_u64(),
            Instruction::F64PromoteF32 => visitor.visit_f64_promote_f32(),
            Instruction::I32ReinterpretF32 => visitor.visit_i32_reinterpret_f32(),
            Instruction::I64ReinterpretF64 => visitor.visit_i64_reinterpret_f64(),
            Instruction::F32ReinterpretI32 => visitor.visit_f32_reinterpret_i32(),
            Instruction::F64ReinterpretI64 => visitor.visit_f64_reinterpret_i64(),
            Instruction::I32TruncSatF32S => visitor.visit_i32_trunc_sat_f32(),
            Instruction::I32TruncSatF32U => visitor.visit_u32_trunc_sat_f32(),
            Instruction::I32TruncSatF64S => visitor.visit_i32_trunc_sat_f64(),
            Instruction::I32TruncSatF64U => visitor.visit_u32_trunc_sat_f64(),
            Instruction::I64TruncSatF32S => visitor.visit_i64_trunc_sat_f32(),
            Instruction::I64TruncSatF32U => visitor.visit_u64_trunc_sat_f32(),
            Instruction::I64TruncSatF64S => visitor.visit_i64_trunc_sat_f64(),
            Instruction::I64TruncSatF64U => visitor.visit_u64_trunc_sat_f64(),
            Instruction::I32Extend8S => visitor.visit_i32_sign_extend8(),
            Instruction::I32Extend16S => visitor.visit_i32_sign_extend16(),
            Instruction::I64Extend8S => visitor.visit_i64_sign_extend8(),
            Instruction::I64Extend16S => visitor.visit_i64_sign_extend16(),
            Instruction::I64Extend32S => visitor.visit_i64_sign_extend32(),
            Instruction::FuncBodyStart { .. } | Instruction::FuncBodyEnd => panic!(
                "expected start of a new instruction at index {} but found: {:?}",
                index, inst
            ),
        }
    }
}
