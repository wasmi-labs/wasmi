use std::collections::HashMap;

use specs::types::FunctionType;

use crate::{FuncRef, MemoryRef, Module, ModuleRef, Signature};

use self::{
    etable::ETable,
    imtable::IMTable,
    itable::{IEntry, ITable},
    jtable::JTable,
};

pub mod etable;
pub mod imtable;
pub mod itable;
pub mod jtable;

#[derive(Debug)]
pub(crate) struct FuncDesc {
    pub(crate) index_within_jtable: u16,
    pub(crate) ftype: FunctionType,
    pub(crate) signature: Signature,
}

#[derive(Debug)]
pub struct Tracer {
    pub itable: ITable,
    pub imtable: IMTable,
    pub etable: ETable,
    pub jtable: JTable,
    module_instance_lookup: Vec<(ModuleRef, u16)>,
    memory_instance_lookup: Vec<(MemoryRef, u16)>,
    function_lookup: Vec<(FuncRef, u16)>,
    last_jump_eid: Vec<u64>,
    function_index_allocator: u32,
    pub(crate) function_index_translation: HashMap<u32, FuncDesc>,
}

impl Tracer {
    /// Create an empty tracer
    pub fn default() -> Self {
        Tracer {
            itable: ITable::default(),
            imtable: IMTable::default(),
            etable: ETable::default(),
            last_jump_eid: vec![0],
            jtable: JTable::default(),
            module_instance_lookup: vec![],
            memory_instance_lookup: vec![],
            function_lookup: vec![],
            function_index_allocator: 1,
            function_index_translation: Default::default(),
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

    pub fn next_memory_id(&self) -> u16 {
        (self.memory_instance_lookup.len() as u16) + 1
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
    pub(crate) fn push_init_memory(&mut self, memref: MemoryRef) {
        let pages = (*memref).limits().initial();
        for i in 0..(pages * 1024) {
            let mut buf = [0u8; 8];
            (*memref).get_into(i * 8, &mut buf).unwrap();
            self.imtable
                .push(self.next_memory_id(), i, u64::from_le_bytes(buf));
        }

        self.memory_instance_lookup
            .push((memref, self.next_memory_id()));
    }

    pub(crate) fn register_module_instance(
        &mut self,
        module: &Module,
        module_instance: &ModuleRef,
    ) {
        let start_fn_idx = module.module().start_section();

        {
            let mut func_index = 0;

            loop {
                if let Some(func) = module_instance.func_by_index(func_index) {
                    let func_index_in_itable = if Some(func_index) == start_fn_idx {
                        0
                    } else {
                        self.allocate_func_index()
                    };

                    let ftype = match *func.as_internal() {
                        crate::func::FuncInstanceInternal::Internal { .. } => {
                            FunctionType::WasmFunction
                        }
                        crate::func::FuncInstanceInternal::Host {
                            host_func_index, ..
                        } => FunctionType::HostFunction(host_func_index),
                    };

                    self.function_lookup
                        .push((func.clone(), func_index_in_itable as u16));
                    self.function_index_translation.insert(
                        func_index,
                        FuncDesc {
                            index_within_jtable: func_index_in_itable as u16,
                            ftype,
                            signature: func.signature().clone(),
                        },
                    );
                    func_index = func_index + 1;
                } else {
                    break;
                }
            }
        }

        {
            let mut func_index = 0;

            loop {
                if let Some(func) = module_instance.func_by_index(func_index) {
                    let funcdesc = self.function_index_translation.get(&func_index).unwrap();

                    if let Some(body) = func.body() {
                        let code = &body.code;
                        let mut iter = code.iterate_from(0);
                        loop {
                            let pc = iter.position();
                            if let Some(instruction) = iter.next() {
                                let _ = self.itable.push(
                                    self.next_module_id() as u32,
                                    funcdesc.index_within_jtable,
                                    pc,
                                    instruction.into(&self.function_index_translation),
                                );
                            } else {
                                break;
                            }
                        }
                    }

                    func_index = func_index + 1;
                } else {
                    break;
                }
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

    pub fn lookup_memory_instance(&self, module_instance: &MemoryRef) -> u16 {
        for m in &self.memory_instance_lookup {
            if &m.0 == module_instance {
                return m.1;
            }
        }

        unreachable!()
    }

    pub fn lookup_function(&self, function: &FuncRef) -> u16 {
        let pos = self
            .function_lookup
            .iter()
            .position(|m| m.0 == *function)
            .unwrap();
        self.function_lookup.get(pos).unwrap().1
    }

    pub fn lookup_ientry(&self, function: &FuncRef, pos: u32) -> IEntry {
        let function_idx = self.lookup_function(function);

        for ientry in &self.itable.0 {
            if ientry.func_index as u16 == function_idx && ientry.pc as u32 == pos {
                return ientry.clone();
            }
        }

        unreachable!()
    }

    pub fn lookup_first_inst(&self, function: &FuncRef) -> IEntry {
        let function_idx = self.lookup_function(function);

        for ientry in &self.itable.0 {
            if ientry.func_index as u16 == function_idx {
                return ientry.clone();
            }
        }

        unreachable!();
    }
}
