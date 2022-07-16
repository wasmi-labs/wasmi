use crate::{FuncRef, Module, ModuleRef};

use self::{
    etable::ETable,
    itable::{IEntry, ITable},
    jtable::JTable,
};

pub mod etable;
pub mod itable;
pub mod jtable;

#[derive(Debug)]
pub struct Tracer {
    pub itable: ITable,
    pub etable: ETable,
    pub jtable: Option<JTable>,
    module_instance_lookup: Vec<(ModuleRef, u16)>,
    function_lookup: Vec<(FuncRef, u32)>,
    last_jump_eid: Vec<u64>,
    function_index_allocator: u32,
}

impl Tracer {
    /// Create an empty tracer
    pub fn default() -> Self {
        Tracer {
            itable: ITable::default(),
            etable: ETable::default(),
            last_jump_eid: vec![0],
            jtable: None,
            module_instance_lookup: vec![],
            function_lookup: vec![],
            function_index_allocator: 1,
        }
    }

    pub fn push_frame(&mut self) {
        self.last_jump_eid.push(self.etable.get_latest_eid());
    }

    pub fn pop_frame(&mut self) {
        self.last_jump_eid.pop().unwrap();
    }

    pub fn next_module_id(&self) -> u16 {
        (self.module_instance_lookup.len() as u16) + 1
    }

    pub fn last_jump_eid(&self) -> u64 {
        *self.last_jump_eid.last().unwrap()
    }

    pub fn eid(&self) -> u64 {
        self.etable.get_latest_eid()
    }

    fn allocate_func_index(&mut self) -> u32 {
        let r = self.function_index_allocator;
        self.function_index_allocator = r + 1;
        r
    }
}

impl Tracer {
    pub fn register_module_instance(&mut self, module: &Module, module_instance: &ModuleRef) {
        let mut func_index = 0;

        let start_fn_idx = module.module().start_section();

        loop {
            if let Some(func) = module_instance.func_by_index(func_index) {
                let func_index_in_itable = if Some(func_index) == start_fn_idx {
                    0
                } else {
                    self.allocate_func_index()
                };
                let body = func.body().expect("Host function is not allowed");
                let code = &body.code;
                let mut iter = code.iterate_from(0);
                loop {
                    let pc = iter.position();
                    if let Some(instruction) = iter.next() {
                        let ientry = self.itable.push(
                            self.next_module_id() as u32,
                            func_index_in_itable,
                            pc,
                            instruction.into(),
                        );

                        if self.jtable.is_none() {
                            self.jtable = Some(JTable::new(&ientry))
                        }
                    } else {
                        break;
                    }
                }

                self.function_lookup
                    .push((func.clone(), func_index_in_itable));
                func_index = func_index + 1;
            } else {
                break;
            }
        }

        self.module_instance_lookup
            .push((module_instance.clone(), self.next_module_id()));
    }

    pub fn lookup_module_instance(&self, module_instance: &ModuleRef) -> u16 {
        for m in &self.module_instance_lookup {
            if &m.0 == module_instance {
                return m.1;
            }
        }

        unreachable!()
    }

    pub fn lookup_function(&self, function: &FuncRef) -> u32 {
        let pos = self
            .function_lookup
            .iter()
            .position(|m| m.0 == *function)
            .unwrap();
        self.function_lookup.get(pos).unwrap().1
    }

    pub fn lookup_first_inst(&self, function: &FuncRef) -> IEntry {
        let function_idx = self.lookup_function(function);

        for ientry in &self.itable.0 {
            if ientry.func_index as u32 == function_idx {
                return ientry.clone();
            }
        }

        unreachable!();
    }
}
