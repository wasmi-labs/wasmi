//! Definitions to compile a Wasm module into `wasmi` bytecode.
//!
//! The implementation is specific to the underlying Wasm parser
//! framework used by `wasmi` which currently is `parity_wasm`.

mod control_frame;
mod error;
mod utils;

use self::{control_frame::ControlFrame, error::TranslationError};
use super::{
    super::{DropKeep, FuncBody, InstructionIdx, InstructionsBuilder, LabelIdx, Target},
    Engine,
};
use crate::v2::interpreter::inst_builder::Reloc;
use alloc::vec::Vec;
use parity_wasm::elements::{self as pwasm, Instruction};
use validation::func::{top_label, FunctionValidationContext, StartedWith};

/// Allows to translate a Wasm functions into `wasmi` bytecode.
#[derive(Debug)]
pub struct FuncBodyTranslator {
    /// The underlying engine which the translator feeds.
    engine: Engine,
    /// The underlying instruction builder to incrementally build up `wasmi` bytecode.
    inst_builder: InstructionsBuilder,
    /// The underlying control flow frames representing Wasm control flow.
    control_frames: Vec<ControlFrame>,
}

impl FuncBodyTranslator {
    /// Creates a new Wasm function body translator for the given [`Engine`].
    pub fn new(engine: &Engine) -> Self {
        let mut inst_builder = InstructionsBuilder::default();
        // Push implicit frame for the whole function block.
        let end_label = inst_builder.new_label();
        let control_frames = vec![ControlFrame::Block { end_label }];
        Self {
            engine: engine.clone(),
            inst_builder,
            control_frames,
        }
    }

    /// Translates the instructions forming a Wasm function body into `wasmi` bytecode.
    ///
    /// Returns a [`FuncBody`] reference to the translated `wasmi` bytecode.
    pub fn translate<'a, I: 'a>(
        &mut self,
        validator: &mut FunctionValidationContext,
        instructions: I,
    ) -> Result<FuncBody, TranslationError>
    where
        I: IntoIterator<Item = &'a pwasm::Instruction>,
        I::IntoIter: ExactSizeIterator,
    {
        for instruction in instructions {
            self.translate_instruction(validator, instruction)?;
        }
        let func_body = self.inst_builder.finish(&self.engine);
        Ok(func_body)
    }

    /// Pops the top most control frame.
    ///
    /// # Panics
    ///
    /// If the control flow frame stack is empty.
    fn pop_control_frame(&mut self) -> ControlFrame {
        self.control_frames
            .pop()
            .expect("encountered unexpected empty control frame stack")
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

    /// Validate the Wasm `inst` and build the respective `wasmi` bytecode.
    fn validate_and_build<F, R>(
        &mut self,
        validator: &mut FunctionValidationContext,
        inst: &Instruction,
        f: F,
    ) -> Result<(), TranslationError>
    where
        F: FnOnce(&mut InstructionsBuilder) -> R,
    {
        validator.step(inst)?;
        f(&mut self.inst_builder);
        Ok(())
    }

    /// Translates a single Wasm instruction into `wasmi` bytecode.
    ///
    /// # Errors
    ///
    /// If there are validation or translation problems.
    fn translate_instruction(
        &mut self,
        validator: &mut FunctionValidationContext,
        instruction: &Instruction,
    ) -> Result<(), TranslationError> {
        use Instruction as Inst;
        match instruction {
            Inst::Unreachable => {
                self.validate_and_build(validator, instruction, InstructionsBuilder::unreachable)?;
            }
            Inst::Nop => {
                validator.step(instruction)?;
            }
            Inst::Block(_block_type) => {
                self.translate_block(validator, instruction)?;
            }
            Inst::Loop(_block_type) => {
                self.translate_loop(validator, instruction)?;
            }
            Inst::If(_block_type) => {
                self.translate_if(validator, instruction)?;
            }
            Inst::Else => {
                self.translate_else(validator, instruction)?;
            }
            Inst::End => {
                self.translate_end(validator, instruction)?;
            }
            Inst::Br(depth) => {
                self.translate_br(depth, validator, instruction)?;
            }
            Inst::BrIf(depth) => {
                self.translate_br_if(validator, instruction, depth)?;
            }
            Inst::BrTable(br_table) => {
                self.translate_br_table(validator, br_table, instruction)?;
            }
            Inst::Return => {
                self.translate_return(validator, instruction)?;
            }
            Inst::Call(_func_idx) => todo!(),
            Inst::CallIndirect(_signature_idx, _table_ref) => todo!(),
            Inst::Drop => todo!(),
            Inst::Select => todo!(),
            Inst::GetLocal(_local_idx) => todo!(),
            Inst::SetLocal(_local_idx) => todo!(),
            Inst::TeeLocal(_local_idx) => todo!(),
            Inst::GetGlobal(_global_idx) => todo!(),
            Inst::SetGlobal(_global_idx) => todo!(),
            Inst::I32Load(_, _) => todo!(),
            Inst::I64Load(_, _) => todo!(),
            Inst::F32Load(_, _) => todo!(),
            Inst::F64Load(_, _) => todo!(),
            Inst::I32Load8S(_, _) => todo!(),
            Inst::I32Load8U(_, _) => todo!(),
            Inst::I32Load16S(_, _) => todo!(),
            Inst::I32Load16U(_, _) => todo!(),
            Inst::I64Load8S(_, _) => todo!(),
            Inst::I64Load8U(_, _) => todo!(),
            Inst::I64Load16S(_, _) => todo!(),
            Inst::I64Load16U(_, _) => todo!(),
            Inst::I64Load32S(_, _) => todo!(),
            Inst::I64Load32U(_, _) => todo!(),
            Inst::I32Store(_, _) => todo!(),
            Inst::I64Store(_, _) => todo!(),
            Inst::F32Store(_, _) => todo!(),
            Inst::F64Store(_, _) => todo!(),
            Inst::I32Store8(_, _) => todo!(),
            Inst::I32Store16(_, _) => todo!(),
            Inst::I64Store8(_, _) => todo!(),
            Inst::I64Store16(_, _) => todo!(),
            Inst::I64Store32(_, _) => todo!(),
            Inst::CurrentMemory(_) => todo!(),
            Inst::GrowMemory(_) => todo!(),
            Inst::I32Const(_) => todo!(),
            Inst::I64Const(_) => todo!(),
            Inst::F32Const(_) => todo!(),
            Inst::F64Const(_) => todo!(),
            Inst::I32Eqz => todo!(),
            Inst::I32Eq => todo!(),
            Inst::I32Ne => todo!(),
            Inst::I32LtS => todo!(),
            Inst::I32LtU => todo!(),
            Inst::I32GtS => todo!(),
            Inst::I32GtU => todo!(),
            Inst::I32LeS => todo!(),
            Inst::I32LeU => todo!(),
            Inst::I32GeS => todo!(),
            Inst::I32GeU => todo!(),
            Inst::I64Eqz => todo!(),
            Inst::I64Eq => todo!(),
            Inst::I64Ne => todo!(),
            Inst::I64LtS => todo!(),
            Inst::I64LtU => todo!(),
            Inst::I64GtS => todo!(),
            Inst::I64GtU => todo!(),
            Inst::I64LeS => todo!(),
            Inst::I64LeU => todo!(),
            Inst::I64GeS => todo!(),
            Inst::I64GeU => todo!(),
            Inst::F32Eq => todo!(),
            Inst::F32Ne => todo!(),
            Inst::F32Lt => todo!(),
            Inst::F32Gt => todo!(),
            Inst::F32Le => todo!(),
            Inst::F32Ge => todo!(),
            Inst::F64Eq => todo!(),
            Inst::F64Ne => todo!(),
            Inst::F64Lt => todo!(),
            Inst::F64Gt => todo!(),
            Inst::F64Le => todo!(),
            Inst::F64Ge => todo!(),
            Inst::I32Clz => todo!(),
            Inst::I32Ctz => todo!(),
            Inst::I32Popcnt => todo!(),
            Inst::I32Add => todo!(),
            Inst::I32Sub => todo!(),
            Inst::I32Mul => todo!(),
            Inst::I32DivS => todo!(),
            Inst::I32DivU => todo!(),
            Inst::I32RemS => todo!(),
            Inst::I32RemU => todo!(),
            Inst::I32And => todo!(),
            Inst::I32Or => todo!(),
            Inst::I32Xor => todo!(),
            Inst::I32Shl => todo!(),
            Inst::I32ShrS => todo!(),
            Inst::I32ShrU => todo!(),
            Inst::I32Rotl => todo!(),
            Inst::I32Rotr => todo!(),
            Inst::I64Clz => todo!(),
            Inst::I64Ctz => todo!(),
            Inst::I64Popcnt => todo!(),
            Inst::I64Add => todo!(),
            Inst::I64Sub => todo!(),
            Inst::I64Mul => todo!(),
            Inst::I64DivS => todo!(),
            Inst::I64DivU => todo!(),
            Inst::I64RemS => todo!(),
            Inst::I64RemU => todo!(),
            Inst::I64And => todo!(),
            Inst::I64Or => todo!(),
            Inst::I64Xor => todo!(),
            Inst::I64Shl => todo!(),
            Inst::I64ShrS => todo!(),
            Inst::I64ShrU => todo!(),
            Inst::I64Rotl => todo!(),
            Inst::I64Rotr => todo!(),
            Inst::F32Abs => todo!(),
            Inst::F32Neg => todo!(),
            Inst::F32Ceil => todo!(),
            Inst::F32Floor => todo!(),
            Inst::F32Trunc => todo!(),
            Inst::F32Nearest => todo!(),
            Inst::F32Sqrt => todo!(),
            Inst::F32Add => todo!(),
            Inst::F32Sub => todo!(),
            Inst::F32Mul => todo!(),
            Inst::F32Div => todo!(),
            Inst::F32Min => todo!(),
            Inst::F32Max => todo!(),
            Inst::F32Copysign => todo!(),
            Inst::F64Abs => todo!(),
            Inst::F64Neg => todo!(),
            Inst::F64Ceil => todo!(),
            Inst::F64Floor => todo!(),
            Inst::F64Trunc => todo!(),
            Inst::F64Nearest => todo!(),
            Inst::F64Sqrt => todo!(),
            Inst::F64Add => todo!(),
            Inst::F64Sub => todo!(),
            Inst::F64Mul => todo!(),
            Inst::F64Div => todo!(),
            Inst::F64Min => todo!(),
            Inst::F64Max => todo!(),
            Inst::F64Copysign => todo!(),
            Inst::I32WrapI64 => todo!(),
            Inst::I32TruncSF32 => todo!(),
            Inst::I32TruncUF32 => todo!(),
            Inst::I32TruncSF64 => todo!(),
            Inst::I32TruncUF64 => todo!(),
            Inst::I64ExtendSI32 => todo!(),
            Inst::I64ExtendUI32 => todo!(),
            Inst::I64TruncSF32 => todo!(),
            Inst::I64TruncUF32 => todo!(),
            Inst::I64TruncSF64 => todo!(),
            Inst::I64TruncUF64 => todo!(),
            Inst::F32ConvertSI32 => todo!(),
            Inst::F32ConvertUI32 => todo!(),
            Inst::F32ConvertSI64 => todo!(),
            Inst::F32ConvertUI64 => todo!(),
            Inst::F32DemoteF64 => todo!(),
            Inst::F64ConvertSI32 => todo!(),
            Inst::F64ConvertUI32 => todo!(),
            Inst::F64ConvertSI64 => todo!(),
            Inst::F64ConvertUI64 => todo!(),
            Inst::F64PromoteF32 => todo!(),
            Inst::I32ReinterpretF32 => todo!(),
            Inst::I64ReinterpretF64 => todo!(),
            Inst::F32ReinterpretI32 => todo!(),
            Inst::F64ReinterpretI64 => todo!(),
        }
        Ok(())
    }

    /// Translates a Wasm `block` control flow instruction into `wasmi` bytecode.
    fn translate_block(
        &mut self,
        validator: &mut FunctionValidationContext,
        instruction: &Instruction,
    ) -> Result<(), TranslationError> {
        validator.step(instruction)?;
        let end_label = self.inst_builder.new_label();
        self.control_frames.push(ControlFrame::Block { end_label });
        Ok(())
    }

    /// Translates a Wasm `loop` control flow instruction into `wasmi` bytecode.
    fn translate_loop(
        &mut self,
        validator: &mut FunctionValidationContext,
        instruction: &Instruction,
    ) -> Result<(), TranslationError> {
        validator.step(instruction)?;
        let header = self.inst_builder.new_label();
        self.inst_builder.resolve_label(header);
        self.control_frames.push(ControlFrame::Loop { header });
        Ok(())
    }

    /// Translates a Wasm `if` control flow instruction into `wasmi` bytecode.
    fn translate_if(
        &mut self,
        validator: &mut FunctionValidationContext,
        instruction: &Instruction,
    ) -> Result<(), TranslationError> {
        validator.step(instruction)?;
        let else_label = self.inst_builder.new_label();
        let end_label = self.inst_builder.new_label();
        self.control_frames.push(ControlFrame::If {
            else_label,
            end_label,
        });
        let dst_pc = self.try_resolve_label(else_label, |pc| Reloc::Br { inst_idx: pc });
        self.inst_builder
            .branch_eqz(Target::new(dst_pc, DropKeep::new(0, 0)));
        Ok(())
    }

    /// Translates a Wasm `else` control flow instruction into `wasmi` bytecode.
    fn translate_else(
        &mut self,
        validator: &mut FunctionValidationContext,
        instruction: &Instruction,
    ) -> Result<(), TranslationError> {
        validator.step(instruction)?;
        let top_frame = self.pop_control_frame();
        let (else_label, end_label) = match top_frame {
            ControlFrame::If { else_label, end_label } => (else_label, end_label),
            unexpected => unreachable!(
                "expect Wasm `if` control flow frame at this point due to validation but found: {:?}",
                unexpected,
            ),
        };
        let dst_pc = self.try_resolve_label(end_label, |pc| Reloc::Br { inst_idx: pc });
        self.inst_builder
            .branch(Target::new(dst_pc, DropKeep::new(0, 0)));
        self.inst_builder.resolve_label(else_label);
        self.control_frames.push(ControlFrame::Else { end_label });
        Ok(())
    }

    /// Translates a Wasm `end` control flow instruction into `wasmi` bytecode.
    fn translate_end(
        &mut self,
        validator: &mut FunctionValidationContext,
        instruction: &Instruction,
    ) -> Result<(), TranslationError> {
        let started_with = top_label(&validator.frame_stack).started_with;
        let return_drop_keep = if validator.frame_stack.len() == 1 {
            // We are about to close the last frame.
            Some(utils::drop_keep_return(
                &validator.locals,
                &validator.value_stack,
                &validator.frame_stack,
            ))
        } else {
            None
        };
        validator.step(instruction)?;
        let top_frame = self.pop_control_frame();
        if let ControlFrame::If { else_label, .. } = top_frame {
            // At this point we can resolve the `Else` label.
            self.inst_builder.resolve_label(else_label);
        }
        if started_with != StartedWith::Loop {
            let end_label = top_frame.end_label();
            self.inst_builder.resolve_label(end_label);
        }
        if let Some(drop_keep) = return_drop_keep {
            // It was the last instruction therefore we emit the explicit return instruction.
            let drop_keep = drop_keep.unwrap_or_else(|error| {
                panic!(
                    "due to validation the value stack must not have underflowed. \
                            Validation also ensures that the frame stack is not empty: {:?}",
                    error
                )
            });
            self.inst_builder.ret(drop_keep);
        }
        Ok(())
    }

    /// Translates a Wasm `br` control flow instruction into `wasmi` bytecode.
    fn translate_br(
        &mut self,
        depth: &u32,
        validator: &mut FunctionValidationContext,
        instruction: &Instruction,
    ) -> Result<(), TranslationError> {
        let target = utils::require_target(
            *depth,
            validator.value_stack.len(),
            &validator.frame_stack,
            &self.control_frames,
        );
        validator.step(instruction)?;
        let (end_label, drop_keep) = target.unwrap_or_else(|error| {
            panic!(
                "due to validation the value stack must not underflow \
                        and the branching depth is valid at this point: {:?}",
                error
            )
        });
        let dst_pc = self.try_resolve_label(end_label, |pc| Reloc::Br { inst_idx: pc });
        self.inst_builder.branch(Target::new(dst_pc, drop_keep));
        Ok(())
    }

    /// Translates a Wasm `br_if` control flow instruction into `wasmi` bytecode.
    fn translate_br_if(
        &mut self,
        validator: &mut FunctionValidationContext,
        instruction: &Instruction,
        depth: &u32,
    ) -> Result<(), TranslationError> {
        validator.step(instruction)?;
        let (end_label, drop_keep) = utils::require_target(
            *depth,
            validator.value_stack.len(),
            &validator.frame_stack,
            &self.control_frames,
        )
        .unwrap_or_else(|error| {
            panic!(
                "due to validation the value stack must not underflow \
                        and the branching depth is valid at this point: {:?}",
                error
            )
        });
        let dst_pc = self.try_resolve_label(end_label, |pc| Reloc::Br { inst_idx: pc });
        self.inst_builder.branch_nez(Target::new(dst_pc, drop_keep));
        Ok(())
    }

    /// Translates a Wasm `br_table` control flow instruction into `wasmi` bytecode.
    fn translate_br_table(
        &mut self,
        validator: &mut FunctionValidationContext,
        br_table: &pwasm::BrTableData,
        instruction: &Instruction,
    ) -> Result<(), TranslationError> {
        // At this point, the condition value is at the top of the stack.
        // But at the point of actual jump the condition will already be
        // popped off.
        let value_stack_height = validator.value_stack.len().saturating_sub(1);
        let targets = br_table
            .table
            .iter()
            .map(|depth| {
                utils::require_target(
                    *depth,
                    value_stack_height,
                    &validator.frame_stack,
                    &self.control_frames,
                )
            })
            .collect::<Result<Vec<_>, _>>();
        let default_target = utils::require_target(
            br_table.default,
            value_stack_height,
            &validator.frame_stack,
            &self.control_frames,
        );
        validator.step(instruction)?;
        const REQUIRE_TARGET_PROOF: &str = "could not resolve targets or default target of the \
                    `br_table` even though it validated properly";
        let targets = targets.unwrap_or_else(|error| panic!("{}: {}", REQUIRE_TARGET_PROOF, error));
        let default_target =
            default_target.unwrap_or_else(|error| panic!("{}: {}", REQUIRE_TARGET_PROOF, error));
        let mut branch_arm_target = |index, label, drop_keep| {
            let dst_pc = self.try_resolve_label(label, |pc| Reloc::BrTable {
                inst_idx: pc,
                target_idx: index,
            });
            Target::new(dst_pc, drop_keep)
        };
        let targets = targets
            .into_iter()
            .enumerate()
            .map(|(target_idx, (label, drop_keep))| branch_arm_target(target_idx, label, drop_keep))
            .collect::<Vec<_>>();
        let default_target = {
            let (label_idx, drop_keep) = default_target;
            branch_arm_target(targets.len(), label_idx, drop_keep)
        };
        self.inst_builder.branch_table(default_target, targets);
        Ok(())
    }

    /// Translates a Wasm `return` control flow instruction into `wasmi` bytecode.
    fn translate_return(
        &mut self,
        validator: &mut FunctionValidationContext,
        instruction: &Instruction,
    ) -> Result<(), TranslationError> {
        let drop_keep = utils::drop_keep_return(
            &validator.locals,
            &validator.value_stack,
            &validator.frame_stack,
        );
        validator.step(instruction)?;
        let drop_keep = drop_keep.unwrap_or_else(|error| {
            panic!(
                "due to validation the value stack must not have underflowed. \
                         Validation also ensures that the frame stack is not empty: {:?}",
                error
            )
        });
        self.inst_builder.ret(drop_keep);
        Ok(())
    }
}
