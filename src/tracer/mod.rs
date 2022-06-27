use crate::{FuncRef, ModuleRef};

use self::{etable::ETable, itable::ITable};

pub mod etable;
pub mod itable;

#[derive(Debug)]
pub struct Tracer {
    pub itable: ITable,
    pub etable: ETable,
    pub(crate) module_instance_lookup: Vec<ModuleRef>,
    pub(crate) function_lookup: Vec<(FuncRef, u32)>,
}

impl Tracer {
    /// Create an empty tracer
    pub fn default() -> Self {
        Tracer {
            itable: ITable::default(),
            etable: ETable::default(),
            module_instance_lookup: vec![],
            function_lookup: vec![],
        }
    }
}

impl Tracer {
    pub fn register_module_instance(&mut self, module_instance: &ModuleRef) {
        let mut func_index = 0;

        loop {
            if let Some(func) = module_instance.func_by_index(func_index) {
                let body = func.body().expect("Host function is not allowed");
                let code = &body.code;
                let mut iter = code.iterate_from(0);
                loop {
                    let pc = iter.position();
                    if let Some(instruction) = iter.next() {
                        self.itable.push(
                            self.module_instance_lookup.len() as u32,
                            func_index,
                            pc,
                            instruction.into(),
                        )
                    } else {
                        break;
                    }
                }

                func_index = func_index + 1;
                self.function_lookup.push((func.clone(), func_index))
            } else {
                break;
            }
        }

        self.module_instance_lookup.push(module_instance.clone());
    }

    pub fn lookup_module_instance(&self, module_instance: &ModuleRef) -> u32 {
        self.module_instance_lookup
            .iter()
            .position(|m| m == module_instance)
            .unwrap() as u32
    }

    pub fn lookup_function(&self, function: &FuncRef) -> u32 {
        let pos = self
            .function_lookup
            .iter()
            .position(|m| m.0 == *function)
            .unwrap();
        self.function_lookup.get(pos).unwrap().1
    }
}
