use parity_wasm::elements::ValueType;
use specs::{etable::EventTableEntry, itable::Opcode, step::StepInfo};

use crate::{runner::ValueInternal, DEFAULT_VALUE_STACK_LIMIT};

use super::itable::IEntry;

pub enum RunInstructionTracePre {
    BrIfNez {
        value: i32,
    },

    GetLocal {
        depth: u32,
        value: ValueInternal,
        vtype: ValueType,
    },

    I32BinOp {
        left: i32,
        right: i32,
    },

    I32Comp {
        left: i32,
        right: i32,
    },

    Drop {
        value: u64,
    },
}

#[derive(Debug, Clone)]
pub struct EEntry {
    pub id: u64,
    pub sp: u64,
    pub last_jump_eid: u64,
    pub inst: IEntry,
    pub step: StepInfo,
}

impl Into<EventTableEntry> for EEntry {
    fn into(self) -> EventTableEntry {
        EventTableEntry {
            eid: self.id,
            sp: (DEFAULT_VALUE_STACK_LIMIT as u64)
                .checked_sub(self.sp)
                .unwrap()
                .checked_sub(1)
                .unwrap(),
            last_jump_eid: self.last_jump_eid,
            inst: self.inst.into(),
            step_info: self.step.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ETable {
    eid: u64,
    entries: Vec<EEntry>,
}

impl Default for ETable {
    fn default() -> Self {
        Self {
            eid: 1,
            entries: vec![],
        }
    }
}

impl ETable {
    pub fn get_latest_eid(&self) -> u64 {
        self.entries.last().unwrap().id
    }

    pub fn get_entries(&self) -> &Vec<EEntry> {
        &self.entries
    }

    fn allocate_eid(&mut self) -> u64 {
        let r = self.eid;
        self.eid = r + 1;
        return r;
    }

    pub fn push(
        &mut self,
        module_instance_index: u16,
        func_index: u32,
        sp: u64,
        pc: u32,
        last_jump_eid: u64,
        opcode: Opcode,
        step: StepInfo,
    ) -> EEntry {
        let eentry = EEntry {
            id: self.allocate_eid(),
            sp,
            last_jump_eid,
            inst: IEntry {
                module_instance_index: module_instance_index as u16,
                func_index: func_index as u16,
                pc: pc as u16,
                opcode,
            },
            step,
        };

        self.entries.push(eentry.clone());

        eentry
    }
}
