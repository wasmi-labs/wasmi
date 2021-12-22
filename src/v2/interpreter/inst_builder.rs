//! Abstractions to build up instructions forming Wasm function bodies.

#![allow(dead_code, missing_docs)] // TODO: remove

use super::{
    isa::{DropKeep, FuncIdx, GlobalIdx, LocalIdx, Offset, SignatureIdx, Target},
    Instruction,
};
use crate::{RuntimeValue, ValueType};
use core::fmt;
use core::fmt::Display;
use alloc::vec::Vec;

#[derive(Debug)]
pub struct Instructions {
    insts: Vec<Instruction>,
}

impl Instructions {
    pub fn build() -> InstructionsBuilder {
        InstructionsBuilder { insts: Vec::new() }
    }
}

#[derive(Debug)]
pub struct InstructionsBuilder {
    insts: Vec<Instruction>,
}

#[derive(Debug)]
pub struct InstructionIdx(usize);

impl InstructionsBuilder {
    fn next_idx(&self) -> InstructionIdx {
        InstructionIdx(self.insts.len())
    }

    fn push_inst(&mut self, inst: Instruction) -> InstructionIdx {
        let idx = self.next_idx();
        self.insts.push(inst);
        idx
    }

    #[must_use]
    pub fn finish(self) -> Instructions {
        Instructions { insts: self.insts }
    }
}

impl InstructionsBuilder {
    pub fn get_local(&mut self, local_idx: LocalIdx) -> InstructionIdx {
        self.push_inst(Instruction::GetLocal(local_idx))
    }

    pub fn set_local(&mut self, local_idx: LocalIdx) -> InstructionIdx {
        self.push_inst(Instruction::SetLocal(local_idx))
    }

    pub fn tee_local(&mut self, local_idx: LocalIdx) -> InstructionIdx {
        self.push_inst(Instruction::TeeLocal(local_idx))
    }

    pub fn branch(&mut self, target: Target) -> InstructionIdx {
        self.push_inst(Instruction::Br(target))
    }

    pub fn branch_eqz(&mut self, target: Target) -> InstructionIdx {
        self.push_inst(Instruction::BrIfEqz(target))
    }

    pub fn branch_nez(&mut self, target: Target) -> InstructionIdx {
        self.push_inst(Instruction::BrIfNez(target))
    }

    pub fn branch_table<I>(&mut self, default_target: Target, targets: I) -> InstructionIdx
    where
        I: IntoIterator<Item = Target>,
        I::IntoIter: ExactSizeIterator,
    {
        let targets = targets.into_iter();
        let count = targets.len() + 1;
        let head = self.push_inst(Instruction::BrTable { count });
        for target in targets {
            self.push_inst(Instruction::BrTableTarget(target));
        }
        self.push_inst(Instruction::BrTableTarget(default_target));
        head
    }

    pub fn unreachable(&mut self) -> InstructionIdx {
        self.push_inst(Instruction::Unreachable)
    }

    pub fn ret(&mut self, drop_keep: DropKeep) -> InstructionIdx {
        self.push_inst(Instruction::Return(drop_keep))
    }

    pub fn call(&mut self, func_idx: FuncIdx) -> InstructionIdx {
        self.push_inst(Instruction::Call(func_idx))
    }

    pub fn call_indirect(&mut self, signature_idx: SignatureIdx) -> InstructionIdx {
        self.push_inst(Instruction::CallIndirect(signature_idx))
    }

    pub fn drop(&mut self) -> InstructionIdx {
        self.push_inst(Instruction::Drop)
    }

    pub fn select(&mut self) -> InstructionIdx {
        self.push_inst(Instruction::Select)
    }

    pub fn get_global(&mut self, global_idx: GlobalIdx) -> InstructionIdx {
        self.push_inst(Instruction::GetGlobal(global_idx))
    }

    pub fn set_global(&mut self, global_idx: GlobalIdx) -> InstructionIdx {
        self.push_inst(Instruction::SetGlobal(global_idx))
    }

    pub fn load(&mut self, value_type: ValueType, offset: Offset) -> InstructionIdx {
        let inst = match value_type {
            ValueType::I32 => Instruction::I32Load(offset),
            ValueType::I64 => Instruction::I64Load(offset),
            ValueType::F32 => Instruction::F32Load(offset),
            ValueType::F64 => Instruction::F64Load(offset),
        };
        self.push_inst(inst)
    }

    pub fn load_extend<T, S>(&mut self, offset: Offset) -> InstructionIdx
    where
        T: ExtendFrom<S>,
    {
        let inst = match (T::result_type(), T::source_type()) {
            (WasmIntType::I32, IntType::I8) => Instruction::I32Load8S(offset),
            (WasmIntType::I32, IntType::U8) => Instruction::I32Load8U(offset),
            (WasmIntType::I32, IntType::I16) => Instruction::I32Load16S(offset),
            (WasmIntType::I32, IntType::U16) => Instruction::I32Load16U(offset),
            (WasmIntType::I64, IntType::I8) => Instruction::I64Load8S(offset),
            (WasmIntType::I64, IntType::U8) => Instruction::I64Load8U(offset),
            (WasmIntType::I64, IntType::I16) => Instruction::I64Load16S(offset),
            (WasmIntType::I64, IntType::U16) => Instruction::I64Load16U(offset),
            (WasmIntType::I64, IntType::I32) => Instruction::I64Load32S(offset),
            (WasmIntType::I64, IntType::U32) => Instruction::I64Load32U(offset),
            (dst, src) => unreachable!(
                "encountered invalid integer extension from {} to {}",
                src, dst
            ),
        };
        self.push_inst(inst)
    }

    pub fn store(&mut self, value_type: ValueType, offset: Offset) -> InstructionIdx {
        let inst = match value_type {
            ValueType::I32 => Instruction::I32Store(offset),
            ValueType::I64 => Instruction::I64Store(offset),
            ValueType::F32 => Instruction::F32Store(offset),
            ValueType::F64 => Instruction::F64Store(offset),
        };
        self.push_inst(inst)
    }

    pub fn store_truncate<T, S>(&mut self, offset: Offset) -> InstructionIdx
    where
        T: TruncateInto<S>,
    {
        let inst = match (T::source_type(), T::result_type()) {
            (WasmIntType::I32, UnsignedIntType::U8) => Instruction::I32Store8(offset),
            (WasmIntType::I32, UnsignedIntType::U16) => Instruction::I32Store16(offset),
            (WasmIntType::I64, UnsignedIntType::U8) => Instruction::I64Store8(offset),
            (WasmIntType::I64, UnsignedIntType::U16) => Instruction::I64Store16(offset),
            (WasmIntType::I64, UnsignedIntType::U32) => Instruction::I64Store32(offset),
            (src, dst) => unreachable!(
                "encountered invalid integer truncation from {} to {}",
                src, dst
            ),
        };
        self.push_inst(inst)
    }

    pub fn memory_grow(&mut self) -> InstructionIdx {
        self.push_inst(Instruction::CurrentMemory)
    }

    pub fn memory_size(&mut self) -> InstructionIdx {
        self.push_inst(Instruction::GrowMemory)
    }

    pub fn constant(&mut self, value: RuntimeValue) -> InstructionIdx {
        let inst = match value {
            RuntimeValue::I32(value) => Instruction::I32Const(value),
            RuntimeValue::I64(value) => Instruction::I64Const(value),
            RuntimeValue::F32(value) => Instruction::F32Const(value),
            RuntimeValue::F64(value) => Instruction::F64Const(value),
        };
        self.push_inst(inst)
    }

    pub fn eq(&mut self, value_type: ValueType) -> InstructionIdx {
        let inst = match value_type {
            ValueType::I32 => Instruction::I32Eq,
            ValueType::I64 => Instruction::I64Eq,
            ValueType::F32 => Instruction::F32Eq,
            ValueType::F64 => Instruction::F64Eq,
        };
        self.push_inst(inst)
    }

    pub fn ne(&mut self, value_type: ValueType) -> InstructionIdx {
        let inst = match value_type {
            ValueType::I32 => Instruction::I32Ne,
            ValueType::I64 => Instruction::I64Ne,
            ValueType::F32 => Instruction::F32Ne,
            ValueType::F64 => Instruction::F64Ne,
        };
        self.push_inst(inst)
    }

    pub fn int_eqz(&mut self, int_type: WasmIntType) -> InstructionIdx {
        let inst = match int_type {
            WasmIntType::I32 => Instruction::I32Eqz,
            WasmIntType::I64 => Instruction::I64Eqz,
        };
        self.push_inst(inst)
    }

    pub fn int_lt(&mut self, int_type: WasmIntType, signedness: Signedness) -> InstructionIdx {
        let inst = match (int_type, signedness) {
            (WasmIntType::I32, Signedness::Signed) => Instruction::I32LtS,
            (WasmIntType::I32, Signedness::Unsigned) => Instruction::I32LtU,
            (WasmIntType::I64, Signedness::Signed) => Instruction::I64LtS,
            (WasmIntType::I64, Signedness::Unsigned) => Instruction::I64LtU,
        };
        self.push_inst(inst)
    }

    pub fn int_gt(&mut self, int_type: WasmIntType, signedness: Signedness) -> InstructionIdx {
        let inst = match (int_type, signedness) {
            (WasmIntType::I32, Signedness::Signed) => Instruction::I32GtS,
            (WasmIntType::I32, Signedness::Unsigned) => Instruction::I32GtU,
            (WasmIntType::I64, Signedness::Signed) => Instruction::I64GtS,
            (WasmIntType::I64, Signedness::Unsigned) => Instruction::I64GtU,
        };
        self.push_inst(inst)
    }

    pub fn int_le(&mut self, int_type: WasmIntType, signedness: Signedness) -> InstructionIdx {
        let inst = match (int_type, signedness) {
            (WasmIntType::I32, Signedness::Signed) => Instruction::I32LeS,
            (WasmIntType::I32, Signedness::Unsigned) => Instruction::I32LeU,
            (WasmIntType::I64, Signedness::Signed) => Instruction::I64LeS,
            (WasmIntType::I64, Signedness::Unsigned) => Instruction::I64LeU,
        };
        self.push_inst(inst)
    }

    pub fn int_ge(&mut self, int_type: WasmIntType, signedness: Signedness) -> InstructionIdx {
        let inst = match (int_type, signedness) {
            (WasmIntType::I32, Signedness::Signed) => Instruction::I32GeS,
            (WasmIntType::I32, Signedness::Unsigned) => Instruction::I32GeU,
            (WasmIntType::I64, Signedness::Signed) => Instruction::I64GeS,
            (WasmIntType::I64, Signedness::Unsigned) => Instruction::I64GeU,
        };
        self.push_inst(inst)
    }

    pub fn float_lt(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Lt,
            WasmFloatType::F64 => Instruction::F64Lt,
        };
        self.push_inst(inst)
    }

    pub fn float_gt(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Gt,
            WasmFloatType::F64 => Instruction::F64Gt,
        };
        self.push_inst(inst)
    }

    pub fn float_le(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Le,
            WasmFloatType::F64 => Instruction::F64Le,
        };
        self.push_inst(inst)
    }

    pub fn float_ge(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Ge,
            WasmFloatType::F64 => Instruction::F64Ge,
        };
        self.push_inst(inst)
    }

    pub fn int_clz(&mut self, int_type: WasmIntType) -> InstructionIdx {
        let inst = match int_type {
            WasmIntType::I32 => Instruction::I32Clz,
            WasmIntType::I64 => Instruction::I64Clz,
        };
        self.push_inst(inst)
    }

    pub fn int_ctz(&mut self, int_type: WasmIntType) -> InstructionIdx {
        let inst = match int_type {
            WasmIntType::I32 => Instruction::I32Ctz,
            WasmIntType::I64 => Instruction::I64Ctz,
        };
        self.push_inst(inst)
    }

    pub fn int_popcnt(&mut self, int_type: WasmIntType) -> InstructionIdx {
        let inst = match int_type {
            WasmIntType::I32 => Instruction::I32Popcnt,
            WasmIntType::I64 => Instruction::I64Popcnt,
        };
        self.push_inst(inst)
    }

    pub fn int_add(&mut self, int_type: WasmIntType) -> InstructionIdx {
        let inst = match int_type {
            WasmIntType::I32 => Instruction::I32Add,
            WasmIntType::I64 => Instruction::I64Add,
        };
        self.push_inst(inst)
    }

    pub fn int_sub(&mut self, int_type: WasmIntType) -> InstructionIdx {
        let inst = match int_type {
            WasmIntType::I32 => Instruction::I32Sub,
            WasmIntType::I64 => Instruction::I64Sub,
        };
        self.push_inst(inst)
    }

    pub fn int_mul(&mut self, int_type: WasmIntType) -> InstructionIdx {
        let inst = match int_type {
            WasmIntType::I32 => Instruction::I32Mul,
            WasmIntType::I64 => Instruction::I64Mul,
        };
        self.push_inst(inst)
    }

    pub fn int_div(&mut self, int_type: WasmIntType, signedness: Signedness) -> InstructionIdx {
        let inst = match (int_type, signedness) {
            (WasmIntType::I32, Signedness::Signed) => Instruction::I32DivS,
            (WasmIntType::I32, Signedness::Unsigned) => Instruction::I32DivU,
            (WasmIntType::I64, Signedness::Signed) => Instruction::I64DivS,
            (WasmIntType::I64, Signedness::Unsigned) => Instruction::I64DivU,
        };
        self.push_inst(inst)
    }

    pub fn int_rem(&mut self, int_type: WasmIntType, signedness: Signedness) -> InstructionIdx {
        let inst = match (int_type, signedness) {
            (WasmIntType::I32, Signedness::Signed) => Instruction::I32RemS,
            (WasmIntType::I32, Signedness::Unsigned) => Instruction::I32RemU,
            (WasmIntType::I64, Signedness::Signed) => Instruction::I64RemS,
            (WasmIntType::I64, Signedness::Unsigned) => Instruction::I64RemU,
        };
        self.push_inst(inst)
    }

    pub fn int_and(&mut self, int_type: WasmIntType) -> InstructionIdx {
        let inst = match int_type {
            WasmIntType::I32 => Instruction::I32And,
            WasmIntType::I64 => Instruction::I64And,
        };
        self.push_inst(inst)
    }

    pub fn int_or(&mut self, int_type: WasmIntType) -> InstructionIdx {
        let inst = match int_type {
            WasmIntType::I32 => Instruction::I32Or,
            WasmIntType::I64 => Instruction::I64Or,
        };
        self.push_inst(inst)
    }

    pub fn int_xor(&mut self, int_type: WasmIntType) -> InstructionIdx {
        let inst = match int_type {
            WasmIntType::I32 => Instruction::I32Xor,
            WasmIntType::I64 => Instruction::I64Xor,
        };
        self.push_inst(inst)
    }

    pub fn int_shl(&mut self, int_type: WasmIntType) -> InstructionIdx {
        let inst = match int_type {
            WasmIntType::I32 => Instruction::I32Shl,
            WasmIntType::I64 => Instruction::I64Shl,
        };
        self.push_inst(inst)
    }

    pub fn int_shr(&mut self, int_type: WasmIntType, signedness: Signedness) -> InstructionIdx {
        let inst = match (int_type, signedness) {
            (WasmIntType::I32, Signedness::Signed) => Instruction::I32ShrS,
            (WasmIntType::I32, Signedness::Unsigned) => Instruction::I32ShrU,
            (WasmIntType::I64, Signedness::Signed) => Instruction::I64ShrS,
            (WasmIntType::I64, Signedness::Unsigned) => Instruction::I64ShrU,
        };
        self.push_inst(inst)
    }

    pub fn int_rotl(&mut self, int_type: WasmIntType) -> InstructionIdx {
        let inst = match int_type {
            WasmIntType::I32 => Instruction::I32Rotl,
            WasmIntType::I64 => Instruction::I64Rotl,
        };
        self.push_inst(inst)
    }

    pub fn int_rotr(&mut self, int_type: WasmIntType) -> InstructionIdx {
        let inst = match int_type {
            WasmIntType::I32 => Instruction::I32Rotr,
            WasmIntType::I64 => Instruction::I64Rotr,
        };
        self.push_inst(inst)
    }

    pub fn float_abs(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Abs,
            WasmFloatType::F64 => Instruction::F64Abs,
        };
        self.push_inst(inst)
    }

    pub fn float_neg(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Neg,
            WasmFloatType::F64 => Instruction::F64Neg,
        };
        self.push_inst(inst)
    }

    pub fn float_ceil(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Ceil,
            WasmFloatType::F64 => Instruction::F64Ceil,
        };
        self.push_inst(inst)
    }

    pub fn float_floor(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Floor,
            WasmFloatType::F64 => Instruction::F64Floor,
        };
        self.push_inst(inst)
    }

    pub fn float_trunc(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Trunc,
            WasmFloatType::F64 => Instruction::F64Trunc,
        };
        self.push_inst(inst)
    }

    pub fn float_nearest(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Nearest,
            WasmFloatType::F64 => Instruction::F64Nearest,
        };
        self.push_inst(inst)
    }

    pub fn float_sqrt(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Sqrt,
            WasmFloatType::F64 => Instruction::F64Sqrt,
        };
        self.push_inst(inst)
    }

    pub fn float_add(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Add,
            WasmFloatType::F64 => Instruction::F64Add,
        };
        self.push_inst(inst)
    }

    pub fn float_sub(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Sub,
            WasmFloatType::F64 => Instruction::F64Sub,
        };
        self.push_inst(inst)
    }

    pub fn float_mul(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Mul,
            WasmFloatType::F64 => Instruction::F64Mul,
        };
        self.push_inst(inst)
    }

    pub fn float_div(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Div,
            WasmFloatType::F64 => Instruction::F64Div,
        };
        self.push_inst(inst)
    }

    pub fn float_min(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Min,
            WasmFloatType::F64 => Instruction::F64Min,
        };
        self.push_inst(inst)
    }

    pub fn float_max(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Max,
            WasmFloatType::F64 => Instruction::F64Max,
        };
        self.push_inst(inst)
    }

    pub fn float_copysign(&mut self, float_type: WasmFloatType) -> InstructionIdx {
        let inst = match float_type {
            WasmFloatType::F32 => Instruction::F32Copysign,
            WasmFloatType::F64 => Instruction::F64Copysign,
        };
        self.push_inst(inst)
    }

    pub fn wrap(&mut self) -> InstructionIdx {
        self.push_inst(Instruction::I32WrapI64)
    }

    pub fn extend(&mut self, signedness: Signedness) -> InstructionIdx {
        let inst = match signedness {
            Signedness::Signed => Instruction::I64ExtendSI32,
            Signedness::Unsigned => Instruction::I64ExtendUI32,
        };
        self.push_inst(inst)
    }

    pub fn float_truncate_to_int(
        &mut self,
        float_type: WasmFloatType,
        int_type: WasmIntType,
        signedness: Signedness,
    ) -> InstructionIdx {
        use WasmFloatType as Float;
        use WasmIntType as Int;
        let inst = match (float_type, int_type, signedness) {
            (Float::F32, Int::I32, Signedness::Signed) => Instruction::I32TruncSF32,
            (Float::F32, Int::I32, Signedness::Unsigned) => Instruction::I32TruncUF32,
            (Float::F32, Int::I64, Signedness::Signed) => Instruction::I64TruncSF32,
            (Float::F32, Int::I64, Signedness::Unsigned) => Instruction::I64TruncUF32,
            (Float::F64, Int::I32, Signedness::Signed) => Instruction::I32TruncSF64,
            (Float::F64, Int::I32, Signedness::Unsigned) => Instruction::I32TruncUF64,
            (Float::F64, Int::I64, Signedness::Signed) => Instruction::I64TruncSF64,
            (Float::F64, Int::I64, Signedness::Unsigned) => Instruction::I64TruncUF64,
        };
        self.push_inst(inst)
    }

    pub fn demote(&mut self) -> InstructionIdx {
        self.push_inst(Instruction::F32DemoteF64)
    }

    pub fn promote(&mut self) -> InstructionIdx {
        self.push_inst(Instruction::F64PromoteF32)
    }

    pub fn int_convert_to_float(
        &mut self,
        int_type: WasmIntType,
        signedness: Signedness,
        float_type: WasmFloatType,
    ) -> InstructionIdx {
        use WasmFloatType as Float;
        use WasmIntType as Int;
        let inst = match (int_type, signedness, float_type) {
            (Int::I32, Signedness::Signed, Float::F32) => Instruction::F32ConvertSI32,
            (Int::I32, Signedness::Signed, Float::F64) => Instruction::F64ConvertSI32,
            (Int::I32, Signedness::Unsigned, Float::F32) => Instruction::F32ConvertUI32,
            (Int::I32, Signedness::Unsigned, Float::F64) => Instruction::F64ConvertUI32,
            (Int::I64, Signedness::Signed, Float::F32) => Instruction::F32ConvertSI64,
            (Int::I64, Signedness::Signed, Float::F64) => Instruction::F64ConvertSI64,
            (Int::I64, Signedness::Unsigned, Float::F32) => Instruction::F32ConvertUI64,
            (Int::I64, Signedness::Unsigned, Float::F64) => Instruction::F64ConvertUI64,
        };
        self.push_inst(inst)
    }

    pub fn reinterpret<S, T>(&mut self) -> InstructionIdx
    where
        S: ReinterpretAs<T>,
    {
        let inst = match (S::source_type(), S::target_type()) {
            (ValueType::I32, ValueType::F32) => Instruction::F32ReinterpretI32,
            (ValueType::I64, ValueType::F64) => Instruction::F64ReinterpretI64,
            (ValueType::F32, ValueType::I32) => Instruction::I32ReinterpretF32,
            (ValueType::F64, ValueType::I64) => Instruction::I64ReinterpretF64,
            (src, dst) => unreachable!(
                "encountered invalid reinterpretation from {} to {}",
                src, dst
            ),
        };
        self.push_inst(inst)
    }
}

pub trait IntPrim {
    fn int_type() -> IntType;
}

macro_rules! impl_int_prim_for {
    ( $( ($prim:ty => $name:ident) ),* $(,)? ) => {
        $(
            impl IntPrim for $prim {
                fn int_type() -> IntType {
                    IntType::$name
                }
            }
        )*
    };
}
impl_int_prim_for!(
    (i8 => I8),
    (i16 => I16),
    (i32 => I32),
    (i64 => I64),
    (u8 => U8),
    (u16 => U16),
    (u32 => U32),
    (u64 => U64),
);

#[derive(Debug)]
pub enum IntType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
}

impl Display for IntType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::I8 => write!(f, "i8"),
            Self::I16 => write!(f, "i16"),
            Self::I32 => write!(f, "i32"),
            Self::I64 => write!(f, "i64"),
            Self::U8 => write!(f, "u8"),
            Self::U16 => write!(f, "u16"),
            Self::U32 => write!(f, "u32"),
            Self::U64 => write!(f, "u64"),
        }
    }
}

impl IntType {
    pub fn into_unsigned(self) -> UnsignedIntType {
        match self {
            Self::I8 | Self::U8 => UnsignedIntType::U8,
            Self::I16 | Self::U16 => UnsignedIntType::U16,
            Self::I32 | Self::U32 => UnsignedIntType::U32,
            Self::I64 | Self::U64 => UnsignedIntType::U64,
        }
    }
}

#[derive(Debug)]
pub enum UnsignedIntType {
    U8,
    U16,
    U32,
    U64,
}

impl Display for UnsignedIntType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::U8 => write!(f, "u8"),
            Self::U16 => write!(f, "u16"),
            Self::U32 => write!(f, "u32"),
            Self::U64 => write!(f, "u64"),
        }
    }
}

pub trait WasmIntPrim {
    fn wasm_int_type() -> WasmIntType;
}

impl WasmIntPrim for i32 {
    fn wasm_int_type() -> WasmIntType {
        WasmIntType::I32
    }
}

impl WasmIntPrim for i64 {
    fn wasm_int_type() -> WasmIntType {
        WasmIntType::I64
    }
}

#[derive(Debug)]
pub enum WasmIntType {
    I32,
    I64,
}

impl Display for WasmIntType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::I32 => write!(f, "i32"),
            Self::I64 => write!(f, "i64"),
        }
    }
}

pub trait ExtendFrom<T> {
    fn result_type() -> WasmIntType;
    fn source_type() -> IntType;
}

macro_rules! impl_extend_from_for {
    ( $( $result_type:ty > $source_type:ty ),* $(,)? ) => {
        $(
            impl ExtendFrom<$source_type> for $result_type {
                fn result_type() -> WasmIntType {
                    <$result_type as WasmIntPrim>::wasm_int_type()
                }

                fn source_type() -> IntType {
                    <$source_type as IntPrim>::int_type()
                }
            }
        )*
    };
}
impl_extend_from_for!(
    i32 > i8,
    i32 > u8,
    i32 > i16,
    i32 > u16,
    i64 > i8,
    i64 > u8,
    i64 > i16,
    i64 > u16,
    i64 > i32,
    i64 > u32,
);

pub trait TruncateInto<T> {
    fn result_type() -> UnsignedIntType;
    fn source_type() -> WasmIntType;
}

macro_rules! impl_truncate_into_for {
    ( $( $source_type:ty > $result_type:ty ),* $(,)? ) => {
        $(
            impl TruncateInto<$result_type> for $source_type {
                fn result_type() -> UnsignedIntType {
                    <$result_type as IntPrim>::int_type().into_unsigned()
                }
                fn source_type() -> WasmIntType {
                    <$source_type as WasmIntPrim>::wasm_int_type()
                }
            }
        )*
    };
}
impl_truncate_into_for!(i32 > i8, i32 > i16, i64 > i8, i64 > i16, i64 > i32,);

pub enum WasmFloatType {
    F32,
    F64,
}

pub enum Signedness {
    Signed,
    Unsigned,
}

pub trait ReinterpretAs<T> {
    fn source_type() -> ValueType;
    fn target_type() -> ValueType;
}

impl ReinterpretAs<i32> for f32 {
    fn source_type() -> ValueType {
        ValueType::F32
    }
    fn target_type() -> ValueType {
        ValueType::I32
    }
}

impl ReinterpretAs<f32> for i32 {
    fn source_type() -> ValueType {
        ValueType::I32
    }
    fn target_type() -> ValueType {
        ValueType::F32
    }
}

impl ReinterpretAs<i64> for f64 {
    fn source_type() -> ValueType {
        ValueType::F64
    }
    fn target_type() -> ValueType {
        ValueType::I64
    }
}

impl ReinterpretAs<f64> for i64 {
    fn source_type() -> ValueType {
        ValueType::I64
    }
    fn target_type() -> ValueType {
        ValueType::F64
    }
}
