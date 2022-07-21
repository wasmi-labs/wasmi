use specs::jtable::JumpTableEntry;

use super::itable::IEntry;

#[derive(Debug, Clone)]
pub struct JEntry {
    eid: u64,
    last_jump_eid: u64,
    inst: IEntry,
}

#[derive(Debug)]
pub struct JTable(pub Vec<JEntry>);

impl JTable {
    pub fn new() -> Self {
        JTable(vec![])
    }

    pub fn push(&mut self, eid: u64, last_jump_eid: u64, inst: &IEntry) {
        self.0.push(JEntry {
            eid,
            last_jump_eid,
            inst: inst.clone(),
        })
    }
}

impl Into<JumpTableEntry> for JEntry {
    fn into(self) -> JumpTableEntry {
        JumpTableEntry {
            eid: self.eid,
            last_jump_eid: self.last_jump_eid,
            inst: Box::new(self.inst.into()),
        }
    }
}
