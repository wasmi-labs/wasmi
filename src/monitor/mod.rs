use parity_wasm::elements::Module;
use wasmi_core::Value;

use crate::{
    isa::Instruction,
    runner::{FunctionContext, InstructionOutcome, ValueStack},
    Error,
    ModuleRef,
};

pub trait Monitor {
    fn register_module(
        &mut self,
        _module: &Module,
        _module_ref: &ModuleRef,
        _entry: &str,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// Called before each exported function(zkmain or start function) is executed.
    fn invoke_exported_function_pre_hook(&mut self) {}

    /// Called before each instruction is executed.
    fn invoke_instruction_pre_hook(
        &mut self,
        _value_stack: &ValueStack,
        _function_context: &FunctionContext,
        _instruction: &Instruction,
    ) {
    }
    /// Called after each instruction is executed.
    fn invoke_instruction_post_hook(
        &mut self,
        _fid: u32,
        _iid: u32,
        _sp: u32,
        _allocated_memory_pages: u32,
        _value_stack: &ValueStack,
        _function_context: &FunctionContext,
        _instruction: &Instruction,
        _outcome: &InstructionOutcome,
    ) {
    }

    /// Called after 'call_host' instruction is executed.
    fn invoke_call_host_post_hook(&mut self, _return_value: Option<Value>) {}
}
