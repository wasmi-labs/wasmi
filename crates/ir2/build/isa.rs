use crate::build::Op;

pub struct Isa {
    ops: Vec<Op>,
}

impl Isa {
    pub fn push_op(&mut self, op: Op) {
        self.ops.push(op);
    }
}
