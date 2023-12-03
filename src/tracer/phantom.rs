use specs::{host_function::HostPlugin, step::StepInfo, types::ValueType};

use super::{etable::ETable, Tracer};
use crate::{
    func::FuncRef,
    isa::{DropKeep, Instruction, Keep},
    Signature,
};

pub struct PhantomFunction;

impl PhantomFunction {
    pub fn build_phantom_function_instructions(
        sig: &Signature,
        // Wasm Image Function Id
        wasm_input_function_idx: u32,
    ) -> Vec<Instruction<'static>> {
        let mut instructions = vec![];

        if sig.return_type().is_some() {
            instructions.push(Instruction::I32Const(0));

            instructions.push(Instruction::Call(wasm_input_function_idx));

            if sig.return_type() != Some(wasmi_core::ValueType::I64) {
                instructions.push(Instruction::I32WrapI64);
            }
        }

        instructions.push(Instruction::Return(DropKeep {
            drop: sig.params().len() as u32,
            keep: if let Some(t) = sig.return_type() {
                Keep::Single(t.into_elements())
            } else {
                Keep::None
            },
        }));

        instructions
    }
}

impl Tracer {
    pub fn fill_trace(
        &mut self,
        current_sp: u32,
        allocated_memory_pages: u32,
        callee_func_ref: &FuncRef,
        callee_sig: &Signature,
        keep_value: Option<u64>,
    ) {
        let has_return_value = callee_sig.return_type().is_some();

        if self.dry_run() {
            if has_return_value {
                self.inc_counter();
                self.inc_counter();
                if callee_sig.return_type() != Some(wasmi_core::ValueType::I64) {
                    self.inc_counter();
                }
            }
            self.inc_counter();
            return;
        }

        let last_jump_eid = self.last_jump_eid();
        let fid = self.lookup_function(callee_func_ref);

        let mut iid = 0;

        let wasm_input_host_function_ref = self.wasm_input_func_ref.clone().unwrap();
        let wasm_input_host_func_index = match wasm_input_host_function_ref.as_internal() {
            crate::func::FuncInstanceInternal::Internal { .. } => unreachable!(),
            crate::func::FuncInstanceInternal::Host {
                host_func_index, ..
            } => host_func_index,
        };

        if has_return_value {
            self.etable.push(
                fid,
                iid,
                current_sp,
                allocated_memory_pages,
                last_jump_eid,
                StepInfo::I32Const { value: 0 },
            );

            iid += 1;

            self.etable.push(
                fid,
                iid,
                current_sp + 1,
                allocated_memory_pages,
                last_jump_eid,
                StepInfo::CallHost {
                    plugin: HostPlugin::HostInput,
                    host_function_idx: *wasm_input_host_func_index,
                    function_name: "wasm_input".to_owned(),
                    signature: specs::host_function::Signature {
                        params: vec![ValueType::I32],
                        return_type: Some(ValueType::I64),
                    },
                    args: vec![0],
                    ret_val: Some(keep_value.unwrap()),
                    op_index_in_plugin: 0,
                },
            );

            iid += 1;

            if callee_sig.return_type() != Some(wasmi_core::ValueType::I64) {
                self.etable.push(
                    fid,
                    iid,
                    current_sp + 1,
                    allocated_memory_pages,
                    last_jump_eid,
                    StepInfo::I32WrapI64 {
                        value: keep_value.unwrap() as i64,
                        result: keep_value.unwrap() as i32,
                    },
                );

                iid += 1;
            }
        }

        self.etable.push(
            fid,
            iid,
            current_sp + has_return_value as u32,
            allocated_memory_pages,
            last_jump_eid,
            StepInfo::Return {
                drop: callee_sig.params().len() as u32,
                keep: if let Some(t) = callee_sig.return_type() {
                    vec![t.into_elements().into()]
                } else {
                    vec![]
                },
                keep_values: keep_value.map_or(vec![], |v| vec![v]),
            },
        );
    }
}
