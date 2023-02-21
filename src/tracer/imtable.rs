use specs::{
    imtable::{InitMemoryTable, InitMemoryTableEntry},
    mtable::{LocationType, VarType},
};

#[derive(Debug, Clone)]
pub struct IMEntry {
    pub is_global: bool,
    pub is_mutable: bool,
    pub module_instance_index: u16,
    pub offset: u32,
    pub vtype: VarType,
    pub value: u64,
}

impl Into<InitMemoryTableEntry> for IMEntry {
    fn into(self) -> InitMemoryTableEntry {
        InitMemoryTableEntry {
            is_mutable: self.is_mutable,
            ltype: if self.is_global {
                LocationType::Global
            } else {
                LocationType::Heap
            },
            offset: self.offset,
            vtype: self.vtype,
            value: self.value,
        }
    }
}

#[derive(Debug, Default)]
pub struct IMTable(Vec<InitMemoryTableEntry>);

impl IMTable {
    pub fn push(
        &mut self,
        is_global: bool,
        is_mutable: bool,
        offset: u32,
        vtype: VarType,
        value: u64,
    ) {
        self.0.push(InitMemoryTableEntry {
            is_mutable,
            ltype: if is_global {
                LocationType::Global
            } else {
                LocationType::Heap
            },
            offset,
            vtype,
            value,
        })
    }

    pub fn finalized(&self) -> InitMemoryTable {
        InitMemoryTable::new(self.0.clone())
    }
}
