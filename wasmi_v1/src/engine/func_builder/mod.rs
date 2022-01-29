mod control_frame;
mod control_stack;
mod inst_builder;
mod value_stack;

pub use self::inst_builder::{
    BrTable,
    InstructionIdx,
    InstructionsBuilder,
    LabelIdx,
    RelativeDepth,
    Reloc,
};
use self::{control_frame::ControlFrame, control_stack::ControlFlowStack, value_stack::ValueStack};
use super::{DropKeep, Instruction, Target};
use crate::{
    module2::{BlockType, FuncIdx, FuncTypeIdx, GlobalIdx, ModuleResources, TableIdx},
    Engine,
    ModuleError,
};
use wasmi_core::ValueType;

/// The interface to translate a `wasmi` bytecode function using Wasm bytecode.
#[derive(Debug)]
pub struct FunctionBuilder<'engine, 'parser> {
    /// The [`Engine`] for which the function is translated.
    engine: &'engine Engine,
    /// The function under construction.
    func: FuncIdx,
    /// The immutable `wasmi` module resources.
    res: ModuleResources<'parser>,
    /// The control flow frame stack that represents the Wasm control flow.
    control_frames: ControlFlowStack,
    /// The emulated value stack.
    value_stack: ValueStack,
    /// The instruction builder.
    ///
    /// # Note
    ///
    /// Allows to incrementally construct the instruction of a function.
    inst_builder: InstructionsBuilder,
    /// The amount of local variables of the currently compiled function.
    len_locals: usize,
    /// The maximum height of the emulated value stack of the translated function.
    ///
    /// # Note
    ///
    /// This does not include input parameters and local variables.
    max_stack_height: usize,
    /// This represents the reachability of the currently translated code.
    ///
    /// - `true`: The currently translated code is reachable.
    /// - `false`: The currently translated code is unreachable and can be skipped.
    ///
    /// # Note
    ///
    /// Visiting the Wasm `Else` or `End` control flow operator resets
    /// reachability to `true` again.
    reachable: bool,
}

impl<'engine, 'parser> FunctionBuilder<'engine, 'parser> {
    /// Creates a new [`FunctionBuilder`].
    pub fn new(engine: &'engine Engine, func: FuncIdx, res: ModuleResources<'parser>) -> Self {
        Self {
            engine,
            func,
            res,
            control_frames: ControlFlowStack::default(),
            value_stack: ValueStack::default(),
            inst_builder: InstructionsBuilder::default(),
            len_locals: 0,
            max_stack_height: 0,
            reachable: true,
        }
    }

    /// Try to resolve the given label.
    ///
    /// In case the label cannot yet be resolved register the [`Reloc`] as its user.
    fn try_resolve_label<F>(&mut self, label: LabelIdx, reloc_provider: F) -> InstructionIdx
    where
        F: FnOnce(InstructionIdx) -> Reloc,
    {
        let pc = self.inst_builder.current_pc();
        self.inst_builder
            .try_resolve_label(label, || reloc_provider(pc))
    }

    /// Translates the given local variables for the translated function.
    pub fn translate_locals(
        &mut self,
        amount: u32,
        _value_type: ValueType,
    ) -> Result<(), ModuleError> {
        self.len_locals += amount as usize;
        Ok(())
    }
}

impl<'engine, 'parser> FunctionBuilder<'engine, 'parser> {
    /// Translates a Wasm `unreachable` instruction.
    pub fn translate_unreachable(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translates a Wasm `block` control flow operator.
    pub fn translate_block(&mut self, _block_type: BlockType) -> Result<(), ModuleError> {
        let end_label = self.inst_builder.new_label();
        self.control_frames
            .push_frame(ControlFrame::Block { end_label });
        Ok(())
    }

    /// Translates a Wasm `block` control flow operator.
    pub fn translate_loop(&mut self, _block_type: BlockType) -> Result<(), ModuleError> {
        let header = self.inst_builder.new_label();
        self.inst_builder.resolve_label(header);
        self.control_frames
            .push_frame(ControlFrame::Loop { header });
        Ok(())
    }

    /// Translates a Wasm `if` control flow operator.
    pub fn translate_if(&mut self, _block_type: BlockType) -> Result<(), ModuleError> {
        let else_label = self.inst_builder.new_label();
        let end_label = self.inst_builder.new_label();
        self.control_frames.push_frame(ControlFrame::If {
            else_label,
            end_label,
        });
        let dst_pc = self.try_resolve_label(else_label, |pc| Reloc::Br { inst_idx: pc });
        let branch_target = Target::new(dst_pc, DropKeep::new(0, 0));
        self.inst_builder
            .push_inst(Instruction::BrIfEqz(branch_target));
        Ok(())
    }

    /// Translates a Wasm `else` control flow operator.
    pub fn translate_else(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translates a Wasm `end` control flow operator.
    pub fn translate_end(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translates a Wasm `br` control flow operator.
    pub fn translate_br(&mut self, relative_depth: u32) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translates a Wasm `br_if` control flow operator.
    pub fn translate_br_if(&mut self, relative_depth: u32) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translates a Wasm `br_table` control flow operator.
    pub fn translate_br_table(&mut self, br_table: impl BrTable) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translates a Wasm `return` control flow operator.
    pub fn translate_return(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translates a Wasm `call` instruction.
    pub fn translate_call(&mut self, func_idx: FuncIdx) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translates a Wasm `call_indirect` instruction.
    pub fn translate_call_indirect(
        &mut self,
        func_type_idx: FuncTypeIdx,
        table_idx: TableIdx,
    ) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translates a Wasm `drop` instruction.
    pub fn translate_drop(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translates a Wasm `select` instruction.
    pub fn translate_select(&mut self) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `local.get` instruction.
    pub fn translate_local_get(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `local.set` instruction.
    pub fn translate_local_set(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `local.tee` instruction.
    pub fn translate_local_tee(&mut self, local_idx: u32) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `global.get` instruction.
    pub fn translate_global_get(&mut self, global_idx: GlobalIdx) -> Result<(), ModuleError> {
        todo!()
    }

    /// Translate a Wasm `global.set` instruction.
    pub fn translate_global_set(&mut self, global_idx: GlobalIdx) -> Result<(), ModuleError> {
        todo!()
    }
}
