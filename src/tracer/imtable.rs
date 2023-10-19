use specs::{
    imtable::{InitMemoryTable, InitMemoryTableEntry},
    mtable::{LocationType, VarType},
};

#[derive(Debug, Default)]
pub struct IMTable(Vec<InitMemoryTableEntry>);

impl IMTable {
    pub fn push(
        &mut self,
        is_global: bool,
        is_mutable: bool,
        start_offset: u32,
        end_offset: u32,
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
            start_offset,
            end_offset,
            vtype,
            value,
        })
    }

    pub fn finalized(&self, k: u32) -> InitMemoryTable {
        InitMemoryTable::new(self.0.clone(), k)
    }
}
