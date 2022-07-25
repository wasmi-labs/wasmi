use specs::imtable::InitMemoryTableEntry;

#[derive(Debug, Clone)]
pub struct IMEntry {
    pub module_instance_index: u16,
    pub offset: u32,
    pub value: u64,
}

impl Into<InitMemoryTableEntry> for IMEntry {
    fn into(self) -> InitMemoryTableEntry {
        InitMemoryTableEntry {
            mmid: self.module_instance_index as u64,
            offset: self.offset as u64,
            value: self.value,
        }
    }
}

#[derive(Debug, Default)]
pub struct IMTable(pub Vec<IMEntry>);

impl IMTable {
    pub(crate) fn push(&mut self, module_instance_index: u16, offset: u32, value: u64) {
        self.0.push(IMEntry {
            module_instance_index,
            offset,
            value,
        })
    }
}
