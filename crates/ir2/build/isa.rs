use crate::build::{
    op::{
        BinaryOp,
        BinaryOpKind,
        CmpBranchOp,
        CmpOpKind,
        CmpSelectOp,
        Commutativity,
        Input,
        LoadOp,
        LoadOpKind,
        StoreOp,
        StoreOpKind,
        UnaryOp,
        UnaryOpKind,
    },
    Op,
};

#[derive(Default)]
pub struct Isa {
    pub ops: Vec<Op>,
}

impl Isa {
    fn push_op(&mut self, op: Op) {
        self.ops.push(op);
    }
}

pub fn wasmi_isa() -> Isa {
    let mut isa = Isa::default();
    add_unary_ops(&mut isa);
    add_binary_ops(&mut isa);
    add_cmp_branch_ops(&mut isa);
    add_cmp_select_ops(&mut isa);
    add_load_ops(&mut isa);
    add_store_ops(&mut isa);
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
    ];
    for op in ops {
        isa.push_op(Op::Unary(UnaryOp::new(op)));
    }
}

fn add_binary_ops(isa: &mut Isa) {
    let ops = [
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
        isa.push_op(Op::Binary(BinaryOp::new(op, Input::Stack, Input::Stack)));
        isa.push_op(Op::Binary(BinaryOp::new(
            op,
            Input::Stack,
            Input::Immediate,
        )));
        if matches!(op.commutativity(), Commutativity::NonCommutative) {
            isa.push_op(Op::Binary(BinaryOp::new(
                op,
                Input::Immediate,
                Input::Stack,
            )));
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
        isa.push_op(Op::CmpBranch(CmpBranchOp::new(
            op,
            Input::Stack,
            Input::Stack,
        )));
        isa.push_op(Op::CmpBranch(CmpBranchOp::new(
            op,
            Input::Stack,
            Input::Immediate,
        )));
        if matches!(op.commutativity(), Commutativity::NonCommutative) {
            isa.push_op(Op::CmpBranch(CmpBranchOp::new(
                op,
                Input::Immediate,
                Input::Stack,
            )));
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
        isa.push_op(Op::CmpSelect(CmpSelectOp::new(
            op,
            Input::Stack,
            Input::Stack,
        )));
        isa.push_op(Op::CmpSelect(CmpSelectOp::new(
            op,
            Input::Stack,
            Input::Immediate,
        )));
        if matches!(op.commutativity(), Commutativity::NonCommutative) {
            isa.push_op(Op::CmpSelect(CmpSelectOp::new(
                op,
                Input::Immediate,
                Input::Stack,
            )));
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
        isa.push_op(Op::Load(LoadOp::new(op, Input::Stack, false, false)));
        isa.push_op(Op::Load(LoadOp::new(op, Input::Immediate, false, false)));
        isa.push_op(Op::Load(LoadOp::new(op, Input::Stack, true, true)));
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
        isa.push_op(Op::Store(StoreOp::new(
            op,
            Input::Stack,
            Input::Stack,
            false,
            false,
        )));
        isa.push_op(Op::Store(StoreOp::new(
            op,
            Input::Stack,
            Input::Immediate,
            false,
            false,
        )));
        isa.push_op(Op::Store(StoreOp::new(
            op,
            Input::Immediate,
            Input::Stack,
            false,
            false,
        )));
        isa.push_op(Op::Store(StoreOp::new(
            op,
            Input::Stack,
            Input::Stack,
            true,
            true,
        )));
        isa.push_op(Op::Store(StoreOp::new(
            op,
            Input::Stack,
            Input::Immediate,
            true,
            true,
        )));
    }
}
