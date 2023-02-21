use parity_wasm::elements::ValueType;
use specs::{
    etable::{EventTable, EventTableEntry},
    itable::InstructionTableEntry,
    mtable::{MemoryReadSize, MemoryStoreSize, VarType},
    step::StepInfo,
};

use crate::{runner::ValueInternal, DEFAULT_VALUE_STACK_LIMIT};

pub enum RunInstructionTracePre {
    BrIfEqz {
        value: i32,
    },
    BrIfNez {
        value: i32,
    },
    BrTable {
        index: i32,
    },

    Call {
        args: Vec<ValueInternal>,
    },
    CallIndirect {
        table_idx: u32,
        type_idx: u32,
        offset: u32,
    },

    SetLocal {
        depth: u32,
        value: ValueInternal,
        vtype: ValueType,
    },
    SetGlobal {
        idx: u32,
        value: ValueInternal,
    },

    Load {
        offset: u32,
        raw_address: u32,
        effective_address: Option<u32>, // use option in case of memory out of bound
        vtype: ValueType,
        load_size: MemoryReadSize,
    },
    Store {
        offset: u32,
        raw_address: u32,
        effective_address: Option<u32>,
        value: u64,
        vtype: ValueType,
        store_size: MemoryStoreSize,
        pre_block_value1: Option<u64>,
        pre_block_value2: Option<u64>,
    },

    GrowMemory(i32),

    I32BinOp {
        left: i32,
        right: i32,
    },
    I32BinShiftOp {
        left: u64,
        right: u64,
    },

    I64BinOp {
        left: i64,
        right: i64,
    },

    I32Single(i32),
    I32Comp {
        left: i32,
        right: i32,
    },
    I64Single(i64),
    I64Comp {
        left: i64,
        right: i64,
    },

    I32WrapI64 {
        value: i64,
    },
    I64ExtendI32 {
        value: i32,
        sign: bool,
    },

    UnaryOp {
        operand: u64,
        vtype: VarType,
    },

    Drop,
    Select {
        val1: u64,
        val2: u64,
        cond: u64,
    },
}

pub(crate) trait ETable {
    fn get_latest_eid(&self) -> u32;

    fn get_last_entry_mut(&mut self) -> Option<&mut EventTableEntry>;

    fn push(
        &mut self,
        inst: InstructionTableEntry,
        sp: u32,
        allocated_memory_pages: u32,
        last_jump_eid: u32,
        step_info: StepInfo,
    ) -> EventTableEntry;
}

impl ETable for EventTable {
    fn get_latest_eid(&self) -> u32 {
        self.entries().last().unwrap().eid
    }

    fn get_last_entry_mut(&mut self) -> Option<&mut EventTableEntry> {
        self.entries_mut().last_mut()
    }

    fn push(
        &mut self,
        inst: InstructionTableEntry,
        sp: u32,
        allocated_memory_pages: u32,
        last_jump_eid: u32,
        step_info: StepInfo,
    ) -> EventTableEntry {
        let sp = (DEFAULT_VALUE_STACK_LIMIT as u32)
            .checked_sub(sp)
            .unwrap()
            .checked_sub(1)
            .unwrap();

        let eentry = EventTableEntry {
            eid: (self.entries().len() + 1).try_into().unwrap(),
            sp,
            allocated_memory_pages,
            last_jump_eid,
            inst,
            step_info,
        };

        self.entries_mut().push(eentry.clone());

        eentry
    }
}
