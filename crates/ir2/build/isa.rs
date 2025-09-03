use crate::build::{
    ident::Ident,
    op::{
        BinaryOp,
        BinaryOpKind,
        CmpBranchOp,
        CmpOpKind,
        CmpSelectOp,
        Commutativity,
        Field,
        FieldTy,
        GenericOp,
        LaneWidth,
        LoadOp,
        LoadOpKind,
        OperandKind,
        StoreOp,
        StoreOpKind,
        TableGetOp,
        TableSetOp,
        UnaryOp,
        UnaryOpKind,
        V128LoadLaneOp,
        V128ReplaceLaneOp,
    },
    Config,
    Op,
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
    add_cmp_select_ops(&mut isa);
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
        UnaryOpKind::I32Clz,
        UnaryOpKind::I32Ctz,
        UnaryOpKind::I32Popcnt,
        UnaryOpKind::I32Sext8,
        UnaryOpKind::I32Sext16,
        UnaryOpKind::I32WrapI64,
        // i64
        UnaryOpKind::I64Clz,
        UnaryOpKind::I64Ctz,
        UnaryOpKind::I64Popcnt,
        UnaryOpKind::I64Sext8,
        UnaryOpKind::I64Sext16,
        UnaryOpKind::I64Sext32,
        // f32
        UnaryOpKind::F32Abs,
        UnaryOpKind::F32Neg,
        UnaryOpKind::F32Ceil,
        UnaryOpKind::F32Floor,
        UnaryOpKind::F32Trunc,
        UnaryOpKind::F32Nearest,
        UnaryOpKind::F32Sqrt,
        UnaryOpKind::F32ConvertS32,
        UnaryOpKind::F32ConvertU32,
        UnaryOpKind::F32ConvertS64,
        UnaryOpKind::F32ConvertU64,
        UnaryOpKind::F32DemoteF64,
        // f64
        UnaryOpKind::F64Abs,
        UnaryOpKind::F64Neg,
        UnaryOpKind::F64Ceil,
        UnaryOpKind::F64Floor,
        UnaryOpKind::F64Trunc,
        UnaryOpKind::F64Nearest,
        UnaryOpKind::F64Sqrt,
        UnaryOpKind::F64ConvertS32,
        UnaryOpKind::F64ConvertU32,
        UnaryOpKind::F64ConvertS64,
        UnaryOpKind::F64ConvertU64,
        UnaryOpKind::F64PromoteF32,
        // f2i conversions
        UnaryOpKind::S32TruncF32,
        UnaryOpKind::U32TruncF32,
        UnaryOpKind::S32TruncF64,
        UnaryOpKind::U32TruncF64,
        UnaryOpKind::S64TruncF32,
        UnaryOpKind::U64TruncF32,
        UnaryOpKind::S64TruncF64,
        UnaryOpKind::U64TruncF64,
        UnaryOpKind::S32TruncSatF32,
        UnaryOpKind::U32TruncSatF32,
        UnaryOpKind::S32TruncSatF64,
        UnaryOpKind::U32TruncSatF64,
        UnaryOpKind::S64TruncSatF32,
        UnaryOpKind::U64TruncSatF32,
        UnaryOpKind::S64TruncSatF64,
        UnaryOpKind::U64TruncSatF64,
    ];
    for op in ops {
        isa.push_op(UnaryOp::new(op, OperandKind::Slot));
    }
}

fn add_binary_ops(isa: &mut Isa) {
    let ops = [
        // comparisons: i32
        BinaryOpKind::Cmp(CmpOpKind::I32Eq),
        BinaryOpKind::Cmp(CmpOpKind::I32NotEq),
        BinaryOpKind::Cmp(CmpOpKind::I32And),
        BinaryOpKind::Cmp(CmpOpKind::I32NotAnd),
        BinaryOpKind::Cmp(CmpOpKind::I32Or),
        BinaryOpKind::Cmp(CmpOpKind::I32NotOr),
        BinaryOpKind::Cmp(CmpOpKind::S32Lt),
        BinaryOpKind::Cmp(CmpOpKind::S32Le),
        BinaryOpKind::Cmp(CmpOpKind::U32Lt),
        BinaryOpKind::Cmp(CmpOpKind::U32Le),
        // comparisons: i64
        BinaryOpKind::Cmp(CmpOpKind::I64Eq),
        BinaryOpKind::Cmp(CmpOpKind::I64NotEq),
        BinaryOpKind::Cmp(CmpOpKind::I64And),
        BinaryOpKind::Cmp(CmpOpKind::I64NotAnd),
        BinaryOpKind::Cmp(CmpOpKind::I64Or),
        BinaryOpKind::Cmp(CmpOpKind::I64NotOr),
        BinaryOpKind::Cmp(CmpOpKind::S64Lt),
        BinaryOpKind::Cmp(CmpOpKind::S64Le),
        BinaryOpKind::Cmp(CmpOpKind::U64Lt),
        BinaryOpKind::Cmp(CmpOpKind::U64Le),
        // comparisons: f32
        BinaryOpKind::Cmp(CmpOpKind::F32Eq),
        BinaryOpKind::Cmp(CmpOpKind::F32NotEq),
        BinaryOpKind::Cmp(CmpOpKind::F32Lt),
        BinaryOpKind::Cmp(CmpOpKind::F32Le),
        // comparisons: f64
        BinaryOpKind::Cmp(CmpOpKind::F64Eq),
        BinaryOpKind::Cmp(CmpOpKind::F64NotEq),
        BinaryOpKind::Cmp(CmpOpKind::F64Lt),
        BinaryOpKind::Cmp(CmpOpKind::F64Le),
        // i32
        BinaryOpKind::I32Add,
        BinaryOpKind::I32Sub,
        BinaryOpKind::I32Mul,
        BinaryOpKind::S32Div,
        BinaryOpKind::U32Div,
        BinaryOpKind::S32Rem,
        BinaryOpKind::U32Rem,
        BinaryOpKind::I32BitAnd,
        BinaryOpKind::I32BitOr,
        BinaryOpKind::I32BitXor,
        BinaryOpKind::I32Shl,
        BinaryOpKind::S32Shr,
        BinaryOpKind::U32Shr,
        BinaryOpKind::I32Rotl,
        BinaryOpKind::I32Rotr,
        // i64
        BinaryOpKind::I64Add,
        BinaryOpKind::I64Sub,
        BinaryOpKind::I64Mul,
        BinaryOpKind::S64Div,
        BinaryOpKind::U64Div,
        BinaryOpKind::S64Rem,
        BinaryOpKind::U64Rem,
        BinaryOpKind::I64BitAnd,
        BinaryOpKind::I64BitOr,
        BinaryOpKind::I64BitXor,
        BinaryOpKind::I64Shl,
        BinaryOpKind::S64Shr,
        BinaryOpKind::U64Shr,
        BinaryOpKind::I64Rotl,
        BinaryOpKind::I64Rotr,
        // f32
        BinaryOpKind::F32Add,
        BinaryOpKind::F32Sub,
        BinaryOpKind::F32Mul,
        BinaryOpKind::F32Div,
        BinaryOpKind::F32Min,
        BinaryOpKind::F32Max,
        BinaryOpKind::F32Copysign,
        // f64
        BinaryOpKind::F64Add,
        BinaryOpKind::F64Sub,
        BinaryOpKind::F64Mul,
        BinaryOpKind::F64Div,
        BinaryOpKind::F64Min,
        BinaryOpKind::F64Max,
        BinaryOpKind::F64Copysign,
    ];
    for op in ops {
        isa.push_op(BinaryOp::new(op, OperandKind::Slot, OperandKind::Slot));
        isa.push_op(BinaryOp::new(op, OperandKind::Slot, OperandKind::Immediate));
        if matches!(op.commutativity(), Commutativity::NonCommutative) {
            isa.push_op(BinaryOp::new(op, OperandKind::Immediate, OperandKind::Slot));
        }
    }
}

fn add_cmp_branch_ops(isa: &mut Isa) {
    let ops = [
        // i32
        CmpOpKind::I32Eq,
        CmpOpKind::I32NotEq,
        CmpOpKind::I32And,
        CmpOpKind::I32NotAnd,
        CmpOpKind::I32Or,
        CmpOpKind::I32NotOr,
        CmpOpKind::S32Lt,
        CmpOpKind::S32Le,
        CmpOpKind::U32Lt,
        CmpOpKind::U32Le,
        // i64
        CmpOpKind::I64Eq,
        CmpOpKind::I64NotEq,
        CmpOpKind::I64And,
        CmpOpKind::I64NotAnd,
        CmpOpKind::I64Or,
        CmpOpKind::I64NotOr,
        CmpOpKind::S64Lt,
        CmpOpKind::S64Le,
        CmpOpKind::U64Lt,
        CmpOpKind::U64Le,
        // f32
        CmpOpKind::F32Eq,
        CmpOpKind::F32NotEq,
        CmpOpKind::F32Lt,
        CmpOpKind::F32NotLt,
        CmpOpKind::F32Le,
        CmpOpKind::F32NotLe,
        // f64
        CmpOpKind::F64Eq,
        CmpOpKind::F64NotEq,
        CmpOpKind::F64Lt,
        CmpOpKind::F64NotLt,
        CmpOpKind::F64Le,
        CmpOpKind::F64NotLe,
    ];
    for op in ops {
        isa.push_op(CmpBranchOp::new(op, OperandKind::Slot, OperandKind::Slot));
        isa.push_op(CmpBranchOp::new(
            op,
            OperandKind::Slot,
            OperandKind::Immediate,
        ));
        if matches!(op.commutativity(), Commutativity::NonCommutative) {
            isa.push_op(CmpBranchOp::new(
                op,
                OperandKind::Immediate,
                OperandKind::Slot,
            ));
        }
    }
}

fn add_cmp_select_ops(isa: &mut Isa) {
    let ops = [
        // i32
        CmpOpKind::I32Eq,
        CmpOpKind::I32And,
        CmpOpKind::I32Or,
        CmpOpKind::S32Lt,
        CmpOpKind::S32Le,
        CmpOpKind::U32Lt,
        CmpOpKind::U32Le,
        // i64
        CmpOpKind::I64Eq,
        CmpOpKind::I64And,
        CmpOpKind::I64Or,
        CmpOpKind::S64Lt,
        CmpOpKind::S64Le,
        CmpOpKind::U64Lt,
        CmpOpKind::U64Le,
        // f32
        CmpOpKind::F32Eq,
        CmpOpKind::F32Lt,
        CmpOpKind::F32Le,
        // f64
        CmpOpKind::F64Eq,
        CmpOpKind::F64Lt,
        CmpOpKind::F64Le,
    ];
    for op in ops {
        isa.push_op(CmpSelectOp::new(op, OperandKind::Slot, OperandKind::Slot));
        isa.push_op(CmpSelectOp::new(
            op,
            OperandKind::Slot,
            OperandKind::Immediate,
        ));
        if matches!(op.commutativity(), Commutativity::NonCommutative) {
            isa.push_op(CmpSelectOp::new(
                op,
                OperandKind::Immediate,
                OperandKind::Slot,
            ));
        }
    }
}

fn add_load_ops(isa: &mut Isa) {
    let ops = [
        // Generic
        LoadOpKind::Load32,
        LoadOpKind::Load64,
        // i32
        LoadOpKind::S32Load8,
        LoadOpKind::S32Load16,
        LoadOpKind::U32Load8,
        LoadOpKind::U32Load16,
        // i64
        LoadOpKind::S64Load8,
        LoadOpKind::S64Load16,
        LoadOpKind::S64Load32,
        LoadOpKind::U64Load8,
        LoadOpKind::U64Load16,
        LoadOpKind::U64Load32,
    ];
    for op in ops {
        isa.push_op(LoadOp::new(op, OperandKind::Slot, false, false));
        isa.push_op(LoadOp::new(op, OperandKind::Immediate, false, false));
        isa.push_op(LoadOp::new(op, OperandKind::Slot, true, true));
    }
}

fn add_store_ops(isa: &mut Isa) {
    let ops = [
        // Generic
        StoreOpKind::Store32,
        StoreOpKind::Store64,
        // i32
        StoreOpKind::I32Store8,
        StoreOpKind::I32Store16,
        // i64
        StoreOpKind::I64Store8,
        StoreOpKind::I64Store16,
        StoreOpKind::I64Store32,
    ];
    for op in ops {
        isa.push_op(StoreOp::new(
            op,
            OperandKind::Slot,
            OperandKind::Slot,
            false,
            false,
        ));
        isa.push_op(StoreOp::new(
            op,
            OperandKind::Slot,
            OperandKind::Immediate,
            false,
            false,
        ));
        isa.push_op(StoreOp::new(
            op,
            OperandKind::Immediate,
            OperandKind::Slot,
            false,
            false,
        ));
        isa.push_op(StoreOp::new(
            op,
            OperandKind::Slot,
            OperandKind::Slot,
            true,
            true,
        ));
        isa.push_op(StoreOp::new(
            op,
            OperandKind::Slot,
            OperandKind::Immediate,
            true,
            true,
        ));
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
            Ident::Return32,
            [Field::new(Ident::Value, FieldTy::U32)],
        )),
        Op::from(GenericOp::new(
            Ident::Return64,
            [Field::new(Ident::Value, FieldTy::U64)],
        )),
        Op::from(GenericOp::new(
            Ident::ReturnSpan,
            [Field::new(Ident::Fuel, FieldTy::BlockFuel)],
        )),
        Op::from(GenericOp::new(
            Ident::Branch,
            [Field::new(Ident::Values, FieldTy::SlotSpan)],
        )),
        Op::from(GenericOp::new(
            Ident::BranchTable,
            [
                Field::new(Ident::Index, FieldTy::Slot),
                Field::new(Ident::LenTargets, FieldTy::U16),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::BranchTableSpan,
            [
                Field::new(Ident::Index, FieldTy::Slot),
                Field::new(Ident::LenTargets, FieldTy::U16),
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
            Ident::Copy,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Value, FieldTy::Slot),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::Copy32,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Value, FieldTy::U32),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::Copy64,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Value, FieldTy::U64),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::CopySpan,
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
                Field::new(Ident::Results, FieldTy::SlotSpan),
                Field::new(Ident::Func, FieldTy::InternalFunc),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::CallImported,
            [
                Field::new(Ident::Results, FieldTy::SlotSpan),
                Field::new(Ident::Func, FieldTy::Func),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::CallIndirect,
            [
                Field::new(Ident::Results, FieldTy::SlotSpan),
                Field::new(Ident::Index, FieldTy::Slot),
                Field::new(Ident::FuncType, FieldTy::FuncType),
                Field::new(Ident::Table, FieldTy::Table),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::ReturnCallInternal,
            [Field::new(Ident::Func, FieldTy::InternalFunc)],
        )),
        Op::from(GenericOp::new(
            Ident::ReturnCallImported,
            [Field::new(Ident::Func, FieldTy::Func)],
        )),
        Op::from(GenericOp::new(
            Ident::ReturnCallIndirect,
            [
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
            Ident::GlobalGet,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Global, FieldTy::Global),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::GlobalSet,
            [
                Field::new(Ident::Global, FieldTy::Global),
                Field::new(Ident::Value, FieldTy::Slot),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::GlobalSet32,
            [
                Field::new(Ident::Global, FieldTy::Global),
                Field::new(Ident::Value, FieldTy::U32),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::GlobalSet64,
            [
                Field::new(Ident::Global, FieldTy::Global),
                Field::new(Ident::Value, FieldTy::U64),
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
            Ident::S64MulWide,
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
        Ident::Copy128,
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
    let kinds = [UnaryOpKind::V128Splat32, UnaryOpKind::V128Splat64];
    for kind in kinds {
        isa.push_op(UnaryOp::new(kind, OperandKind::Slot));
        isa.push_op(UnaryOp::new(kind, OperandKind::Immediate));
    }
}

fn add_simd_extract_lane_ops(isa: &mut Isa) {
    let ops = [
        Op::from(GenericOp::new(
            Ident::S8x16ExtractLane,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Value, FieldTy::Slot),
                Field::new(Ident::Lane, FieldTy::ImmLaneIdx16),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::U8x16ExtractLane,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Value, FieldTy::Slot),
                Field::new(Ident::Lane, FieldTy::ImmLaneIdx16),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::S16x8ExtractLane,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Value, FieldTy::Slot),
                Field::new(Ident::Lane, FieldTy::ImmLaneIdx8),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::U16x8ExtractLane,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Value, FieldTy::Slot),
                Field::new(Ident::Lane, FieldTy::ImmLaneIdx8),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::U32x4ExtractLane,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Value, FieldTy::Slot),
                Field::new(Ident::Lane, FieldTy::ImmLaneIdx4),
            ],
        )),
        Op::from(GenericOp::new(
            Ident::U64x2ExtractLane,
            [
                Field::new(Ident::Result, FieldTy::Slot),
                Field::new(Ident::Value, FieldTy::Slot),
                Field::new(Ident::Lane, FieldTy::ImmLaneIdx2),
            ],
        )),
    ];
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
    let kinds = [
        // Miscellaneous
        BinaryOpKind::I8x16Swizzle,
        // Integer Comparisons
        BinaryOpKind::I8x16Eq,
        BinaryOpKind::I8x16NotEq,
        BinaryOpKind::I16x8Eq,
        BinaryOpKind::I16x8NotEq,
        BinaryOpKind::I32x4Eq,
        BinaryOpKind::I32x4NotEq,
        BinaryOpKind::I64x2Eq,
        BinaryOpKind::I64x2NotEq,
        BinaryOpKind::S8x16Lt,
        BinaryOpKind::S8x16Le,
        BinaryOpKind::S16x8Lt,
        BinaryOpKind::S16x8Le,
        BinaryOpKind::S32x4Lt,
        BinaryOpKind::S32x4Le,
        BinaryOpKind::S64x2Lt,
        BinaryOpKind::S64x2Le,
        BinaryOpKind::U8x16Lt,
        BinaryOpKind::U8x16Le,
        BinaryOpKind::U16x8Lt,
        BinaryOpKind::U16x8Le,
        BinaryOpKind::U32x4Lt,
        BinaryOpKind::U32x4Le,
        BinaryOpKind::U64x2Lt,
        BinaryOpKind::U64x2Le,
        // Float Comparisons
        BinaryOpKind::F32x4Eq,
        BinaryOpKind::F32x4NotEq,
        BinaryOpKind::F32x4Lt,
        BinaryOpKind::F32x4Le,
        BinaryOpKind::F64x2Eq,
        BinaryOpKind::F64x2NotEq,
        BinaryOpKind::F64x2Lt,
        BinaryOpKind::F64x2Le,
        // Bitwise
        BinaryOpKind::V128And,
        BinaryOpKind::V128AndNot,
        BinaryOpKind::V128Or,
        BinaryOpKind::V128Xor,
        // i8x16 Ops
        BinaryOpKind::S8x16NarrowI16x8,
        BinaryOpKind::U8x16NarrowI16x8,
        BinaryOpKind::I8x16Add,
        BinaryOpKind::S8x16AddSat,
        BinaryOpKind::U8x16AddSat,
        BinaryOpKind::I8x16Sub,
        BinaryOpKind::S8x16SubSat,
        BinaryOpKind::U8x16SubSat,
        BinaryOpKind::S8x16Min,
        BinaryOpKind::U8x16Min,
        BinaryOpKind::S8x16Max,
        BinaryOpKind::U8x16Max,
        BinaryOpKind::U8x16Avgr,
        // i16x8 Ops
        BinaryOpKind::S16x8Q15MulrSat,
        BinaryOpKind::S16x8NarrowI32x4,
        BinaryOpKind::U16x8NarrowI32x4,
        BinaryOpKind::S16x8ExtmulLowI8x16,
        BinaryOpKind::U16x8ExtmulLowI8x16,
        BinaryOpKind::S16x8ExtmulHighI8x16,
        BinaryOpKind::U16x8ExtmulHighI8x16,
        BinaryOpKind::I16x8Add,
        BinaryOpKind::S16x8AddSat,
        BinaryOpKind::U16x8AddSat,
        BinaryOpKind::I16x8Sub,
        BinaryOpKind::S16x8SubSat,
        BinaryOpKind::U16x8SubSat,
        BinaryOpKind::I16x8Mul,
        BinaryOpKind::S16x8Min,
        BinaryOpKind::U16x8Min,
        BinaryOpKind::S16x8Max,
        BinaryOpKind::U16x8Max,
        BinaryOpKind::U16x8Avgr,
        // i32x4 Ops
        BinaryOpKind::I32x4Add,
        BinaryOpKind::I32x4Sub,
        BinaryOpKind::I32x4Mul,
        BinaryOpKind::S32x4Min,
        BinaryOpKind::U32x4Min,
        BinaryOpKind::S32x4Max,
        BinaryOpKind::U32x4Max,
        BinaryOpKind::S32x4DotI16x8,
        BinaryOpKind::S32x4ExtmulLowI16x8,
        BinaryOpKind::U32x4ExtmulLowI16x8,
        BinaryOpKind::S32x4ExtmulHighI16x8,
        BinaryOpKind::U32x4ExtmulHighI16x8,
        // i64x2 Ops
        BinaryOpKind::I64x2Add,
        BinaryOpKind::I64x2Sub,
        BinaryOpKind::I64x2Mul,
        BinaryOpKind::S64x2ExtmulLowI32x4,
        BinaryOpKind::U64x2ExtmulLowI32x4,
        BinaryOpKind::S64x2ExtmulHighI32x4,
        BinaryOpKind::U64x2ExtmulHighI32x4,
        // f32x4 Ops
        BinaryOpKind::F32x4Add,
        BinaryOpKind::F32x4Sub,
        BinaryOpKind::F32x4Mul,
        BinaryOpKind::F32x4Div,
        BinaryOpKind::F32x4Min,
        BinaryOpKind::F32x4Max,
        BinaryOpKind::F32x4Pmin,
        BinaryOpKind::F32x4Pmax,
        // f64x2 Ops
        BinaryOpKind::F64x2Add,
        BinaryOpKind::F64x2Sub,
        BinaryOpKind::F64x2Mul,
        BinaryOpKind::F64x2Div,
        BinaryOpKind::F64x2Min,
        BinaryOpKind::F64x2Max,
        BinaryOpKind::F64x2Pmin,
        BinaryOpKind::F64x2Pmax,
    ];
    for kind in kinds {
        isa.push_op(BinaryOp::new(kind, OperandKind::Slot, OperandKind::Slot));
    }
}

fn add_simd_shift_ops(isa: &mut Isa) {
    let kinds = [
        BinaryOpKind::I8x16Shl,
        BinaryOpKind::S8x16Shr,
        BinaryOpKind::U8x16Shr,
        BinaryOpKind::I16x8Shl,
        BinaryOpKind::S16x8Shr,
        BinaryOpKind::U16x8Shr,
        BinaryOpKind::I32x4Shl,
        BinaryOpKind::S32x4Shr,
        BinaryOpKind::U32x4Shr,
        BinaryOpKind::I64x2Shl,
        BinaryOpKind::S64x2Shr,
        BinaryOpKind::U64x2Shr,
    ];
    for kind in kinds {
        isa.push_op(BinaryOp::new(kind, OperandKind::Slot, OperandKind::Slot));
        isa.push_op(BinaryOp::new(
            kind,
            OperandKind::Slot,
            OperandKind::Immediate,
        ));
    }
}

fn add_simd_unary_ops(isa: &mut Isa) {
    let kinds = [
        // SIMD: Generic Unary Ops
        UnaryOpKind::V128Not,
        UnaryOpKind::V128AnyTrue,
        // SIMD: `i8x16` Unary Ops
        UnaryOpKind::I8x16Abs,
        UnaryOpKind::I8x16Neg,
        UnaryOpKind::I8x16Popcnt,
        UnaryOpKind::I8x16AllTrue,
        UnaryOpKind::I8x16Bitmask,
        // SIMD: `i16x8` Unary Ops
        UnaryOpKind::I16x8Abs,
        UnaryOpKind::I16x8Neg,
        UnaryOpKind::I16x8AllTrue,
        UnaryOpKind::I16x8Bitmask,
        UnaryOpKind::S16x8ExtaddPairwiseI8x16,
        UnaryOpKind::U16x8ExtaddPairwiseI8x16,
        UnaryOpKind::S16x8ExtendLowI8x16,
        UnaryOpKind::U16x8ExtendLowI8x16,
        UnaryOpKind::S16x8ExtendHighI8x16,
        UnaryOpKind::U16x8ExtendHighI8x16,
        // SIMD: `i32x4` Unary Ops
        UnaryOpKind::I32x4Abs,
        UnaryOpKind::I32x4Neg,
        UnaryOpKind::I32x4AllTrue,
        UnaryOpKind::I32x4Bitmask,
        UnaryOpKind::S32x4ExtaddPairwiseI16x8,
        UnaryOpKind::U32x4ExtaddPairwiseI16x8,
        UnaryOpKind::S32x4ExtendLowI16x8,
        UnaryOpKind::U32x4ExtendLowI16x8,
        UnaryOpKind::S32x4ExtendHighI16x8,
        UnaryOpKind::U32x4ExtendHighI16x8,
        // SIMD: `i64x2` Unary Ops
        UnaryOpKind::I64x2Abs,
        UnaryOpKind::I64x2Neg,
        UnaryOpKind::I64x2AllTrue,
        UnaryOpKind::I64x2Bitmask,
        UnaryOpKind::S64x2ExtendLowI32x4,
        UnaryOpKind::U64x2ExtendLowI32x4,
        UnaryOpKind::S64x2ExtendHighI32x4,
        UnaryOpKind::U64x2ExtendHighI32x4,
        // SIMD: `f32x4` Unary Ops
        UnaryOpKind::F32x4DemoteZeroF64x2,
        UnaryOpKind::F32x4Ceil,
        UnaryOpKind::F32x4Floor,
        UnaryOpKind::F32x4Trunc,
        UnaryOpKind::F32x4Nearest,
        UnaryOpKind::F32x4Abs,
        UnaryOpKind::F32x4Neg,
        UnaryOpKind::F32x4Sqrt,
        // SIMD: `f64x2` Unary Ops
        UnaryOpKind::F64x2PromoteLowF32x4,
        UnaryOpKind::F64x2Ceil,
        UnaryOpKind::F64x2Floor,
        UnaryOpKind::F64x2Trunc,
        UnaryOpKind::F64x2Nearest,
        UnaryOpKind::F64x2Abs,
        UnaryOpKind::F64x2Neg,
        UnaryOpKind::F64x2Sqrt,
        // SIMD: Conversions
        UnaryOpKind::S32x4TruncSatF32x4,
        UnaryOpKind::U32x4TruncSatF32x4,
        UnaryOpKind::S32x4TruncSatZeroF64x2,
        UnaryOpKind::U32x4TruncSatZeroF64x2,
        UnaryOpKind::F32x4ConvertS32x4,
        UnaryOpKind::F32x4ConvertU32x4,
        UnaryOpKind::F64x2ConvertLowS32x4,
        UnaryOpKind::F64x2ConvertLowU32x4,
    ];
    for kind in kinds {
        isa.push_op(UnaryOp::new(kind, OperandKind::Slot));
    }
}

fn add_simd_load_ops(isa: &mut Isa) {
    let ops = [
        LoadOpKind::V128Load,
        LoadOpKind::S16x8Load8x8,
        LoadOpKind::U16x8Load8x8,
        LoadOpKind::S32x4Load16x4,
        LoadOpKind::U32x4Load16x4,
        LoadOpKind::S64x2Load32x2,
        LoadOpKind::U64x2Load32x2,
        LoadOpKind::V128Load8Splat,
        LoadOpKind::V128Load16Splat,
        LoadOpKind::V128Load32Splat,
        LoadOpKind::V128Load64Splat,
        LoadOpKind::V128Load32Zero,
        LoadOpKind::V128Load64Zero,
    ];
    for op in ops {
        isa.push_op(LoadOp::new(op, OperandKind::Slot, false, false));
        isa.push_op(LoadOp::new(op, OperandKind::Slot, true, true));
    }
    let widths = [
        LaneWidth::W8,
        LaneWidth::W16,
        LaneWidth::W32,
        LaneWidth::W64,
    ];
    for width in widths {
        isa.push_op(V128LoadLaneOp::new(width, OperandKind::Slot, false, false));
        isa.push_op(V128LoadLaneOp::new(width, OperandKind::Slot, true, true));
    }
}

fn add_simd_store_ops(isa: &mut Isa) {
    let kinds = [
        StoreOpKind::Store128,
        StoreOpKind::V128Store8Lane,
        StoreOpKind::V128Store16Lane,
        StoreOpKind::V128Store32Lane,
        StoreOpKind::V128Store64Lane,
    ];
    for kind in kinds {
        isa.push_op(StoreOp::new(
            kind,
            OperandKind::Slot,
            OperandKind::Slot,
            false,
            false,
        ));
        isa.push_op(StoreOp::new(
            kind,
            OperandKind::Slot,
            OperandKind::Slot,
            true,
            true,
        ));
    }
}

fn add_relaxed_simd_ops(isa: &mut Isa) {
    let kinds = [
        BinaryOpKind::S16x8RelaxedDotI8x16I7x16,
        BinaryOpKind::S32x4RelaxedDotI8x16I7x16Add,
        BinaryOpKind::F32x4RelaxedMadd,
        BinaryOpKind::F32x4RelaxedNmadd,
        BinaryOpKind::F64x2RelaxedMadd,
        BinaryOpKind::F64x2RelaxedNmadd,
    ];
    for kind in kinds {
        isa.push_op(BinaryOp::new(kind, OperandKind::Slot, OperandKind::Slot));
    }
}
