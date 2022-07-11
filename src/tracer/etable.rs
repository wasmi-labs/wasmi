use crate::runner::ValueInternal;

use super::itable::IEntry;

pub enum RunInstructionTracePre {
    BrIfNez { value: i32 },

    GetLocal { depth: u32, value: ValueInternal },

    I32BinOp { left: i32, right: i32 },

    I32Comp { left: i32, right: i32 },
}

#[derive(Debug, Clone)]
pub enum RunInstructionTraceStep {
    BrIfNez {
        value: i32,
        dst_pc: u32,
    },
    Return {
        drop: u32,
        keep: u32,
        drop_values: Vec<ValueInternal>,
        keep_values: Vec<ValueInternal>,
    },

    Call {
        index: u32,
    },

    GetLocal {
        depth: u32,
        value: ValueInternal,
    },

    I32Const {
        value: i32,
    },

    I32BinOp {
        left: i32,
        right: i32,
        value: i32,
    },

    I32Comp {
        left: i32,
        right: i32,
        value: bool,
    },
}

#[derive(Debug, Clone)]
pub struct EEntry {
    pub id: u64,
    pub sp: u64,
    pub inst: IEntry,
    pub step: RunInstructionTraceStep,
}

#[derive(Debug)]
pub struct ETable(pub Vec<EEntry>);

impl Default for ETable {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl ETable {
    pub fn push(
        &mut self,
        module_instance_index: u32,
        func_index: u32,
        sp: u64,
        pc: u32,
        opcode: u32,
        step: RunInstructionTraceStep,
    ) {
        self.0.push(EEntry {
            id: self.0.len() as u64,
            sp,
            inst: IEntry {
                module_instance_index: module_instance_index as u16,
                func_index: func_index as u16,
                pc: pc as u16,
                opcode: opcode as u64,
            },
            step,
        })
    }
}
