use std::collections::HashMap;

use regex::Regex;
use specs::{
    brtable::{ElemEntry, ElemTable},
    configure_table::ConfigureTable,
    etable::EventTable,
    host_function::HostFunctionDesc,
    itable::InstructionTableInternal,
    jtable::{JumpTable, StaticFrameEntry},
    mtable::VarType,
    types::FunctionType,
};

use crate::{
    runner::{from_value_internal_to_u64_with_typ, ValueInternal},
    FuncRef,
    GlobalRef,
    MemoryRef,
    ModuleRef,
    Signature,
};

use self::{etable::ETable, imtable::IMTable, phantom::PhantomFunction};

pub mod etable;
pub mod imtable;
pub mod phantom;

#[derive(Debug)]
pub struct FuncDesc {
    pub ftype: FunctionType,
    pub signature: Signature,
}

#[derive(Debug)]
pub struct Observer {
    pub counter: usize,
    pub is_in_phantom: bool,
}

#[derive(Debug)]
pub struct Tracer {
    pub itable: InstructionTableInternal,
    pub imtable: IMTable,
    pub etable: EventTable,
    pub jtable: JumpTable,
    pub elem_table: ElemTable,
    pub configure_table: ConfigureTable,
    type_of_func_ref: Vec<(FuncRef, u32)>,
    function_lookup: HashMap<FuncRef, u32>,
    pub(crate) function_lookup_name: HashMap<u32, String>,
    pub(crate) last_jump_eid: Vec<u32>,
    pub(crate) function_desc: HashMap<u32, FuncDesc>,
    pub host_function_index_lookup: HashMap<usize, HostFunctionDesc>,
    pub static_jtable_entries: Vec<StaticFrameEntry>,
    pub phantom_functions: Vec<String>,
    pub phantom_functions_ref: Vec<FuncRef>,
    // Wasm Image Function Idx
    pub wasm_input_func_idx: Option<u32>,
    pub wasm_input_func_ref: Option<FuncRef>,
    dry_run: bool,

    pub observer: Observer,
}

impl Tracer {
    /// Create an empty tracer
    pub fn new(
        host_plugin_lookup: HashMap<usize, HostFunctionDesc>,
        phantom_functions: &Vec<String>,
        dry_run: bool,
    ) -> Self {
        Tracer {
            itable: InstructionTableInternal::default(),
            imtable: IMTable::default(),
            etable: EventTable::default(),
            last_jump_eid: vec![],
            jtable: JumpTable::default(),
            elem_table: ElemTable::default(),
            configure_table: ConfigureTable::default(),
            type_of_func_ref: vec![],
            function_lookup: HashMap::default(),
            function_desc: Default::default(),
            function_lookup_name: Default::default(),
            host_function_index_lookup: host_plugin_lookup,
            static_jtable_entries: vec![],
            phantom_functions: phantom_functions.clone(),
            phantom_functions_ref: vec![],
            wasm_input_func_ref: None,
            wasm_input_func_idx: None,
            dry_run,

            observer: Observer {
                counter: 0,
                is_in_phantom: false,
            },
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

    fn lookup_host_plugin(&self, function_index: usize) -> HostFunctionDesc {
        self.host_function_index_lookup
            .get(&function_index)
            .unwrap()
            .clone()
    }

    pub fn inc_counter(&mut self) {
        self.observer.counter += 1;
    }

    pub fn set_in_phantom(&mut self) {
        self.observer.is_in_phantom = true;
    }

    pub fn set_exit_phantom(&mut self) {
        self.observer.is_in_phantom = false;
    }

    pub fn dry_run(&self) -> bool {
        self.dry_run
    }
}

impl Tracer {
    pub(crate) fn push_init_memory(&mut self, memref: MemoryRef) {
        // one page contains 64KB*1024/8=8192 u64 entries
        const ENTRIES: u32 = 8192;

        let pages = (*memref).limits().initial();
        for i in 0..(pages * ENTRIES) {
            let mut buf = [0u8; 8];
            (*memref).get_into(i * 8, &mut buf).unwrap();

            let v = u64::from_le_bytes(buf);

            if v != 0 {
                self.imtable
                    .push(false, true, i, VarType::I64, u64::from_le_bytes(buf));
            }
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

    pub(crate) fn register_module_instance(&mut self, module_instance: &ModuleRef) {
        {
            let mut func_index = 0;

            loop {
                if let Some(func) = module_instance.func_by_index(func_index) {
                    if Some(&func) == self.wasm_input_func_ref.as_ref() {
                        self.wasm_input_func_idx = Some(func_index)
                    }

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

                    self.function_lookup.insert(func.clone(), func_index);
                    self.function_desc.insert(
                        func_index,
                        FuncDesc {
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
            let phantom_functions = self.phantom_functions.clone();

            for func_name_regex in phantom_functions {
                let re = Regex::new(&func_name_regex).unwrap();

                for (export_name, export) in module_instance.exports.borrow().iter() {
                    if re.is_match(export_name) && export.as_func().is_some() {
                        self.push_phantom_function(export.as_func().unwrap().clone());
                    }
                }
            }
        }

        {
            let mut func_index = 0;

            loop {
                if let Some(func) = module_instance.func_by_index(func_index) {
                    let funcdesc = self.function_desc.get(&func_index).unwrap();
                    let function_name = self.lookup_function_name(func_index);

                    if self.is_phantom_function(&func) {
                        let instructions = PhantomFunction::build_phantom_function_instructions(
                            &funcdesc.signature,
                            self.wasm_input_func_idx.unwrap(),
                        );

                        for (iid, inst) in instructions.into_iter().enumerate() {
                            self.itable.push(
                                func_index,
                                function_name.clone(),
                                iid as u32,
                                inst.into(&self.function_desc),
                            )
                        }
                    } else {
                        if let Some(body) = func.body() {
                            let code = &body.code;
                            let mut iter = code.iterate_from(0);
                            loop {
                                let pc = iter.position();
                                if let Some(instruction) = iter.next() {
                                    let _ = self.itable.push(
                                        func_index,
                                        function_name.clone(),
                                        pc,
                                        instruction.into(&self.function_desc),
                                    );
                                } else {
                                    break;
                                }
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

    pub fn lookup_function_name(&self, function: u32) -> String {
        if let Some(name) = self.function_lookup_name.get(&function) {
            name.to_owned()
        } else {
            function.to_string()
        }
    }

    pub fn lookup_function(&self, function: &FuncRef) -> u32 {
        *self.function_lookup.get(function).unwrap()
    }

    pub fn push_phantom_function(&mut self, function: FuncRef) {
        self.phantom_functions_ref.push(function)
    }

    pub fn is_phantom_function(&self, func: &FuncRef) -> bool {
        self.phantom_functions_ref.contains(func)
    }
}
