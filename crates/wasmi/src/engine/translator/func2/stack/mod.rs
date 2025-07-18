mod control;
mod locals;
mod operand;
mod operands;

use self::{
    control::ControlStack,
    locals::LocalsRegistry,
    operands::{OperandStack, StackOperand},
};
pub use self::{
    control::{
        AcquiredTarget,
        BlockControlFrame,
        ControlFrame,
        ControlFrameBase,
        ControlFrameKind,
        ElseControlFrame,
        ElseReachability,
        IfControlFrame,
        IfReachability,
        LoopControlFrame,
    },
    locals::LocalIdx,
    operand::{ImmediateOperand, Operand, TempOperand},
    operands::{OperandIdx, PreservedLocalsIter},
};
use super::{Reset, ReusableAllocations};
use crate::{
    core::{TypedVal, ValType},
    engine::{
        translator::{labels::LabelRef, utils::Instr},
        BlockType,
    },
    Engine,
    Error,
};
use alloc::vec::Vec;

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

impl ReusableAllocations for Stack {
    type Allocations = StackAllocations;

    fn into_allocations(self) -> StackAllocations {
        StackAllocations {
            operands: self.operands,
            controls: self.controls,
        }
    }
}

impl Stack {
    /// Creates a new empty [`Stack`] from the given `engine`.
    pub fn new(engine: &Engine, alloc: StackAllocations) -> Self {
        let StackAllocations { operands, controls } = alloc.into_reset();
        Self {
            engine: engine.clone(),
            operands,
            controls,
        }
    }

    /// Register `amount` local variables of common type `ty`.
    ///
    /// # Errors
    ///
    /// If too many local variables are being registered.
    pub fn register_locals(&mut self, amount: u32, ty: ValType) -> Result<(), Error> {
        self.operands.register_locals(amount, ty)
    }

    /// Returns `true` if the control stack is empty.
    pub fn is_control_empty(&self) -> bool {
        self.controls.is_empty()
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
        debug_assert!(height <= self.height());
        while self.height() > height {
            self.pop();
        }
    }

    /// Returns `true` is fuel metering is enabled for the associated [`Engine`].
    fn is_fuel_metering_enabled(&self) -> bool {
        self.engine.config().get_consume_fuel()
    }

    /// Pushes the function enclosing Wasm `block` onto the [`Stack`].
    ///
    /// # Note
    ///
    /// - If `consume_fuel` is `None` fuel metering is expected to be disabled.
    /// - If `consume_fuel` is `Some` fuel metering is expected to be enabled.
    ///
    /// # Errors
    ///
    /// If the stack height exceeds the maximum height.
    pub fn push_func_block(
        &mut self,
        ty: BlockType,
        label: LabelRef,
        consume_fuel: Option<Instr>,
    ) -> Result<(), Error> {
        debug_assert!(self.controls.is_empty());
        debug_assert!(self.is_fuel_metering_enabled() == consume_fuel.is_some());
        self.controls.push_block(ty, 0, label, consume_fuel);
        Ok(())
    }

    /// Pushes a Wasm `block` onto the [`Stack`].
    ///
    /// # Note
    ///
    /// This inherits the `consume_fuel` [`Instr`] from the parent [`ControlFrame`].
    ///
    /// # Errors
    ///
    /// If the stack height exceeds the maximum height.
    pub fn push_block(&mut self, ty: BlockType, label: LabelRef) -> Result<(), Error> {
        debug_assert!(!self.controls.is_empty());
        let len_params = usize::from(ty.len_params(&self.engine));
        let block_height = self.height() - len_params;
        let fuel_metering = self.is_fuel_metering_enabled();
        let consume_fuel = match fuel_metering {
            true => {
                let consume_instr = self
                    .controls
                    .get(0)
                    .consume_fuel_instr()
                    .expect("control frame must have consume instructions");
                Some(consume_instr)
            }
            false => None,
        };
        self.controls
            .push_block(ty, block_height, label, consume_fuel);
        Ok(())
    }

    /// Pushes a Wasm `loop` onto the [`Stack`].
    ///
    /// # Panics (debug)
    ///
    /// - If `consume_fuel` is `None` and fuel metering is enabled.
    /// - If any of the Wasm `loop` operand parameters are _not_ [`Operand::Temp`].
    ///
    /// # Errors
    ///
    /// If the stack height exceeds the maximum height.
    pub fn push_loop(
        &mut self,
        ty: BlockType,
        label: LabelRef,
        consume_fuel: Option<Instr>,
    ) -> Result<(), Error> {
        debug_assert!(!self.controls.is_empty());
        debug_assert!(self.is_fuel_metering_enabled() == consume_fuel.is_some());
        let len_params = usize::from(ty.len_params(&self.engine));
        let block_height = self.height() - len_params;
        debug_assert!(self
            .operands
            .peek(len_params)
            .all(|operand| operand.is_temp()));
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
        debug_assert!(!self.controls.is_empty());
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
        if_frame: IfControlFrame,
        is_end_of_then_reachable: bool,
        consume_fuel: Option<Instr>,
    ) -> Result<(), Error> {
        debug_assert!(self.is_fuel_metering_enabled() == consume_fuel.is_some());
        // self.trunc(if_frame.height());
        let else_operands =
            self.controls
                .push_else(if_frame, consume_fuel, is_end_of_then_reachable);
        if let Some(else_operands) = else_operands {
            for else_operand in else_operands {
                self.operands.push_operand(else_operand)?;
            }
        }
        Ok(())
    }

    /// Pushes an unreachable Wasm control onto the [`Stack`].
    ///
    /// # Errors
    ///
    /// If the stack height exceeds the maximum height.
    pub fn push_unreachable(&mut self, kind: ControlFrameKind) -> Result<(), Error> {
        self.controls.push_unreachable(kind);
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

    /// Pushes the top-most `else` operands from the control stack onto the operand stack.
    ///
    /// # Panics (Debug)
    ///
    /// If the `else` operands are not in orphaned state.
    pub fn push_else_operands(&mut self, frame: &impl ControlFrameBase) -> Result<(), Error> {
        self.trunc(frame.height());
        for else_operand in self.controls.pop_else_operands() {
            self.operands.push_operand(else_operand)?;
        }
        Ok(())
    }

    /// Returns a shared reference to the [`ControlFrame`] at `depth`.
    ///
    /// # Panics
    ///
    /// If `depth` is out of bounds for `self`.
    pub fn peek_control(&self, depth: usize) -> &ControlFrame {
        self.controls.get(depth)
    }

    /// Returns an exclusive reference to the [`ControlFrame`] at `depth`.
    ///
    /// # Note
    ///
    /// This returns an [`AcquiredTarget`] to differentiate between the function
    /// body Wasm `block` and other control frames in order to know whether a branching
    /// target returns or branches.
    ///
    /// # Panics
    ///
    /// If `depth` is out of bounds for `self`.
    pub fn peek_control_mut(&mut self, depth: usize) -> AcquiredTarget<'_> {
        self.controls.acquire_target(depth)
    }

    /// Pushes the [`Operand`] back to the [`Stack`].
    ///
    /// Returns the new [`OperandIdx`].
    ///
    /// # Errors
    ///
    /// - If too many operands have been pushed onto the [`Stack`].
    /// - If the local with `local_idx` does not exist.
    pub fn push_operand(&mut self, operand: Operand) -> Result<OperandIdx, Error> {
        self.operands.push_operand(operand)
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

    /// Peeks the 2 top-most [`Operand`]s.
    ///
    /// # Panics
    ///
    /// If there aren't at least 2 [`Operand`]s on the [`Stack`].
    pub fn peek2(&self) -> (Operand, Operand) {
        let v0 = self.peek(1);
        let v1 = self.peek(0);
        (v0, v1)
    }

    /// Peeks the 3 top-most [`Operand`]s.
    ///
    /// # Panics
    ///
    /// If there aren't at least 2 [`Operand`]s on the [`Stack`].
    pub fn peek3(&self) -> (Operand, Operand, Operand) {
        let v0 = self.peek(2);
        let v1 = self.peek(1);
        let v2 = self.peek(0);
        (v0, v1, v2)
    }

    /// Peeks the top-most `len` operands from the stack and store them into `buffer`.
    ///
    /// Operands stored into the buffer are placed in order.
    pub fn peek_n(&mut self, len: usize, buffer: &mut Vec<Operand>) {
        buffer.clear();
        buffer.extend((0..len).rev().map(|depth| self.peek(depth)));
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

    /// Pops `len` operands from the stack and store them into `buffer`.
    ///
    /// Operands stored into the buffer are placed in order.
    pub fn pop_n(&mut self, len: usize, buffer: &mut Vec<Operand>) {
        buffer.clear();
        for _ in 0..len {
            let operand = self.pop();
            buffer.push(operand);
        }
        buffer.reverse();
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
    pub fn preserve_locals(&mut self, local_index: LocalIdx) -> PreservedLocalsIter<'_> {
        self.operands.preserve_locals(local_index)
    }

    /// Converts and returns the [`Operand`] at `depth` into a [`Operand::Temp`].
    ///
    /// # Note
    ///
    /// - Returns the [`Operand`] at `depth` before being converted to an [`Operand::Temp`].
    /// - [`Operand::Temp`] will have their optional `instr` set to `None`.
    ///
    /// # Panics
    ///
    /// If `depth` is out of bounds for the [`Stack`] of operands.
    #[must_use]
    pub fn operand_to_temp(&mut self, depth: usize) -> Operand {
        self.operands.operand_to_temp(depth)
    }

    /// Returns the current [`Instruction::ConsumeFuel`] if fuel metering is enabled.
    ///
    /// Returns `None` otherwise.
    pub fn consume_fuel_instr(&self) -> Option<Instr> {
        debug_assert!(!self.controls.is_empty());
        self.controls.get(0).consume_fuel_instr()
    }
}
