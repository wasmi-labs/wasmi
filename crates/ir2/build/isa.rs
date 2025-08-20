use crate::build::{
    op::{UnaryOp, UnaryOpKind},
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
