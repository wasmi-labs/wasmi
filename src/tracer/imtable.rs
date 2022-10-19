use specs::{
    imtable::InitMemoryTableEntry,
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
            mmid: self.module_instance_index as u64,
            offset: self.offset as u64,
            vtype: self.vtype,
            value: self.value,
        }
    }
}

#[derive(Debug, Default)]
pub struct IMTable(pub Vec<IMEntry>);

impl IMTable {
    pub(crate) fn push(
        &mut self,
        is_global: bool,
        is_mutable: bool,
        module_instance_index: u16,
        offset: u32,
        vtype: VarType,
        value: u64,
    ) {
        self.0.push(IMEntry {
            is_mutable,
            is_global,
            module_instance_index,
            offset,
            vtype,
            value,
        })
    }
}
