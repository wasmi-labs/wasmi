use specs::types::FunctionType;

use crate::Signature;

#[derive(Debug)]
pub struct FuncDesc {
    pub ftype: FunctionType,
    pub signature: Signature,
}

#[derive(Debug, Default)]
pub struct Observer {
    pub counter: usize,
    pub is_in_phantom: bool,
}
