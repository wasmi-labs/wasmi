use parity_wasm::elements::ValueType;
use specs::{
    etable::{EventTable, EventTableBackend, EventTableEntry},
    mtable::{MemoryReadSize, MemoryStoreSize, VarType},
    step::StepInfo,
    TraceBackend,
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
    I32SignExtendI8 {
        value: i32,
    },
    I32SignExtendI16 {
        value: i32,
    },
    I64SignExtendI8 {
        value: i64,
    },
    I64SignExtendI16 {
        value: i64,
    },
    I64SignExtendI32 {
        value: i64,
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

pub struct ETable {
    pub(crate) eid: u32,
    slices: Vec<EventTableBackend>,
    entries: Vec<EventTableEntry>,
    capacity: u32,
    backend: TraceBackend,
}

impl ETable {
    pub(crate) fn new(capacity: u32, backend: TraceBackend) -> Self {
        Self {
            eid: 0,
            slices: Vec::default(),
            entries: Vec::with_capacity(capacity as usize),
            capacity,
            backend,
        }
    }

    fn flush(&mut self) {
        let empty = Vec::with_capacity(self.capacity as usize);
        let entries = std::mem::replace(&mut self.entries, empty);

        let event_table = match &self.backend {
            TraceBackend::File(path_builder) => {
                let path = path_builder(self.slices.len(), &EventTable::new(entries));

                EventTableBackend::Json(path)
            }
            TraceBackend::Memory => EventTableBackend::Memory(EventTable::new(entries)),
        };

        self.slices.push(event_table);
    }

    pub(crate) fn push(
        &mut self,
        fid: u32,
        iid: u32,
        sp: u32,
        allocated_memory_pages: u32,
        last_jump_eid: u32,
        step_info: StepInfo,
    ) {
        if self.entries.len() == self.capacity as usize {
            self.flush();
        }

        self.eid += 1;

        let sp = (DEFAULT_VALUE_STACK_LIMIT as u32)
            .checked_sub(sp)
            .unwrap()
            .checked_sub(1)
            .unwrap();

        let eentry = EventTableEntry {
            eid: self.eid,
            fid,
            iid,
            sp,
            allocated_memory_pages,
            last_jump_eid,
            step_info,
        };

        self.entries.push(eentry);
    }

    pub(crate) fn entries_mut(&mut self) -> &mut Vec<EventTableEntry> {
        &mut self.entries
    }

    pub fn finalized(mut self) -> Vec<EventTableBackend> {
        self.flush();

        self.slices
    }
}
