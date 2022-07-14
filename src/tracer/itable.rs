use core::fmt::Debug;

use specs::itable::{InstructionTableEntry, Opcode};

#[derive(Debug, Clone)]
pub struct IEntry {
    pub module_instance_index: u16,
    pub func_index: u16,
    pub pc: u16,
    pub opcode: Opcode,
}

impl Into<InstructionTableEntry> for IEntry {
    fn into(self) -> InstructionTableEntry {
        InstructionTableEntry {
            moid: self.module_instance_index,
            mmid: self.module_instance_index,
            fid: self.func_index,
            bid: 0,
            iid: self.pc,
            opcode: self.opcode,
        }
    }
}

#[derive(Debug)]
pub struct ITable(pub Vec<IEntry>);

impl Default for ITable {
    fn default() -> Self {
        Self(vec![])
    }
}

impl ITable {
    pub(crate) fn push(
        &mut self,
        module_instance_index: u32,
        func_index: u32,
        pc: u32,
        opcode: Opcode,
    ) -> IEntry {
        let ientry = IEntry {
            module_instance_index: module_instance_index as u16,
            func_index: func_index as u16,
            pc: pc as u16,
            opcode,
        };

        self.0.push(ientry.clone());

        ientry
    }
}
