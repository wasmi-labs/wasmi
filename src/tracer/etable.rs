use parity_wasm::elements::ValueType;
use specs::{etable::EventTableEntry, itable::Opcode, step::StepInfo};

use crate::runner::ValueInternal;

use super::itable::IEntry;

pub enum RunInstructionTracePre {
    BrIfNez { value: i32 },

    GetLocal { depth: u32, value: ValueInternal, vtype: ValueType },

    I32BinOp { left: i32, right: i32 },

    I32Comp { left: i32, right: i32 },

    Drop { value: u64 },
}

#[derive(Debug, Clone)]
pub struct EEntry {
    pub id: u64,
    pub sp: u64,
    pub inst: IEntry,
    pub step: StepInfo,
}

impl Into<EventTableEntry> for EEntry {
    fn into(self) -> EventTableEntry {
        EventTableEntry {
            eid: self.id,
            sp: self.sp,
            // FIXME: fill with correct value
            last_jump_eid: 0,
            inst: self.inst.into(),
            step_info: self.step.clone(),
        }
    }
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
        opcode: Opcode,
        step: StepInfo,
    ) {
        self.0.push(EEntry {
            id: self.0.len() as u64,
            sp,
            inst: IEntry {
                module_instance_index: module_instance_index as u16,
                func_index: func_index as u16,
                pc: pc as u16,
                opcode,
            },
            step,
        })
    }
}
