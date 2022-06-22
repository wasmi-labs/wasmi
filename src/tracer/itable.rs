use core::fmt::Debug;

#[derive(Debug)]
pub struct IEntry {
    pub module_instance_index: u32,
    pub func_index: u32,
    pub pc: u32,
    pub opcode: u32,
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
            module_instance_index,
            func_index,
            pc,
            opcode,
        })
    }
}
