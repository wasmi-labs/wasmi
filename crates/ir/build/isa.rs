use crate::build::{
    Config,
    Op,
    ident::Ident,
    op::{
        BinaryOp,
        BinaryOpCaps,
        CmpBranchOp,
        Field,
        GenericOp,
        LaneWidth,
        LoadKind,
        LoadOp,
        MemoryOperand,
        OffsetOperand,
        OperandKind,
        SelectOp,
        SelectWidth,
        SimdTy,
        StoreKind,
        StoreOp,
        TableGetOp,
        TableSetOp,
        TernaryOp,
        TernaryOpKind,
        UnaryOp,
        V128ExtractLaneOp,
        V128ReplaceLaneOp,
    },
    ty::{FieldTy, Ty},
};

#[derive(Default)]
pub struct Isa {
    pub ops: Vec<Op>,
}

impl Isa {
    fn push_op(&mut self, op: impl Into<Op>) {
        self.ops.push(op.into());
    }

    fn push_ops(&mut self, ops: impl IntoIterator<Item = Op>) {
        for op in ops {
            self.ops.push(op);
        }
    }
}

pub fn wasmi_isa(config: &Config) -> Isa {
    let mut isa = Isa::default();
    isa.ops.reserve_exact(500);
    add_unary_ops(&mut isa);
    add_binary_ops(&mut isa);
    add_cmp_branch_ops(&mut isa);
    add_select_ops(&mut isa);
    add_load_ops(&mut isa);
    add_store_ops(&mut isa);
    add_control_ops(&mut isa);
    add_copy_ops(&mut isa);
    add_call_ops(&mut isa);
    add_global_ops(&mut isa);
    add_memory_ops(&mut isa);
    add_table_ops(&mut isa);
    add_wide_arithmetic_ops(&mut isa);
    add_simd_ops(&mut isa, config);
    isa
}

fn add_unary_ops(isa: &mut Isa) {
    let ops = [
        // i32
        (Ident::Clz, Ty::I32, Ty::I32),
        (Ident::Ctz, Ty::I32, Ty::I32),
        (Ident::Popcnt, Ty::I32, Ty::I32),
        (Ident::Sext, Ty::I32, Ty::Bits8),
        (Ident::Sext, Ty::I32, Ty::Bits16),
        (Ident::Wrap, Ty::I32, Ty::I64),
        // i64
        (Ident::Clz, Ty::I64, Ty::I64),
        (Ident::Ctz, Ty::I64, Ty::I64),
        (Ident::Popcnt, Ty::I64, Ty::I64),
        (Ident::Sext, Ty::I64, Ty::Bits8),
        (Ident::Sext, Ty::I64, Ty::Bits16),
        (Ident::Sext, Ty::I64, Ty::Bits32),
        // f32
        (Ident::Abs, Ty::F32, Ty::F32),
        (Ident::Neg, Ty::F32, Ty::F32),
        (Ident::Ceil, Ty::F32, Ty::F32),
        (Ident::Floor, Ty::F32, Ty::F32),
        (Ident::Trunc, Ty::F32, Ty::F32),
        (Ident::Nearest, Ty::F32, Ty::F32),
        (Ident::Sqrt, Ty::F32, Ty::F32),
        (Ident::Convert, Ty::F32, Ty::I32),
        (Ident::Convert, Ty::F32, Ty::U32),
        (Ident::Convert, Ty::F32, Ty::I64),
        (Ident::Convert, Ty::F32, Ty::U64),
        (Ident::Demote, Ty::F32, Ty::F64),
        // f64
        (Ident::Abs, Ty::F64, Ty::F64),
        (Ident::Neg, Ty::F64, Ty::F64),
        (Ident::Ceil, Ty::F64, Ty::F64),
        (Ident::Floor, Ty::F64, Ty::F64),
        (Ident::Trunc, Ty::F64, Ty::F64),
        (Ident::Nearest, Ty::F64, Ty::F64),
        (Ident::Sqrt, Ty::F64, Ty::F64),
        (Ident::Convert, Ty::F64, Ty::I32),
        (Ident::Convert, Ty::F64, Ty::U32),
        (Ident::Convert, Ty::F64, Ty::I64),
        (Ident::Convert, Ty::F64, Ty::U64),
        (Ident::Promote, Ty::F64, Ty::F32),
        // f2i conversions
        (Ident::Trunc, Ty::I32, Ty::F32),
        (Ident::Trunc, Ty::U32, Ty::F32),
        (Ident::Trunc, Ty::I32, Ty::F64),
        (Ident::Trunc, Ty::U32, Ty::F64),
        (Ident::Trunc, Ty::I64, Ty::F32),
        (Ident::Trunc, Ty::U64, Ty::F32),
        (Ident::Trunc, Ty::I64, Ty::F64),
        (Ident::Trunc, Ty::U64, Ty::F64),
        (Ident::TruncSat, Ty::I32, Ty::F32),
        (Ident::TruncSat, Ty::U32, Ty::F32),
        (Ident::TruncSat, Ty::I32, Ty::F64),
        (Ident::TruncSat, Ty::U32, Ty::F64),
        (Ident::TruncSat, Ty::I64, Ty::F32),
        (Ident::TruncSat, Ty::U64, Ty::F32),
        (Ident::TruncSat, Ty::I64, Ty::F64),
        (Ident::TruncSat, Ty::U64, Ty::F64),
    ];
    for (ident, result_ty, value_ty) in ops {
        isa.push_op(UnaryOp::new(ident, result_ty, value_ty, OperandKind::Slot))
    }
}

fn add_binary_ops(isa: &mut Isa) {
    #[rustfmt::skip]
    let ops = [
        // comparisons: i32
        (Ident::Eq, Ty::I32, Ty::I32, Ty::I32, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::And, Ty::I32, Ty::I32, Ty::I32, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::Or, Ty::I32, Ty::I32, Ty::I32, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::NotEq, Ty::I32, Ty::I32, Ty::I32, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::NotAnd, Ty::I32, Ty::I32, Ty::I32, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::NotOr, Ty::I32, Ty::I32, Ty::I32, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::Lt, Ty::I32, Ty::I32, Ty::I32, BinaryOpCaps::CMP),
        (Ident::Le, Ty::I32, Ty::I32, Ty::I32, BinaryOpCaps::CMP),
        (Ident::Lt, Ty::U32, Ty::U32, Ty::U32, BinaryOpCaps::CMP),
        (Ident::Le, Ty::U32, Ty::U32, Ty::U32, BinaryOpCaps::CMP),
        // comparisons: i64
        (Ident::Eq, Ty::I32, Ty::I64, Ty::I64, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::And, Ty::I32, Ty::I64, Ty::I64, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::Or, Ty::I32, Ty::I64, Ty::I64, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::NotEq, Ty::I32, Ty::I64, Ty::I64, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::NotAnd, Ty::I32, Ty::I64, Ty::I64, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::NotOr, Ty::I32, Ty::I64, Ty::I64, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::Lt, Ty::I32, Ty::I64, Ty::I64, BinaryOpCaps::CMP),
        (Ident::Le, Ty::I32, Ty::I64, Ty::I64, BinaryOpCaps::CMP),
        (Ident::Lt, Ty::U32, Ty::U64, Ty::U64, BinaryOpCaps::CMP),
        (Ident::Le, Ty::U32, Ty::U64, Ty::U64, BinaryOpCaps::CMP),
        // comparisons: f32
        (Ident::Eq, Ty::I32, Ty::F32, Ty::F32, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::Lt, Ty::I32, Ty::F32, Ty::F32, BinaryOpCaps::CMP),
        (Ident::Le, Ty::I32, Ty::F32, Ty::F32, BinaryOpCaps::CMP),
        (Ident::NotEq, Ty::I32, Ty::F32, Ty::F32, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::NotLt, Ty::I32, Ty::F32, Ty::F32, BinaryOpCaps::CMP),
        (Ident::NotLe, Ty::I32, Ty::F32, Ty::F32, BinaryOpCaps::CMP),
        // comparisons: f64
        (Ident::Eq, Ty::I32, Ty::F64, Ty::F64, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::Lt, Ty::I32, Ty::F64, Ty::F64, BinaryOpCaps::CMP),
        (Ident::Le, Ty::I32, Ty::F64, Ty::F64, BinaryOpCaps::CMP),
        (Ident::NotEq, Ty::I32, Ty::F64, Ty::F64, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::NotLt, Ty::I32, Ty::F64, Ty::F64, BinaryOpCaps::CMP),
        (Ident::NotLe, Ty::I32, Ty::F64, Ty::F64, BinaryOpCaps::CMP),
        // i32
        (Ident::Add, Ty::I32, Ty::I32, Ty::I32, BinaryOpCaps::COMMUTATIVE),
        (Ident::Sub, Ty::I32, Ty::I32, Ty::I32, BinaryOpCaps::NONE),
        (Ident::Mul, Ty::I32, Ty::I32, Ty::I32, BinaryOpCaps::COMMUTATIVE),
        (Ident::Div, Ty::I32, Ty::I32, Ty::NonZeroI32, BinaryOpCaps::NONE),
        (Ident::Div, Ty::U32, Ty::U32, Ty::NonZeroU32, BinaryOpCaps::NONE),
        (Ident::Rem, Ty::I32, Ty::I32, Ty::NonZeroI32, BinaryOpCaps::NONE),
        (Ident::Rem, Ty::U32, Ty::U32, Ty::NonZeroU32, BinaryOpCaps::NONE),
        (Ident::BitAnd, Ty::I32, Ty::I32, Ty::I32, BinaryOpCaps::COMMUTATIVE),
        (Ident::BitOr, Ty::I32, Ty::I32, Ty::I32, BinaryOpCaps::COMMUTATIVE),
        (Ident::BitXor, Ty::I32, Ty::I32, Ty::I32, BinaryOpCaps::COMMUTATIVE),
        (Ident::Shl, Ty::I32, Ty::I32, Ty::U8, BinaryOpCaps::NONE),
        (Ident::Shr, Ty::I32, Ty::I32, Ty::U8, BinaryOpCaps::NONE),
        (Ident::Shr, Ty::U32, Ty::U32, Ty::U8, BinaryOpCaps::NONE),
        (Ident::Rotl, Ty::I32, Ty::I32, Ty::U8, BinaryOpCaps::NONE),
        (Ident::Rotr, Ty::I32, Ty::I32, Ty::U8, BinaryOpCaps::NONE),
        // i64
        (Ident::Add, Ty::I64, Ty::I64, Ty::I64, BinaryOpCaps::COMMUTATIVE),
        (Ident::Sub, Ty::I64, Ty::I64, Ty::I64, BinaryOpCaps::NONE),
        (Ident::Mul, Ty::I64, Ty::I64, Ty::I64, BinaryOpCaps::COMMUTATIVE),
        (Ident::Div, Ty::I64, Ty::I64, Ty::NonZeroI64, BinaryOpCaps::NONE),
        (Ident::Div, Ty::U64, Ty::U64, Ty::NonZeroU64, BinaryOpCaps::NONE),
        (Ident::Rem, Ty::I64, Ty::I64, Ty::NonZeroI64, BinaryOpCaps::NONE),
        (Ident::Rem, Ty::U64, Ty::U64, Ty::NonZeroU64, BinaryOpCaps::NONE),
        (Ident::BitAnd, Ty::I64, Ty::I64, Ty::I64, BinaryOpCaps::COMMUTATIVE),
        (Ident::BitOr, Ty::I64, Ty::I64, Ty::I64, BinaryOpCaps::COMMUTATIVE),
        (Ident::BitXor, Ty::I64, Ty::I64, Ty::I64, BinaryOpCaps::COMMUTATIVE),
        (Ident::Shl, Ty::I64, Ty::I64, Ty::U8, BinaryOpCaps::NONE),
        (Ident::Shr, Ty::I64, Ty::I64, Ty::U8, BinaryOpCaps::NONE),
        (Ident::Shr, Ty::U64, Ty::U64, Ty::U8, BinaryOpCaps::NONE),
        (Ident::Rotl, Ty::I64, Ty::I64, Ty::U8, BinaryOpCaps::NONE),
        (Ident::Rotr, Ty::I64, Ty::I64, Ty::U8, BinaryOpCaps::NONE),
        // f32
        (Ident::Add, Ty::F32, Ty::F32, Ty::F32, BinaryOpCaps::NONE),
        (Ident::Sub, Ty::F32, Ty::F32, Ty::F32, BinaryOpCaps::NONE),
        (Ident::Mul, Ty::F32, Ty::F32, Ty::F32, BinaryOpCaps::NONE),
        (Ident::Div, Ty::F32, Ty::F32, Ty::F32, BinaryOpCaps::NONE),
        (Ident::Min, Ty::F32, Ty::F32, Ty::F32, BinaryOpCaps::NONE),
        (Ident::Max, Ty::F32, Ty::F32, Ty::F32, BinaryOpCaps::NONE),
        (Ident::Copysign, Ty::F32, Ty::F32, Ty::SignF32, BinaryOpCaps::NONE),
        // // f64
        (Ident::Add, Ty::F64, Ty::F64, Ty::F64, BinaryOpCaps::NONE),
        (Ident::Sub, Ty::F64, Ty::F64, Ty::F64, BinaryOpCaps::NONE),
        (Ident::Mul, Ty::F64, Ty::F64, Ty::F64, BinaryOpCaps::NONE),
        (Ident::Div, Ty::F64, Ty::F64, Ty::F64, BinaryOpCaps::NONE),
        (Ident::Min, Ty::F64, Ty::F64, Ty::F64, BinaryOpCaps::NONE),
        (Ident::Max, Ty::F64, Ty::F64, Ty::F64, BinaryOpCaps::NONE),
        (Ident::Copysign, Ty::F64, Ty::F64, Ty::SignF64, BinaryOpCaps::NONE),
    ];
    for (ident, result_ty, lhs_ty, rhs_ty, caps) in ops {
        isa.push_op(BinaryOp::new(
            ident,
            result_ty,
            lhs_ty,
            rhs_ty,
            OperandKind::Slot,
            OperandKind::Slot,
            caps,
        ));
        isa.push_op(BinaryOp::new(
            ident,
            result_ty,
            lhs_ty,
            rhs_ty,
            OperandKind::Slot,
            OperandKind::Immediate,
            caps,
        ));
        if !caps.is_commutative() {
            isa.push_op(BinaryOp::new(
                ident,
                result_ty,
                lhs_ty,
                rhs_ty,
                OperandKind::Immediate,
                OperandKind::Slot,
                caps,
            ));
        }
    }
}

fn add_cmp_branch_ops(isa: &mut Isa) {
    let ops = [
        // i32
        (Ident::Eq, Ty::I32, BinaryOpCaps::COMMUTATIVE),
        (Ident::NotEq, Ty::I32, BinaryOpCaps::COMMUTATIVE),
        (Ident::And, Ty::I32, BinaryOpCaps::COMMUTATIVE),
        (Ident::NotAnd, Ty::I32, BinaryOpCaps::COMMUTATIVE),
        (Ident::Or, Ty::I32, BinaryOpCaps::COMMUTATIVE),
        (Ident::NotOr, Ty::I32, BinaryOpCaps::COMMUTATIVE),
        (Ident::Lt, Ty::I32, BinaryOpCaps::NONE),
        (Ident::Le, Ty::I32, BinaryOpCaps::NONE),
        (Ident::Lt, Ty::U32, BinaryOpCaps::NONE),
        (Ident::Le, Ty::U32, BinaryOpCaps::NONE),
        // i64
        (Ident::Eq, Ty::I64, BinaryOpCaps::COMMUTATIVE),
        (Ident::NotEq, Ty::I64, BinaryOpCaps::COMMUTATIVE),
        (Ident::And, Ty::I64, BinaryOpCaps::COMMUTATIVE),
        (Ident::NotAnd, Ty::I64, BinaryOpCaps::COMMUTATIVE),
        (Ident::Or, Ty::I64, BinaryOpCaps::COMMUTATIVE),
        (Ident::NotOr, Ty::I64, BinaryOpCaps::COMMUTATIVE),
        (Ident::Lt, Ty::I64, BinaryOpCaps::NONE),
        (Ident::Le, Ty::I64, BinaryOpCaps::NONE),
        (Ident::Lt, Ty::U64, BinaryOpCaps::NONE),
        (Ident::Le, Ty::U64, BinaryOpCaps::NONE),
        // f32
        (Ident::Eq, Ty::F32, BinaryOpCaps::COMMUTATIVE),
        (Ident::NotEq, Ty::F32, BinaryOpCaps::COMMUTATIVE),
        (Ident::Lt, Ty::F32, BinaryOpCaps::NONE),
        (Ident::NotLt, Ty::F32, BinaryOpCaps::NONE),
        (Ident::Le, Ty::F32, BinaryOpCaps::NONE),
        (Ident::NotLe, Ty::F32, BinaryOpCaps::NONE),
        // f64
        (Ident::Eq, Ty::F64, BinaryOpCaps::COMMUTATIVE),
        (Ident::NotEq, Ty::F64, BinaryOpCaps::COMMUTATIVE),
        (Ident::Lt, Ty::F64, BinaryOpCaps::NONE),
        (Ident::NotLt, Ty::F64, BinaryOpCaps::NONE),
        (Ident::Le, Ty::F64, BinaryOpCaps::NONE),
        (Ident::NotLe, Ty::F64, BinaryOpCaps::NONE),
    ];
    for (ident, input_ty, caps) in ops {
        for rhs in [OperandKind::Slot, OperandKind::Immediate] {
            isa.push_op(CmpBranchOp::new(ident, input_ty, OperandKind::Slot, rhs));
        }
        if !caps.is_commutative() {
            isa.push_op(CmpBranchOp::new(
                ident,
                input_ty,
                OperandKind::Immediate,
                OperandKind::Slot,
            ));
        }
    }
}

fn add_select_ops(isa: &mut Isa) {
    isa.push_op(SelectOp::new(
        SelectWidth::None,
        OperandKind::Slot,
        OperandKind::Slot,
    ));
    for width in [SelectWidth::Bits32, SelectWidth::Bits64] {
        for true_val in [OperandKind::Slot, OperandKind::Immediate] {
            for false_val in [OperandKind::Slot, OperandKind::Immediate] {
                if matches!(true_val, OperandKind::Slot) && matches!(false_val, OperandKind::Slot) {
                    continue;
                }
                isa.push_op(SelectOp::new(width, true_val, false_val));
            }
        }
    }
}

fn add_load_ops(isa: &mut Isa) {
    #[rustfmt::skip]
    let ops = [
        // Generic
        (LoadKind::Value, Ty::U32),
        (LoadKind::Value, Ty::U64),
        // i32
        (LoadKind::Extend { result_ty: Ty::I32 }, Ty::Bits8),
        (LoadKind::Extend { result_ty: Ty::I32 }, Ty::Bits16),
        (LoadKind::Extend { result_ty: Ty::U32 }, Ty::Bits8),
        (LoadKind::Extend { result_ty: Ty::U32 }, Ty::Bits16),
        // i64
        (LoadKind::Extend { result_ty: Ty::I64 }, Ty::Bits8),
        (LoadKind::Extend { result_ty: Ty::I64 }, Ty::Bits16),
        (LoadKind::Extend { result_ty: Ty::I64 }, Ty::Bits32),
        (LoadKind::Extend { result_ty: Ty::U64 }, Ty::Bits8),
        (LoadKind::Extend { result_ty: Ty::U64 }, Ty::Bits16),
        (LoadKind::Extend { result_ty: Ty::U64 }, Ty::Bits32),
    ];
    for (kind, loaded_ty) in ops {
        for ptr in [OperandKind::Slot, OperandKind::Immediate] {
            isa.push_op(LoadOp::new(
                kind,
                loaded_ty,
                ptr,
                MemoryOperand::Immediate,
                OffsetOperand::Offset,
            ));
        }
        isa.push_op(LoadOp::new(
            kind,
            loaded_ty,
            OperandKind::Slot,
            MemoryOperand::Mem0,
            OffsetOperand::Offset16,
        ));
    }
}

fn add_store_ops(isa: &mut Isa) {
    #[rustfmt::skip]
    let ops = [
        // Generic
        (StoreKind::Value, Ty::U32),
        (StoreKind::Value, Ty::U64),
        // i32
        (StoreKind::Wrap { stored_ty: Ty::SignedBits8 }, Ty::I32),
        (StoreKind::Wrap { stored_ty: Ty::SignedBits16 }, Ty::I32),
        // i64
        (StoreKind::Wrap { stored_ty: Ty::SignedBits8 }, Ty::I64),
        (StoreKind::Wrap { stored_ty: Ty::SignedBits16 }, Ty::I64),
        (StoreKind::Wrap { stored_ty: Ty::SignedBits32 }, Ty::I64),
    ];
    for (kind, value_ty) in ops {
        for value in [OperandKind::Slot, OperandKind::Immediate] {
            for ptr in [OperandKind::Slot, OperandKind::Immediate] {
                isa.push_op(StoreOp::new(
                    kind,
                    value_ty,
                    ptr,
                    value,
                    MemoryOperand::Immediate,
                    OffsetOperand::Offset,
                ));
            }
            isa.push_op(StoreOp::new(
                kind,
                value_ty,
                OperandKind::Slot,
                value,
                MemoryOperand::Mem0,
                OffsetOperand::Offset16,
            ));
        }
    }
}

fn add_control_ops(isa: &mut Isa) {
    let ops = [
        Op::from(GenericOp::new(
            Ident::Trap,
            [Field::new(Ident::TrapCode, FieldTy::TrapCode)],
        )),
        Op::from(GenericOp::new(
            Ident::ConsumeFuel,
            [Field::new(Ident::Fuel, FieldTy::BlockFuel)],
        )),
        Op::from(GenericOp::new(Ident::Return, [])),
        Op::from(GenericOp::new(
            Ident::ReturnSlot,
            [Field::new(Ident::Value, FieldTy::Slot)],
        )),
        Op::from(GenericOp::new(
            Ident::ReturnImm32,
            [Field::new(Ident::Value, FieldTy::U32)],
        )),
        Op::from(GenericOp::new(
            Ident::ReturnImm64,
            [Field::new(Ident::Value, FieldTy::U64)],
        )),
        Op::from(GenericOp::new(
            Ident::ReturnSpan,
            [Field::new(Ident::Values, FieldTy::BoundedSlotSpan)],
        )),
        Op::from(GenericOp::new(
            Ident::Branch,
            [Field::new(Ident::Offset, FieldTy::BranchOffset)],
        )),
        Op::from(GenericOp::new(
            Ident::BranchTable,
            [
                Field::new(Ident::LenTargets, FieldTy::U32),
                Field::new(Ident::Index, FieldTy::Slot),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::BranchTableSpan,
            [
                Field::new(Ident::LenTargets, FieldTy::U32),
                Field::new(Ident::Index, FieldTy::Slot),
                Field::new(Ident::Values, FieldTy::SlotSpan),
                Field::new(Ident::LenValues, FieldTy::U16),
            ],
        )),
    ];
    isa.push_ops(ops);
}

fn add_copy_ops(isa: &mut Isa) {
    let ops = [
        Op::from(GenericOp::new(
            Ident::CopySlot,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Value, FieldTy::Slot),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::CopyImm32,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Value, FieldTy::U32),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::CopyImm64,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Value, FieldTy::U64),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::CopySpanAsc,
            [
                Field::new(Ident::Results, FieldTy::SlotSpan),
                Field::new(Ident::Values, FieldTy::SlotSpan),
                Field::new(Ident::Len, FieldTy::U16),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::CopySpanDes,
            [
                Field::new(Ident::Results, FieldTy::SlotSpan),
                Field::new(Ident::Values, FieldTy::SlotSpan),
                Field::new(Ident::Len, FieldTy::U16),
            ],
        )),
    ];
    isa.push_ops(ops);
}

fn add_call_ops(isa: &mut Isa) {
    let ops = [
        Op::from(GenericOp::new(
            Ident::RefFunc,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Func, FieldTy::Func),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::CallInternal,
            [
                Field::new(Ident::Params, FieldTy::BoundedSlotSpan),
                Field::new(Ident::Func, FieldTy::InternalFunc),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::CallImported,
            [
                Field::new(Ident::Params, FieldTy::BoundedSlotSpan),
                Field::new(Ident::Func, FieldTy::Func),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::CallIndirect,
            [
                Field::new(Ident::Params, FieldTy::BoundedSlotSpan),
                Field::new(Ident::Index, FieldTy::Slot),
                Field::new(Ident::FuncType, FieldTy::FuncType),
                Field::new(Ident::Table, FieldTy::Table),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::ReturnCallInternal,
            [
                Field::new(Ident::Params, FieldTy::BoundedSlotSpan),
                Field::new(Ident::Func, FieldTy::InternalFunc),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::ReturnCallImported,
            [
                Field::new(Ident::Params, FieldTy::BoundedSlotSpan),
                Field::new(Ident::Func, FieldTy::Func),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::ReturnCallIndirect,
            [
                Field::new(Ident::Params, FieldTy::BoundedSlotSpan),
                Field::new(Ident::Index, FieldTy::Slot),
                Field::new(Ident::FuncType, FieldTy::FuncType),
                Field::new(Ident::Table, FieldTy::Table),
            ],
        )),
    ];
    isa.push_ops(ops);
}

fn add_global_ops(isa: &mut Isa) {
    let ops = [
        Op::from(GenericOp::new(
            Ident::GlobalGet64,
            [
                Field::new(Ident::Global, FieldTy::Global),
                Field::new(Ident::Result, FieldTy::Slot),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::GlobalSet32I,
            [
                Field::new(Ident::Global, FieldTy::Global),
                Field::new(Ident::Value, FieldTy::U32),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::GlobalSet64S,
            [
                Field::new(Ident::Global, FieldTy::Global),
                Field::new(Ident::Value, FieldTy::Slot),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::GlobalSet64I,
            [
                Field::new(Ident::Value, FieldTy::U64),
                Field::new(Ident::Global, FieldTy::Global),
            ],
        )),
    ];
    isa.push_ops(ops);
}

fn add_table_ops(isa: &mut Isa) {
    let ops = [
        Op::TableGet(TableGetOp::new(OperandKind::Slot)),
        Op::TableGet(TableGetOp::new(OperandKind::Immediate)),
        Op::TableSet(TableSetOp::new(OperandKind::Slot, OperandKind::Slot)),
        Op::TableSet(TableSetOp::new(OperandKind::Slot, OperandKind::Immediate)),
        Op::TableSet(TableSetOp::new(OperandKind::Immediate, OperandKind::Slot)),
        Op::TableSet(TableSetOp::new(
            OperandKind::Immediate,
            OperandKind::Immediate,
        )),
        Op::from(GenericOp::new(
            Ident::TableSize,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Table, FieldTy::Table),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::TableGrow,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Delta, FieldTy::Slot),
                Field::new(Ident::Value, FieldTy::Slot),
                Field::new(Ident::Table, FieldTy::Table),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::TableCopy,
            [
                Field::new(Ident::DstTable, FieldTy::Table),
                Field::new(Ident::SrcTable, FieldTy::Table),
                Field::new(Ident::Dst, FieldTy::Slot),
                Field::new(Ident::Src, FieldTy::Slot),
                Field::new(Ident::Len, FieldTy::Slot),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::TableFill,
            [
                Field::new(Ident::Table, FieldTy::Table),
                Field::new(Ident::Dst, FieldTy::Slot),
                Field::new(Ident::Len, FieldTy::Slot),
                Field::new(Ident::Value, FieldTy::Slot),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::TableInit,
            [
                Field::new(Ident::Table, FieldTy::Table),
                Field::new(Ident::Elem, FieldTy::Elem),
                Field::new(Ident::Dst, FieldTy::Slot),
                Field::new(Ident::Src, FieldTy::Slot),
                Field::new(Ident::Len, FieldTy::Slot),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::ElemDrop,
            [Field::new(Ident::Elem, FieldTy::Elem)],
        )),
    ];
    isa.push_ops(ops);
}

fn add_memory_ops(isa: &mut Isa) {
    let ops = [
        Op::from(GenericOp::new(
            Ident::DataDrop,
            [Field::new(Ident::Data, FieldTy::Data)],
        )),
        Op::from(GenericOp::new(
            Ident::MemorySize,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Memory, FieldTy::Memory),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::MemoryGrow,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Delta, FieldTy::Slot),
                Field::new(Ident::Memory, FieldTy::Memory),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::MemoryCopy,
            [
                Field::new(Ident::DstMemory, FieldTy::Memory),
                Field::new(Ident::SrcMemory, FieldTy::Memory),
                Field::new(Ident::Dst, FieldTy::Slot),
                Field::new(Ident::Src, FieldTy::Slot),
                Field::new(Ident::Len, FieldTy::Slot),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::MemoryFill,
            [
                Field::new(Ident::Memory, FieldTy::Memory),
                Field::new(Ident::Dst, FieldTy::Slot),
                Field::new(Ident::Len, FieldTy::Slot),
                Field::new(Ident::Value, FieldTy::Slot),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::MemoryInit,
            [
                Field::new(Ident::Memory, FieldTy::Memory),
                Field::new(Ident::Data, FieldTy::Data),
                Field::new(Ident::Dst, FieldTy::Slot),
                Field::new(Ident::Src, FieldTy::Slot),
                Field::new(Ident::Len, FieldTy::Slot),
            ],
        )),
    ];
    isa.push_ops(ops);
}

fn add_wide_arithmetic_ops(isa: &mut Isa) {
    let ops = [
        Op::from(GenericOp::new(
            Ident::I64Add128,
            [
                Field::new(Ident::Results, FieldTy::FixedSlotSpan2),
                Field::new(Ident::LhsLo, FieldTy::Slot),
                Field::new(Ident::LhsHi, FieldTy::Slot),
                Field::new(Ident::RhsLo, FieldTy::Slot),
                Field::new(Ident::RhsHi, FieldTy::Slot),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::I64Sub128,
            [
                Field::new(Ident::Results, FieldTy::FixedSlotSpan2),
                Field::new(Ident::LhsLo, FieldTy::Slot),
                Field::new(Ident::LhsHi, FieldTy::Slot),
                Field::new(Ident::RhsLo, FieldTy::Slot),
                Field::new(Ident::RhsHi, FieldTy::Slot),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::I64MulWide,
            [
                Field::new(Ident::Results, FieldTy::FixedSlotSpan2),
                Field::new(Ident::Lhs, FieldTy::Slot),
                Field::new(Ident::Rhs, FieldTy::Slot),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::U64MulWide,
            [
                Field::new(Ident::Results, FieldTy::FixedSlotSpan2),
                Field::new(Ident::Lhs, FieldTy::Slot),
                Field::new(Ident::Rhs, FieldTy::Slot),
            ],
        )),
    ];
    isa.push_ops(ops);
}

fn add_simd_ops(isa: &mut Isa, config: &Config) {
    if !config.simd {
        return;
    }
    isa.push_op(GenericOp::new(
        Ident::CopyImm128,
        [
            Field::new(Ident::Result, FieldTy::Slot),
            Field::new(Ident::ValueLo, FieldTy::U64),
            Field::new(Ident::ValueHi, FieldTy::U64),
        ],
    ));
    isa.push_op(GenericOp::new(
        Ident::I8x16Shuffle,
        [
            Field::new(Ident::Result, FieldTy::Slot),
            Field::new(Ident::Lhs, FieldTy::Slot),
            Field::new(Ident::Rhs, FieldTy::Slot),
            Field::new(Ident::Selector, FieldTy::Array16ImmLaneIdx32),
        ],
    ));
    isa.push_op(Op::from(GenericOp::new(
        Ident::GlobalSet128S,
        [
            Field::new(Ident::Global, FieldTy::Global),
            Field::new(Ident::Value, FieldTy::Slot),
        ],
    )));
    isa.push_op(Op::from(GenericOp::new(
        Ident::GlobalGet128,
        [
            Field::new(Ident::Global, FieldTy::Global),
            Field::new(Ident::Result, FieldTy::Slot),
        ],
    )));
    isa.push_op(Op::from(GenericOp::new(
        Ident::Select128,
        [
            Field::new(Ident::Result, FieldTy::Slot),
            Field::new(Ident::Selector, FieldTy::Slot),
            Field::new(Ident::ValTrue, FieldTy::Slot),
            Field::new(Ident::ValFalse, FieldTy::Slot),
        ],
    )));
    add_simd_splat_ops(isa);
    add_simd_extract_lane_ops(isa);
    add_simd_replace_lane_ops(isa);
    add_simd_binary_ops(isa);
    add_simd_shift_ops(isa);
    add_simd_unary_ops(isa);
    add_simd_load_ops(isa);
    add_simd_store_ops(isa);
    add_relaxed_simd_ops(isa);
}

fn add_simd_splat_ops(isa: &mut Isa) {
    let kinds = [
        (Ident::Splat, Ty::Bits8),
        (Ident::Splat, Ty::Bits16),
        (Ident::Splat, Ty::Bits32),
        (Ident::Splat, Ty::Bits64),
    ];
    for (ident, value_ty) in kinds {
        isa.push_op(UnaryOp::new(ident, Ty::V128, value_ty, OperandKind::Slot));
        isa.push_op(UnaryOp::new(
            ident,
            Ty::V128,
            value_ty,
            OperandKind::Immediate,
        ));
    }
}

fn add_simd_extract_lane_ops(isa: &mut Isa) {
    let ops = [
        V128ExtractLaneOp::new(SimdTy::I8x16),
        V128ExtractLaneOp::new(SimdTy::U8x16),
        V128ExtractLaneOp::new(SimdTy::I16x8),
        V128ExtractLaneOp::new(SimdTy::U16x8),
        V128ExtractLaneOp::new(SimdTy::U32x4),
        V128ExtractLaneOp::new(SimdTy::U64x2),
    ]
    .map(Op::from);
    isa.push_ops(ops);
}

fn add_simd_replace_lane_ops(isa: &mut Isa) {
    let widths = [
        LaneWidth::W8,
        LaneWidth::W16,
        LaneWidth::W32,
        LaneWidth::W64,
    ];
    for width in widths {
        isa.push_op(V128ReplaceLaneOp::new(width, OperandKind::Slot));
        isa.push_op(V128ReplaceLaneOp::new(width, OperandKind::Immediate));
    }
}

fn add_simd_binary_ops(isa: &mut Isa) {
    #[rustfmt::skip]
    let ops = [
        // Miscellaneous
        (Ident::Swizzle, Ty::I8x16, Ty::I8x16, BinaryOpCaps::NONE),
        // Integer Comparisons
        (Ident::Eq, Ty::I8x16, Ty::I8x16, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::NotEq, Ty::I8x16, Ty::I8x16, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::Eq, Ty::I16x8, Ty::I16x8, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::NotEq, Ty::I16x8, Ty::I16x8, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::Eq, Ty::I32x4, Ty::I32x4, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::NotEq, Ty::I32x4, Ty::I32x4, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::Eq, Ty::I64x2, Ty::I64x2, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::NotEq, Ty::I64x2, Ty::I64x2, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::Lt, Ty::I8x16, Ty::I8x16, BinaryOpCaps::CMP),
        (Ident::Le, Ty::I8x16, Ty::I8x16, BinaryOpCaps::CMP),
        (Ident::Lt, Ty::I16x8, Ty::I16x8, BinaryOpCaps::CMP),
        (Ident::Le, Ty::I16x8, Ty::I16x8, BinaryOpCaps::CMP),
        (Ident::Lt, Ty::I32x4, Ty::I32x4, BinaryOpCaps::CMP),
        (Ident::Le, Ty::I32x4, Ty::I32x4, BinaryOpCaps::CMP),
        (Ident::Lt, Ty::I64x2, Ty::I64x2, BinaryOpCaps::CMP),
        (Ident::Le, Ty::I64x2, Ty::I64x2, BinaryOpCaps::CMP),
        (Ident::Lt, Ty::U8x16, Ty::U8x16, BinaryOpCaps::CMP),
        (Ident::Le, Ty::U8x16, Ty::U8x16, BinaryOpCaps::CMP),
        (Ident::Lt, Ty::U16x8, Ty::U16x8, BinaryOpCaps::CMP),
        (Ident::Le, Ty::U16x8, Ty::U16x8, BinaryOpCaps::CMP),
        (Ident::Lt, Ty::U32x4, Ty::U32x4, BinaryOpCaps::CMP),
        (Ident::Le, Ty::U32x4, Ty::U32x4, BinaryOpCaps::CMP),
        // Float Comparisons
        (Ident::Eq, Ty::F32x4, Ty::F32x4, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::NotEq, Ty::F32x4, Ty::F32x4, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::Lt, Ty::F32x4, Ty::F32x4, BinaryOpCaps::CMP),
        (Ident::Le, Ty::F32x4, Ty::F32x4, BinaryOpCaps::CMP),
        (Ident::Eq, Ty::F64x2, Ty::F64x2, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::NotEq, Ty::F64x2, Ty::F64x2, BinaryOpCaps::CMP | BinaryOpCaps::COMMUTATIVE),
        (Ident::Lt, Ty::F64x2, Ty::F64x2, BinaryOpCaps::CMP),
        (Ident::Le, Ty::F64x2, Ty::F64x2, BinaryOpCaps::CMP),
        // Bitwise
        (Ident::And, Ty::V128, Ty::V128, BinaryOpCaps::COMMUTATIVE),
        (Ident::AndNot, Ty::V128, Ty::V128, BinaryOpCaps::COMMUTATIVE),
        (Ident::Or, Ty::V128, Ty::V128, BinaryOpCaps::COMMUTATIVE),
        (Ident::Xor, Ty::V128, Ty::V128, BinaryOpCaps::COMMUTATIVE),
        // i8x16 Ops
        (Ident::Narrow, Ty::I8x16, Ty::I16x8, BinaryOpCaps::NONE),
        (Ident::Narrow, Ty::U8x16, Ty::I16x8, BinaryOpCaps::NONE),
        (Ident::Add, Ty::I8x16, Ty::I8x16, BinaryOpCaps::COMMUTATIVE),
        (Ident::AddSat, Ty::I8x16, Ty::I8x16, BinaryOpCaps::COMMUTATIVE),
        (Ident::AddSat, Ty::U8x16, Ty::U8x16, BinaryOpCaps::COMMUTATIVE),
        (Ident::Sub, Ty::I8x16, Ty::I8x16, BinaryOpCaps::NONE),
        (Ident::SubSat, Ty::I8x16, Ty::I8x16, BinaryOpCaps::NONE),
        (Ident::SubSat, Ty::U8x16, Ty::U8x16, BinaryOpCaps::NONE),
        (Ident::Min, Ty::I8x16, Ty::I8x16, BinaryOpCaps::COMMUTATIVE),
        (Ident::Min, Ty::U8x16, Ty::U8x16, BinaryOpCaps::COMMUTATIVE),
        (Ident::Max, Ty::I8x16, Ty::I8x16, BinaryOpCaps::COMMUTATIVE),
        (Ident::Max, Ty::U8x16, Ty::U8x16, BinaryOpCaps::COMMUTATIVE),
        (Ident::Avgr, Ty::U8x16, Ty::U8x16, BinaryOpCaps::COMMUTATIVE),
        // i16x8 Ops
        (Ident::RelaxedDotI8x16I7x16, Ty::I16x8, Ty::I16x8, BinaryOpCaps::NONE), // TODO: what to do for `input_ty`?
        (Ident::Q15MulrSat, Ty::I16x8, Ty::I16x8, BinaryOpCaps::COMMUTATIVE),
        (Ident::Narrow, Ty::I16x8, Ty::I32x4, BinaryOpCaps::NONE),
        (Ident::Narrow, Ty::U16x8, Ty::I32x4, BinaryOpCaps::NONE),
        (Ident::ExtmulLow, Ty::I16x8, Ty::I8x16, BinaryOpCaps::NONE),
        (Ident::ExtmulLow, Ty::U16x8, Ty::I8x16, BinaryOpCaps::NONE),
        (Ident::ExtmulHigh, Ty::I16x8, Ty::I8x16, BinaryOpCaps::NONE),
        (Ident::ExtmulHigh, Ty::U16x8, Ty::I8x16, BinaryOpCaps::NONE),
        (Ident::Add, Ty::I16x8, Ty::I16x8, BinaryOpCaps::COMMUTATIVE),
        (Ident::AddSat, Ty::I16x8, Ty::I16x8, BinaryOpCaps::COMMUTATIVE),
        (Ident::AddSat, Ty::U16x8, Ty::U16x8, BinaryOpCaps::COMMUTATIVE),
        (Ident::Sub, Ty::I16x8, Ty::I16x8, BinaryOpCaps::NONE),
        (Ident::SubSat, Ty::I16x8, Ty::I16x8, BinaryOpCaps::NONE),
        (Ident::SubSat, Ty::U16x8, Ty::U16x8, BinaryOpCaps::NONE),
        (Ident::Mul, Ty::I16x8, Ty::I16x8, BinaryOpCaps::COMMUTATIVE),
        (Ident::Min, Ty::I16x8, Ty::I16x8, BinaryOpCaps::COMMUTATIVE),
        (Ident::Min, Ty::U16x8, Ty::U16x8, BinaryOpCaps::COMMUTATIVE),
        (Ident::Max, Ty::I16x8, Ty::I16x8, BinaryOpCaps::COMMUTATIVE),
        (Ident::Max, Ty::U16x8, Ty::U16x8, BinaryOpCaps::COMMUTATIVE),
        (Ident::Avgr, Ty::U16x8, Ty::U16x8, BinaryOpCaps::COMMUTATIVE),
        // i32x4 Ops
        (Ident::Add, Ty::I32x4, Ty::I32x4, BinaryOpCaps::COMMUTATIVE),
        (Ident::Sub, Ty::I32x4, Ty::I32x4, BinaryOpCaps::NONE),
        (Ident::Mul, Ty::I32x4, Ty::I32x4, BinaryOpCaps::COMMUTATIVE),
        (Ident::Min, Ty::I32x4, Ty::I32x4, BinaryOpCaps::COMMUTATIVE),
        (Ident::Min, Ty::U32x4, Ty::U32x4, BinaryOpCaps::COMMUTATIVE),
        (Ident::Max, Ty::I32x4, Ty::I32x4, BinaryOpCaps::COMMUTATIVE),
        (Ident::Max, Ty::U32x4, Ty::U32x4, BinaryOpCaps::COMMUTATIVE),
        (Ident::Dot, Ty::I32x4, Ty::I16x8, BinaryOpCaps::COMMUTATIVE),
        (Ident::ExtmulLow, Ty::I32x4, Ty::I16x8, BinaryOpCaps::COMMUTATIVE),
        (Ident::ExtmulLow, Ty::U32x4, Ty::I16x8, BinaryOpCaps::COMMUTATIVE),
        (Ident::ExtmulHigh, Ty::I32x4, Ty::I16x8, BinaryOpCaps::COMMUTATIVE),
        (Ident::ExtmulHigh, Ty::U32x4, Ty::I16x8, BinaryOpCaps::COMMUTATIVE),
        // i64x2 Ops
        (Ident::Add, Ty::I64x2, Ty::I64x2, BinaryOpCaps::COMMUTATIVE),
        (Ident::Sub, Ty::I64x2, Ty::I64x2, BinaryOpCaps::NONE),
        (Ident::Mul, Ty::I64x2, Ty::I64x2, BinaryOpCaps::COMMUTATIVE),
        (Ident::ExtmulLow, Ty::I64x2, Ty::I32x4, BinaryOpCaps::COMMUTATIVE),
        (Ident::ExtmulLow, Ty::U64x2, Ty::I32x4, BinaryOpCaps::COMMUTATIVE),
        (Ident::ExtmulHigh, Ty::I64x2, Ty::I32x4, BinaryOpCaps::COMMUTATIVE),
        (Ident::ExtmulHigh, Ty::U64x2, Ty::I32x4, BinaryOpCaps::COMMUTATIVE),
        // f32x4 Ops
        (Ident::Add, Ty::F32x4, Ty::F32x4, BinaryOpCaps::NONE),
        (Ident::Sub, Ty::F32x4, Ty::F32x4, BinaryOpCaps::NONE),
        (Ident::Mul, Ty::F32x4, Ty::F32x4, BinaryOpCaps::NONE),
        (Ident::Div, Ty::F32x4, Ty::F32x4, BinaryOpCaps::NONE),
        (Ident::Min, Ty::F32x4, Ty::F32x4, BinaryOpCaps::NONE),
        (Ident::Max, Ty::F32x4, Ty::F32x4, BinaryOpCaps::NONE),
        (Ident::Pmin, Ty::F32x4, Ty::F32x4, BinaryOpCaps::NONE),
        (Ident::Pmax, Ty::F32x4, Ty::F32x4, BinaryOpCaps::NONE),
        // f64x2 Ops
        (Ident::Add, Ty::F64x2, Ty::F64x2, BinaryOpCaps::NONE),
        (Ident::Sub, Ty::F64x2, Ty::F64x2, BinaryOpCaps::NONE),
        (Ident::Mul, Ty::F64x2, Ty::F64x2, BinaryOpCaps::NONE),
        (Ident::Div, Ty::F64x2, Ty::F64x2, BinaryOpCaps::NONE),
        (Ident::Min, Ty::F64x2, Ty::F64x2, BinaryOpCaps::NONE),
        (Ident::Max, Ty::F64x2, Ty::F64x2, BinaryOpCaps::NONE),
        (Ident::Pmin, Ty::F64x2, Ty::F64x2, BinaryOpCaps::NONE),
        (Ident::Pmax, Ty::F64x2, Ty::F64x2, BinaryOpCaps::NONE),
    ];
    for (ident, result_ty, input_ty, caps) in ops {
        isa.push_op(BinaryOp::new(
            ident,
            result_ty,
            input_ty,
            input_ty,
            OperandKind::Slot,
            OperandKind::Slot,
            caps,
        ));
    }
}

fn add_simd_shift_ops(isa: &mut Isa) {
    let ops = [
        (Ident::Shl, Ty::I8x16, Ty::I8x16),
        (Ident::Shr, Ty::I8x16, Ty::I8x16),
        (Ident::Shr, Ty::U8x16, Ty::U8x16),
        (Ident::Shl, Ty::I16x8, Ty::I16x8),
        (Ident::Shr, Ty::I16x8, Ty::I16x8),
        (Ident::Shr, Ty::U16x8, Ty::U16x8),
        (Ident::Shl, Ty::I32x4, Ty::I32x4),
        (Ident::Shr, Ty::I32x4, Ty::I32x4),
        (Ident::Shr, Ty::U32x4, Ty::U32x4),
        (Ident::Shl, Ty::I64x2, Ty::I64x2),
        (Ident::Shr, Ty::I64x2, Ty::I64x2),
        (Ident::Shr, Ty::U64x2, Ty::U64x2),
    ];
    for (ident, result_ty, lhs_ty) in ops {
        for rhs in [OperandKind::Slot, OperandKind::Immediate] {
            isa.push_op(BinaryOp::new(
                ident,
                result_ty,
                lhs_ty,
                Ty::U8,
                OperandKind::Slot,
                rhs,
                BinaryOpCaps::NONE,
            ));
        }
    }
}

fn add_simd_unary_ops(isa: &mut Isa) {
    let kinds = [
        // SIMD: Generic Unary Ops
        (Ident::Not, Ty::V128, Ty::V128),
        (Ident::AnyTrue, Ty::V128, Ty::V128),
        // SIMD: `i8x16` Unary Ops
        (Ident::Abs, Ty::I8x16, Ty::I8x16),
        (Ident::Neg, Ty::I8x16, Ty::I8x16),
        (Ident::Popcnt, Ty::I8x16, Ty::I8x16),
        (Ident::AllTrue, Ty::I8x16, Ty::I8x16),
        (Ident::Bitmask, Ty::I8x16, Ty::I8x16),
        // SIMD: `i16x8` Unary Ops
        (Ident::Abs, Ty::I16x8, Ty::I16x8),
        (Ident::Neg, Ty::I16x8, Ty::I16x8),
        (Ident::AllTrue, Ty::I16x8, Ty::I16x8),
        (Ident::Bitmask, Ty::I16x8, Ty::I16x8),
        (Ident::ExtaddPairwise, Ty::I16x8, Ty::I8x16),
        (Ident::ExtaddPairwise, Ty::U16x8, Ty::I8x16),
        (Ident::ExtendLow, Ty::I16x8, Ty::I8x16),
        (Ident::ExtendLow, Ty::U16x8, Ty::I8x16),
        (Ident::ExtendHigh, Ty::I16x8, Ty::I8x16),
        (Ident::ExtendHigh, Ty::U16x8, Ty::I8x16),
        // SIMD: `i32x4` Unary Ops
        (Ident::Abs, Ty::I32x4, Ty::I32x4),
        (Ident::Neg, Ty::I32x4, Ty::I32x4),
        (Ident::AllTrue, Ty::I32x4, Ty::I32x4),
        (Ident::Bitmask, Ty::I32x4, Ty::I32x4),
        (Ident::ExtaddPairwise, Ty::I32x4, Ty::I16x8),
        (Ident::ExtaddPairwise, Ty::U32x4, Ty::I16x8),
        (Ident::ExtendLow, Ty::I32x4, Ty::I16x8),
        (Ident::ExtendLow, Ty::U32x4, Ty::I16x8),
        (Ident::ExtendHigh, Ty::I32x4, Ty::I16x8),
        (Ident::ExtendHigh, Ty::U32x4, Ty::I16x8),
        // SIMD: `i64x2` Unary Ops
        (Ident::Abs, Ty::I64x2, Ty::I64x2),
        (Ident::Neg, Ty::I64x2, Ty::I64x2),
        (Ident::AllTrue, Ty::I64x2, Ty::I64x2),
        (Ident::Bitmask, Ty::I64x2, Ty::I64x2),
        (Ident::ExtendLow, Ty::I64x2, Ty::I32x4),
        (Ident::ExtendLow, Ty::U64x2, Ty::I32x4),
        (Ident::ExtendHigh, Ty::I64x2, Ty::I32x4),
        (Ident::ExtendHigh, Ty::U64x2, Ty::I32x4),
        // SIMD: `f32x4` Unary Ops
        (Ident::DemoteZero, Ty::F32x4, Ty::F64x2),
        (Ident::Ceil, Ty::F32x4, Ty::F32x4),
        (Ident::Floor, Ty::F32x4, Ty::F32x4),
        (Ident::Trunc, Ty::F32x4, Ty::F32x4),
        (Ident::Nearest, Ty::F32x4, Ty::F32x4),
        (Ident::Abs, Ty::F32x4, Ty::F32x4),
        (Ident::Neg, Ty::F32x4, Ty::F32x4),
        (Ident::Sqrt, Ty::F32x4, Ty::F32x4),
        // SIMD: `f64x2` Unary Ops
        (Ident::PromoteLow, Ty::F64x2, Ty::F32x4),
        (Ident::Ceil, Ty::F64x2, Ty::F64x2),
        (Ident::Floor, Ty::F64x2, Ty::F64x2),
        (Ident::Trunc, Ty::F64x2, Ty::F64x2),
        (Ident::Nearest, Ty::F64x2, Ty::F64x2),
        (Ident::Abs, Ty::F64x2, Ty::F64x2),
        (Ident::Neg, Ty::F64x2, Ty::F64x2),
        (Ident::Sqrt, Ty::F64x2, Ty::F64x2),
        // SIMD: Conversions
        (Ident::TruncSat, Ty::I32x4, Ty::F32x4),
        (Ident::TruncSat, Ty::U32x4, Ty::F32x4),
        (Ident::TruncSatZero, Ty::I32x4, Ty::F64x2),
        (Ident::TruncSatZero, Ty::U32x4, Ty::F64x2),
        (Ident::Convert, Ty::F32x4, Ty::I32x4),
        (Ident::Convert, Ty::F32x4, Ty::U32x4),
        (Ident::ConvertLow, Ty::F64x2, Ty::I32x4),
        (Ident::ConvertLow, Ty::F64x2, Ty::U32x4),
    ];
    for (ident, result_ty, value_ty) in kinds {
        isa.push_op(UnaryOp::new(ident, result_ty, value_ty, OperandKind::Slot));
    }
}

fn add_simd_load_ops(isa: &mut Isa) {
    #[rustfmt::skip]
    let ops = [
        (LoadKind::Value, Ty::V128),
        // load-widen
        (LoadKind::Widen { result_ty: Ty::I16x8 }, Ty::Bits8x8),
        (LoadKind::Widen { result_ty: Ty::U16x8 }, Ty::Bits8x8),
        (LoadKind::Widen { result_ty: Ty::I32x4 }, Ty::Bits16x4),
        (LoadKind::Widen { result_ty: Ty::U32x4 }, Ty::Bits16x4),
        (LoadKind::Widen { result_ty: Ty::I64x2 }, Ty::Bits32x2),
        (LoadKind::Widen { result_ty: Ty::U64x2 }, Ty::Bits32x2),
        // load-splat
        (LoadKind::Splat, Ty::Bits8),
        (LoadKind::Splat, Ty::Bits16),
        (LoadKind::Splat, Ty::Bits32),
        (LoadKind::Splat, Ty::Bits64),
        // load-low
        (LoadKind::Low, Ty::Bits32),
        (LoadKind::Low, Ty::Bits64),
        // load-lane
        (LoadKind::Lane { width: LaneWidth::W8 }, Ty::Bits8),
        (LoadKind::Lane { width: LaneWidth::W16 }, Ty::Bits16),
        (LoadKind::Lane { width: LaneWidth::W32 }, Ty::Bits32),
        (LoadKind::Lane { width: LaneWidth::W64 }, Ty::Bits64),
    ];
    for (kind, loaded_ty) in ops {
        isa.push_op(LoadOp::new(
            kind,
            loaded_ty,
            OperandKind::Slot,
            MemoryOperand::Immediate,
            OffsetOperand::Offset,
        ));
        isa.push_op(LoadOp::new(
            kind,
            loaded_ty,
            OperandKind::Slot,
            MemoryOperand::Mem0,
            OffsetOperand::Offset16,
        ));
    }
}

fn add_simd_store_ops(isa: &mut Isa) {
    #[rustfmt::skip]
    let kinds = [
        (StoreKind::Value, Ty::V128),
        (StoreKind::Lane { width: LaneWidth::W8 }, Ty::V128),
        (StoreKind::Lane { width: LaneWidth::W16 }, Ty::V128),
        (StoreKind::Lane { width: LaneWidth::W32 }, Ty::V128),
        (StoreKind::Lane { width: LaneWidth::W64 }, Ty::V128),
    ];
    for (kind, value_ty) in kinds {
        isa.push_op(StoreOp::new(
            kind,
            value_ty,
            OperandKind::Slot,
            OperandKind::Slot,
            MemoryOperand::Immediate,
            OffsetOperand::Offset,
        ));
        isa.push_op(StoreOp::new(
            kind,
            value_ty,
            OperandKind::Slot,
            OperandKind::Slot,
            MemoryOperand::Mem0,
            OffsetOperand::Offset16,
        ));
    }
}

fn add_relaxed_simd_ops(isa: &mut Isa) {
    let kinds = [
        TernaryOpKind::I32x4RelaxedDotI8x16I7x16Add,
        TernaryOpKind::F32x4RelaxedMadd,
        TernaryOpKind::F32x4RelaxedNmadd,
        TernaryOpKind::F64x2RelaxedMadd,
        TernaryOpKind::F64x2RelaxedNmadd,
        TernaryOpKind::V128Bitselect,
    ];
    for kind in kinds {
        isa.push_op(TernaryOp::new(kind));
    }
}
