use std::collections::HashMap;

use specs::{
    brtable::{ElemEntry, ElemTable},
    configure_table::ConfigureTable,
    etable::EventTable,
    host_function::HostFunctionDesc,
    itable::{InstructionTable, InstructionTableEntry},
    jtable::{JumpTable, StaticFrameEntry},
    mtable::VarType,
    types::FunctionType,
};

use crate::{
    runner::{from_value_internal_to_u64_with_typ, ValueInternal},
    FuncRef,
    GlobalRef,
    MemoryRef,
    Module,
    ModuleRef,
    Signature,
};

use self::{etable::ETable, imtable::IMTable};

pub mod etable;
pub mod imtable;

#[derive(Debug)]
pub struct FuncDesc {
    pub index_within_jtable: u32,
    pub ftype: FunctionType,
    pub signature: Signature,
}

#[derive(Debug)]
pub struct Tracer {
    pub itable: InstructionTable,
    pub imtable: IMTable,
    pub etable: EventTable,
    pub jtable: JumpTable,
    pub elem_table: ElemTable,
    pub configure_table: ConfigureTable,
    type_of_func_ref: Vec<(FuncRef, u32)>,
    function_lookup: Vec<(FuncRef, u32)>,
    pub(crate) last_jump_eid: Vec<u32>,
    function_index_allocator: u32,
    pub(crate) function_index_translation: HashMap<u32, FuncDesc>,
    pub host_function_index_lookup: HashMap<usize, HostFunctionDesc>,
    pub static_jtable_entries: Vec<StaticFrameEntry>,
}

impl Tracer {
    /// Create an empty tracer
    pub fn new(host_plugin_lookup: HashMap<usize, HostFunctionDesc>) -> Self {
        Tracer {
            itable: InstructionTable::default(),
            imtable: IMTable::default(),
            etable: EventTable::default(),
            last_jump_eid: vec![],
            jtable: JumpTable::default(),
            elem_table: ElemTable::default(),
            configure_table: ConfigureTable::default(),
            type_of_func_ref: vec![],
            function_lookup: vec![],
            function_index_allocator: 1,
            function_index_translation: Default::default(),
            host_function_index_lookup: host_plugin_lookup,
            static_jtable_entries: vec![],
        }
    }

    pub fn push_frame(&mut self) {
        self.last_jump_eid.push(self.etable.get_latest_eid());
    }

    pub fn pop_frame(&mut self) {
        self.last_jump_eid.pop().unwrap();
    }

    pub fn last_jump_eid(&self) -> u32 {
        *self.last_jump_eid.last().unwrap()
    }

    pub fn eid(&self) -> u32 {
        self.etable.get_latest_eid()
    }

    fn allocate_func_index(&mut self) -> u32 {
        let r = self.function_index_allocator;
        self.function_index_allocator = r + 1;
        r
    }

    fn lookup_host_plugin(&self, function_index: usize) -> HostFunctionDesc {
        self.host_function_index_lookup
            .get(&function_index)
            .unwrap()
            .clone()
    }
}

impl Tracer {
    pub(crate) fn push_init_memory(&mut self, memref: MemoryRef) {
        let pages = (*memref).limits().initial();
        // one page contains 64KB*1024/8=8192 u64 entries
        for i in 0..(pages * 8192) {
            let mut buf = [0u8; 8];
            (*memref).get_into(i * 8, &mut buf).unwrap();
            self.imtable
                .push(false, true, i, VarType::I64, u64::from_le_bytes(buf));
        }
    }

    pub(crate) fn push_global(&mut self, globalidx: u32, globalref: &GlobalRef) {
        let vtype = globalref.elements_value_type().into();

        self.imtable.push(
            true,
            globalref.is_mutable(),
            globalidx,
            vtype,
            from_value_internal_to_u64_with_typ(vtype, ValueInternal::from(globalref.get())),
        );
    }

    pub(crate) fn push_elem(&mut self, table_idx: u32, offset: u32, func_idx: u32, type_idx: u32) {
        self.elem_table.insert(ElemEntry {
            table_idx,
            type_idx,
            offset,
            func_idx,
        })
    }

    pub(crate) fn push_type_of_func_ref(&mut self, func: FuncRef, type_idx: u32) {
        self.type_of_func_ref.push((func, type_idx))
    }

    #[allow(dead_code)]
    pub(crate) fn statistics_instructions<'a>(&mut self, module_instance: &ModuleRef) {
        let mut func_index = 0;
        let mut insts = vec![];

        loop {
            if let Some(func) = module_instance.func_by_index(func_index) {
                let body = func.body().unwrap();

                let code = &body.code.vec;

                for inst in code {
                    if insts.iter().position(|i| i == inst).is_none() {
                        insts.push(inst.clone())
                    }
                }
            } else {
                break;
            }

            func_index = func_index + 1;
        }

        for inst in insts {
            println!("{:?}", inst);
        }
    }

    pub(crate) fn lookup_type_of_func_ref(&self, func_ref: &FuncRef) -> u32 {
        self.type_of_func_ref
            .iter()
            .find(|&f| f.0 == *func_ref)
            .unwrap()
            .1
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
                        } => {
                            let plugin_desc = self.lookup_host_plugin(host_func_index);

                            match plugin_desc {
                                HostFunctionDesc::Internal {
                                    name,
                                    op_index_in_plugin,
                                    plugin,
                                } => FunctionType::HostFunction {
                                    plugin,
                                    function_index: host_func_index,
                                    function_name: name,
                                    op_index_in_plugin,
                                },
                                HostFunctionDesc::External { name, op, sig } => {
                                    FunctionType::HostFunctionExternal {
                                        function_name: name,
                                        op,
                                        sig,
                                    }
                                }
                            }
                        }
                    };

                    self.function_lookup
                        .push((func.clone(), func_index_in_itable));
                    self.function_index_translation.insert(
                        func_index,
                        FuncDesc {
                            index_within_jtable: func_index_in_itable,
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
    }

    pub fn lookup_function(&self, function: &FuncRef) -> u32 {
        let pos = self
            .function_lookup
            .iter()
            .position(|m| m.0 == *function)
            .unwrap();
        self.function_lookup.get(pos).unwrap().1
    }

    pub fn lookup_ientry(&self, function: &FuncRef, pos: u32) -> InstructionTableEntry {
        let function_idx = self.lookup_function(function);

        for ientry in self.itable.entries() {
            if ientry.fid == function_idx && ientry.iid as u32 == pos {
                return ientry.clone();
            }
        }

        unreachable!()
    }

    pub fn lookup_first_inst(&self, function: &FuncRef) -> InstructionTableEntry {
        let function_idx = self.lookup_function(function);

        for ientry in self.itable.entries() {
            if ientry.fid == function_idx {
                return ientry.clone();
            }
        }

        unreachable!();
    }
}
