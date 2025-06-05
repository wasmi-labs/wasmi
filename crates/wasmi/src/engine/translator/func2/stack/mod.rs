mod control;
mod locals;
mod operand;
mod operands;

use self::{
    control::{
        BlockControlFrame,
        ControlFrame,
        ControlFrameKind,
        ControlStack,
        ElseControlFrame,
        ElseReachability,
        IfControlFrame,
        IfReachability,
        LoopControlFrame,
        UnreachableControlFrame,
    },
    locals::LocalsRegistry,
    operands::{OperandStack, StackOperand},
};
pub use self::{
    locals::LocalIdx,
    operand::Operand,
    operands::{OperandIdx, PreservedLocalsIter},
};
use super::Reset;
use crate::{
    core::{TypedVal, UntypedVal, ValType},
    engine::{
        translator::{Instr, LabelRef},
        BlockType,
        TranslationError,
    },
    ir::Reg,
    Engine,
    Error,
};
use alloc::vec::Vec;
use core::{array, mem, num::NonZero};

#[cfg(doc)]
use crate::ir::Instruction;

/// The Wasm value stack during translation from Wasm to Wasmi bytecode.
#[derive(Debug)]
pub struct Stack {
    /// The underlying [`Engine`].
    engine: Engine,
    /// The Wasm value stack.
    operands: OperandStack,
    /// The Wasm control stack.
    controls: ControlStack,
}

/// Reusable heap allocations for the [`Stack`].
#[derive(Debug, Default)]
pub struct StackAllocations {
    /// The Wasm value stack.
    operands: OperandStack,
    /// The Wasm control stack.
    controls: ControlStack,
}

impl Reset for StackAllocations {
    fn reset(&mut self) {
        self.operands.reset();
        self.controls.reset();
    }
}

impl Stack {
    /// Creates a new empty [`Stack`] from the given `engine`.
    pub fn new(engine: &Engine, alloc: StackAllocations) -> Self {
        Self {
            engine: engine.clone(),
            operands: alloc.operands,
            controls: alloc.controls,
        }
    }

    /// Returns the reusable [`StackAllocations`] of `self`.
    pub fn into_allocations(self) -> StackAllocations {
        StackAllocations {
            operands: self.operands,
            controls: self.controls,
        }
    }

    /// Resets the [`Stack`] for reuse.
    pub fn reset(&mut self) {
        self.operands.reset();
        self.controls.reset();
    }

    /// Register `amount` local variables of common type `ty`.
    ///
    /// # Errors
    ///
    /// If too many local variables are being registered.
    pub fn register_locals(&mut self, amount: u32, ty: ValType) -> Result<(), Error> {
        self.operands.register_locals(amount, ty)
    }

    /// Returns the current height of the [`Stack`].
    ///
    /// # Note
    ///
    /// The height is equal to the number of [`Operand`]s on the [`Stack`].
    pub fn height(&self) -> usize {
        self.operands.height()
    }

    /// Returns the maximum height of the [`Stack`].
    ///
    /// # Note
    ///
    /// The height is equal to the number of [`Operand`]s on the [`Stack`].
    pub fn max_height(&self) -> usize {
        self.operands.max_height()
    }

    /// Truncates `self` to the target `height`.
    ///
    /// All operands above `height` are dropped.
    ///
    /// # Panic
    ///
    /// If `height` is greater than the current height of `self`.
    pub fn trunc(&mut self, height: usize) {
        self.operands.trunc(height);
    }

    /// Returns `true` is fuel metering is enabled for the associated [`Engine`].
    fn is_fuel_metering_enabled(&self) -> bool {
        self.engine.config().get_consume_fuel()
    }

    /// Pushes a Wasm `block` onto the [`Stack`].
    ///
    /// # Note
    ///
    /// If `consume_fuel` is `None` and fuel metering is enabled this will inherit
    /// the [`Instruction::ConsumeFuel`] from the last control frame on the [`Stack`].
    ///
    /// # Errors
    ///
    /// If the stack height exceeds the maximum height.
    pub fn push_block(
        &mut self,
        ty: BlockType,
        label: LabelRef,
        consume_fuel: Option<Instr>,
    ) -> Result<(), Error> {
        let len_params = usize::from(ty.len_params(&self.engine));
        let block_height = self.height() - len_params;
        let fuel_metering = self.is_fuel_metering_enabled();
        let consume_fuel = match consume_fuel {
            None if fuel_metering => {
                let consume_instr = self
                    .controls
                    .get(0)
                    .consume_fuel_instr()
                    .expect("control frame must have consume instructions");
                Some(consume_instr)
            }
            consume_fuel => consume_fuel,
        };
        self.controls
            .push_block(ty, block_height, label, consume_fuel);
        Ok(())
    }

    /// Pushes a Wasm `loop` onto the [`Stack`].
    ///
    /// # Note
    ///
    /// Calls `f` for every non [`Operand::Temp`] operand on the [`Stack`]
    /// that is also a parameter to the pushed Wasm `loop` control frame.
    ///
    /// # Panics (debug)
    ///
    /// If `consume_fuel` is `None` and fuel metering is enabled.
    ///
    /// # Errors
    ///
    /// If the stack height exceeds the maximum height.
    pub fn push_loop(
        &mut self,
        ty: BlockType,
        label: LabelRef,
        consume_fuel: Option<Instr>,
        mut f: impl FnMut(Operand) -> Result<(), Error>,
    ) -> Result<(), Error> {
        debug_assert!(self.is_fuel_metering_enabled() == consume_fuel.is_some());
        let len_params = usize::from(ty.len_params(&self.engine));
        let block_height = self.height() - len_params;
        for depth in 0..block_height {
            if let Some(operand) = self.operand_to_temp(depth) {
                f(operand)?;
            }
        }
        self.controls
            .push_loop(ty, block_height, label, consume_fuel);
        Ok(())
    }

    /// Pushes a Wasm `if` onto the [`Stack`].
    ///
    /// # Panics (debug)
    ///
    /// If `consume_fuel` is `None` and fuel metering is enabled.
    ///
    /// # Errors
    ///
    /// If the stack height exceeds the maximum height.
    pub fn push_if(
        &mut self,
        ty: BlockType,
        label: LabelRef,
        reachability: IfReachability,
        consume_fuel: Option<Instr>,
    ) -> Result<(), Error> {
        debug_assert!(self.is_fuel_metering_enabled() == consume_fuel.is_some());
        let len_params = usize::from(ty.len_params(&self.engine));
        let block_height = self.height() - len_params;
        let else_operands = self.operands.peek(len_params);
        debug_assert!(len_params == else_operands.len());
        self.controls.push_if(
            ty,
            block_height,
            label,
            consume_fuel,
            reachability,
            else_operands,
        );
        Ok(())
    }

    /// Pushes a Wasm `else` onto the [`Stack`].
    ///
    /// # Panics (debug)
    ///
    /// If `consume_fuel` is `None` and fuel metering is enabled.
    ///
    /// # Errors
    ///
    /// If the stack height exceeds the maximum height.
    pub fn push_else(
        &mut self,
        ty: BlockType,
        label: LabelRef,
        reachability: ElseReachability,
        is_end_of_then_reachable: bool,
        consume_fuel: Option<Instr>,
    ) -> Result<(), Error> {
        debug_assert!(self.is_fuel_metering_enabled() == consume_fuel.is_some());
        let len_params = usize::from(ty.len_params(&self.engine));
        let block_height = self.height() - len_params;
        let else_operands = self.controls.push_else(
            ty,
            block_height,
            label,
            consume_fuel,
            reachability,
            is_end_of_then_reachable,
        );
        for operand in else_operands {
            match operand {
                Operand::Local(op) => {
                    self.operands.push_local(op.local_index())?;
                }
                Operand::Temp(op) => {
                    self.operands.push_temp(op.ty(), op.instr())?;
                }
                Operand::Immediate(op) => {
                    self.operands.push_immediate(op.val())?;
                }
            }
        }
        Ok(())
    }

    /// Pushes an unreachable Wasm control onto the [`Stack`].
    ///
    /// # Errors
    ///
    /// If the stack height exceeds the maximum height.
    pub fn push_unreachable(
        &mut self,
        ty: BlockType,
        kind: UnreachableControlFrame,
    ) -> Result<(), Error> {
        let len_params = usize::from(ty.len_params(&self.engine));
        let block_height = self.height() - len_params;
        self.controls.push_unreachable(ty, block_height, kind);
        Ok(())
    }

    /// Pops the top-most control frame from the control stack and returns it.
    ///
    /// # Panics
    ///
    /// If the control stack is empty.
    pub fn pop_control(&mut self) -> ControlFrame {
        self.controls
            .pop()
            .unwrap_or_else(|| panic!("tried to pop control from empty control stack"))
    }

    /// Pushes a local variable with index `local_idx` to the [`Stack`].
    ///
    /// # Errors
    ///
    /// - If too many operands have been pushed onto the [`Stack`].
    /// - If the local with `local_idx` does not exist.
    pub fn push_local(&mut self, local_index: LocalIdx) -> Result<OperandIdx, Error> {
        self.operands.push_local(local_index)
    }

    /// Pushes a temporary with type `ty` on the [`Stack`].
    ///
    /// # Errors
    ///
    /// If too many operands have been pushed onto the [`Stack`].
    pub fn push_temp(&mut self, ty: ValType, instr: Option<Instr>) -> Result<OperandIdx, Error> {
        self.operands.push_temp(ty, instr)
    }

    /// Pushes an immediate `value` on the [`Stack`].
    ///
    /// # Errors
    ///
    /// If too many operands have been pushed onto the [`Stack`].
    pub fn push_immediate(&mut self, value: impl Into<TypedVal>) -> Result<OperandIdx, Error> {
        self.operands.push_immediate(value)
    }

    /// Peeks the [`Operand`] at `depth`.
    ///
    /// # Note
    ///
    /// A depth of 0 peeks the top-most [`Operand`] on `self`.
    ///
    /// # Panics
    ///
    /// If `depth` is out of bounds for `self`.
    pub fn peek(&self, depth: usize) -> Operand {
        self.operands.get(depth)
    }

    /// Pops the top-most [`Operand`] from the [`Stack`].
    ///
    /// # Panics
    ///
    /// If `self` is empty.
    pub fn pop(&mut self) -> Operand {
        self.operands.pop()
    }

    /// Pops the two top-most [`Operand`] from the [`Stack`].
    ///
    /// # Note
    ///
    /// The last returned [`Operand`] is the top-most one.
    ///
    /// # Panics
    ///
    /// If `self` does not contain enough operands to pop.
    pub fn pop2(&mut self) -> (Operand, Operand) {
        let o2 = self.pop();
        let o1 = self.pop();
        (o1, o2)
    }

    /// Pops the two top-most [`Operand`] from the [`Stack`].
    ///
    /// # Note
    ///
    /// The last returned [`Operand`] is the top-most one.
    ///
    /// # Panics
    ///
    /// If `self` does not contain enough operands to pop.
    pub fn pop3(&mut self) -> (Operand, Operand, Operand) {
        let o3 = self.pop();
        let o2 = self.pop();
        let o1 = self.pop();
        (o1, o2, o3)
    }

    /// Preserve all locals on the [`Stack`] that refer to `local_index`.
    ///
    /// This is done by converting those locals to [`Operand::Temp`] and yielding them.
    ///
    /// # Note
    ///
    /// The users must fully consume all items yielded by the returned iterator in order
    /// for the local preservation to take full effect.
    ///
    /// # Panics
    ///
    /// If the local at `local_index` is out of bounds.
    #[must_use]
    pub fn preserve_locals(&mut self, local_index: LocalIdx) -> PreservedLocalsIter {
        self.operands.preserve_locals(local_index)
    }

    /// Converts and returns the [`StackOperand`] at `depth` into a [`Operand::Temp`].
    ///
    /// # Note
    ///
    /// Returns `None` if operand at `depth` is [`Operand::Temp`] already.
    ///
    /// # Panics
    ///
    /// If `depth` is out of bounds for the [`Stack`] of operands.
    #[must_use]
    pub fn operand_to_temp(&mut self, depth: usize) -> Option<Operand> {
        self.operands.operand_to_temp(depth)
    }
}
