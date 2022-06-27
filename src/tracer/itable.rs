use core::fmt::Debug;

#[derive(Debug)]
pub struct IEntry {
    pub module_instance_index: u16,
    pub func_index: u16,
    pub pc: u16,
    pub opcode: u64,
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
        opcode: u32,
    ) {
        self.0.push(IEntry {
            module_instance_index: module_instance_index as u16,
            func_index: func_index as u16,
            pc: pc as u16,
            opcode: opcode as u64,
        })
    }
}
