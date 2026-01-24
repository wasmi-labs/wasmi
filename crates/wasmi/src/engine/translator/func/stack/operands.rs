use super::{ImmediateOperand, LocalIdx, LocalOperand, LocalsHead, Operand, Reset, TempOperand};
use crate::{
    Error,
    ValType,
    core::{TypedVal, UntypedVal},
    engine::{TranslationError, translator::utils::required_cells_for_ty},
    ir::Slot,
};
use alloc::vec::Vec;
use core::{num::NonZero, slice};

/// A [`StackOperand`] or [`Operand`] position on the [`OperandStack`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StackPos(NonZero<usize>);

impl From<StackPos> for usize {
    fn from(value: StackPos) -> Self {
        value.0.get().wrapping_sub(1)
    }
}

impl From<usize> for StackPos {
    fn from(value: usize) -> Self {
        let Some(operand_idx) = NonZero::new(value.wrapping_add(1)) else {
            panic!("out of bounds `StackPos`: {value}")
        };
        Self(operand_idx)
    }
}

/// An [`Operand`] on the [`OperandStack`].
///
/// This is the internal version of [`Operand`] with information that shall remain
/// hidden to the outside.
#[derive(Debug, Copy, Clone)]
pub enum StackOperand {
    /// A local variable.
    Local {
        /// The temporary stack offset of the operand.
        temp_slot: Slot,
        /// The type of the local operand.
        ///
        /// This does not have to be the type of the associated local but
        /// might be a type overwrite. This is useful for Wasm `reinterpret`
        /// operators with local operand inputs.
        ty: ValType,
        /// The index of the local variable.
        local_index: LocalIdx,
        /// The previous [`StackOperand::Local`] on the [`OperandStack`].
        prev_local: Option<StackPos>,
        /// The next [`StackOperand::Local`] on the [`OperandStack`].
        next_local: Option<StackPos>,
    },
    /// A temporary value on the [`OperandStack`].
    Temp {
        /// The temporary stack offset of the operand.
        temp_slot: Slot,
        /// The type of the temporary operand.
        ty: ValType,
    },
    /// An immediate value on the [`OperandStack`].
    Immediate {
        /// The temporary stack offset of the operand.
        temp_slot: Slot,
        /// The type of the immediate operand.
        ty: ValType,
        /// The value of the immediate operand.
        val: UntypedVal,
    },
}

impl StackOperand {
    /// Returns the [`ValType`] of the [`StackOperand`].
    pub fn ty(&self) -> ValType {
        match self {
            Self::Temp { ty, .. } | Self::Immediate { ty, .. } | Self::Local { ty, .. } => *ty,
        }
    }

    /// Returns the temporary [`Slot`] of the [`StackOperand`].
    pub fn temp_slot(&self) -> Slot {
        match self {
            | Self::Temp { temp_slot, .. }
            | Self::Immediate { temp_slot, .. }
            | Self::Local { temp_slot, .. } => *temp_slot,
        }
    }
}

/// The Wasm operand (or value) stack.
#[derive(Debug, Default)]
pub struct OperandStack {
    /// The current set of operands on the [`OperandStack`].
    operands: Vec<StackOperand>,
    /// Stores the first occurrences of every local variable on the [`OperandStack`] if any.
    local_heads: LocalsHead,
    /// The current number of local operands on the `operands` stack.
    ///
    /// This field is required to optimize [`OperandStack::preserve_all_locals`].
    len_locals: usize,
    /// The current top-most temporary stack offset.
    ///
    /// # Note
    ///
    /// - This is used and advanced for the next operand pushed to the stack.
    /// - Upon popping an operand this offset is decreased.
    temp_offset: u16,
    /// The maximum recorded temporary stack offset.
    max_offset: u16,
}

impl Reset for OperandStack {
    fn reset(&mut self) {
        self.operands.clear();
        self.local_heads.reset();
        self.len_locals = 0;
        self.temp_offset = 0;
        self.max_offset = 0;
    }
}

impl OperandStack {
    /// Slot `amount` local variables.
    ///
    /// # Errors
    ///
    /// If too many local variables are being registered.
    pub fn register_locals(&mut self, amount: usize, ty: ValType) -> Result<(), Error> {
        self.local_heads.register(amount)?;
        let cells_per_item = required_cells_for_ty(ty);
        let required_cells = amount
            .checked_mul(usize::from(cells_per_item))
            .ok_or_else(|| Error::from(TranslationError::AllocatedTooManySlots))?;
        self.push_temp_offset(required_cells)?;
        Ok(())
    }

    /// Pushes the offset for temporary operands by `delta`.
    ///
    /// Returns the temporary offset before this operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the new temporary offset is out of bounds.
    fn push_temp_offset(&mut self, delta: usize) -> Result<Slot, Error> {
        let Ok(delta) = u16::try_from(delta) else {
            return Err(Error::from(TranslationError::AllocatedTooManySlots));
        };
        let old_offset = self.temp_offset;
        self.temp_offset = old_offset
            .checked_add(delta)
            .ok_or_else(|| Error::from(TranslationError::AllocatedTooManySlots))?;
        self.max_offset = self.max_offset.max(self.temp_offset);
        Ok(Slot::from(old_offset))
    }

    /// Pops the offset for temporary operands by `delta`.
    ///
    /// # Panics
    ///
    /// If the temporary offset would drop below zero.
    fn pop_temp_offset(&mut self, delta: usize) -> Result<(), Error> {
        let Ok(delta) = u16::try_from(delta) else {
            return Err(Error::from(TranslationError::AllocatedTooManySlots));
        };
        self.temp_offset = self.temp_offset.checked_sub(delta).unwrap_or_else(|| {
            panic!(
                "underflow in `pop_temp_offset`: temp_offset = {}, delta = {delta}",
                self.temp_offset
            )
        });
        Ok(())
    }

    /// Returns the current height of `self`
    ///
    /// # Note
    ///
    /// The height is equal to the number of [`StackOperand`]s on `self`.
    pub fn height(&self) -> usize {
        self.operands.len()
    }

    /// Returns the temporary [`Slot`] allocated for the next pushed operand.
    pub fn next_temp_slot(&self) -> Slot {
        Slot::from(self.temp_offset)
    }

    /// Returns the maximum stack offset of `self`.
    ///
    /// # Note
    ///
    /// This value is equal to the maximum number of cells a function requires to operate.
    pub fn max_stack_offset(&self) -> usize {
        usize::from(self.max_offset)
    }

    /// Returns the [`StackPos`] of the next pushed operand.
    fn next_stack_pos(&self) -> StackPos {
        StackPos::from(self.operands.len())
    }

    /// Returns the [`StackPos`] of the operand at `depth`.
    fn depth_to_stack_pos(&self, depth: usize) -> StackPos {
        StackPos::from(self.height() - depth - 1)
    }

    /// Pushes the [`Operand`] back to the [`OperandStack`].
    ///
    /// Returns the new [`StackPos`].
    ///
    /// # Errors
    ///
    /// - If too many operands have been pushed onto the [`OperandStack`].
    /// - If the local with `local_idx` does not exist.
    #[inline]
    pub fn push_operand(&mut self, operand: Operand) -> Result<Operand, Error> {
        match operand {
            Operand::Local(op) => self
                .push_local(op.local_index(), op.ty())
                .map(Operand::from),
            Operand::Temp(op) => self.push_temp(op.ty()).map(Operand::from),
            Operand::Immediate(op) => self.push_immediate(op.val()).map(Operand::from),
        }
    }

    /// Pushes a local variable with index `local_idx` to the [`OperandStack`].
    ///
    /// # Errors
    ///
    /// - If too many operands have been pushed onto the [`OperandStack`].
    /// - If the local with `local_idx` does not exist.
    #[inline]
    pub fn push_local(
        &mut self,
        local_index: LocalIdx,
        ty: ValType,
    ) -> Result<LocalOperand, Error> {
        let stack_pos = self.next_stack_pos();
        let next_local = self.local_heads.replace_first(local_index, Some(stack_pos));
        if let Some(next_local) = next_local {
            self.update_prev_local(next_local, Some(stack_pos));
        }
        let temp_slot = self.push_temp_offset(usize::from(required_cells_for_ty(ty)))?;
        self.operands.push(StackOperand::Local {
            temp_slot,
            ty,
            local_index,
            prev_local: None,
            next_local,
        });
        self.len_locals += 1;
        Ok(LocalOperand::new(temp_slot, ty, local_index))
    }

    /// Pushes a temporary with type `ty` on the [`OperandStack`].
    ///
    /// # Errors
    ///
    /// If too many operands have been pushed onto the [`OperandStack`].
    #[inline]
    pub fn push_temp(&mut self, ty: ValType) -> Result<TempOperand, Error> {
        let stack_pos = self.next_stack_pos();
        let temp_slot = self.push_temp_offset(usize::from(required_cells_of_type(ty)))?;
        let temp_slot = self.push_temp_offset(usize::from(required_cells_for_ty(ty)))?;
        self.operands.push(StackOperand::Temp { temp_slot, ty });
        Ok(TempOperand::new(temp_slot, ty, stack_pos))
    }

    /// Pushes an immediate `value` on the [`OperandStack`].
    ///
    /// # Errors
    ///
    /// If too many operands have been pushed onto the [`OperandStack`].
    #[inline]
    pub fn push_immediate(
        &mut self,
        value: impl Into<TypedVal>,
    ) -> Result<ImmediateOperand, Error> {
        let value = value.into();
        let ty = value.ty();
        let val = value.untyped();
        let temp_slot = self.push_temp_offset(usize::from(required_cells_for_ty(ty)))?;
        self.operands
            .push(StackOperand::Immediate { temp_slot, ty, val });
        Ok(ImmediateOperand::new(temp_slot, ty, val))
    }

    /// Returns an iterator that yields the last `n` [`Operand`]s.
    ///
    /// # Panics
    ///
    /// If `n` is out of bounds for `self`.
    pub fn peek(&self, n: usize) -> PeekedOperands<'_> {
        let len_operands = self.operands.len();
        let first_index = len_operands - n;
        let Some(operands) = self.operands.get(first_index..) else {
            return PeekedOperands::empty();
        };
        PeekedOperands {
            stack_pos: first_index,
            operands: operands.iter(),
        }
    }

    /// Pops the top-most [`StackOperand`] from `self` if any.
    ///
    /// # Panics
    ///
    /// If `self` is empty.
    #[inline]
    pub fn pop(&mut self) -> Operand {
        let Some(operand) = self.operands.pop() else {
            panic!("tried to pop operand from empty stack");
        };
        self.pop_temp_offset(usize::from(required_cells_for_ty(operand.ty())))
            .unwrap_or_else(|error| panic!("failed to pop temporary offset: {error}"));
        let stack_pos = self.next_stack_pos();
        self.try_unlink_local(operand);
        Operand::new(stack_pos, operand)
    }

    /// Returns the [`Operand`] at `depth`.
    ///
    /// # Panics
    ///
    /// If `depth` is out of bounds for `self`.
    #[inline]
    pub fn get(&self, depth: usize) -> Operand {
        let stack_pos = self.depth_to_stack_pos(depth);
        let operand = self.get_at(stack_pos);
        Operand::new(stack_pos, operand)
    }

    /// Returns the [`StackOperand`] at `index`.
    ///
    /// # Panics
    ///
    /// If `index` is out of bounds for `self`.
    #[inline]
    fn get_at(&self, pos: StackPos) -> StackOperand {
        self.operands[usize::from(pos)]
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
    /// If `depth` is out of bounds for the [`OperandStack`] of operands.
    #[must_use]
    pub fn operand_to_temp(&mut self, depth: usize) -> Operand {
        let stack_pos = self.depth_to_stack_pos(depth);
        let operand = self.operand_to_temp_at(stack_pos);
        Operand::new(stack_pos, operand)
    }

    /// Converts and returns the [`StackOperand`] at `index` into a [`StackOperand::Temp`].
    ///
    /// # Note
    ///
    /// - Returns the [`Operand`] at `index` before being converted to an [`Operand::Temp`].
    /// - [`Operand::Temp`] will have their optional `instr` set to `None`.
    ///
    /// # Panics
    ///
    /// If `index` is out of bounds for `self`.
    #[must_use]
    fn operand_to_temp_at(&mut self, index: StackPos) -> StackOperand {
        let operand = self.get_at(index);
        let temp_slot = operand.temp_slot();
        let ty = operand.ty();
        self.try_unlink_local(operand);
        self.operands[usize::from(index)] = StackOperand::Temp { temp_slot, ty };
        operand
    }

    /// Preserve all locals on the [`OperandStack`] that refer to `local_index`.
    ///
    /// This is done by converting those locals to [`StackOperand::Temp`] and yielding them.
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
        let stack_pos = self.local_heads.replace_first(local_index, None);
        PreservedLocalsIter {
            operands: self,
            stack_pos,
        }
    }

    /// Preserve all locals on the [`OperandStack`].
    ///
    /// This is done by converting those locals to [`StackOperand::Temp`] and yielding them.
    ///
    /// # Note
    ///
    /// The users must fully consume all items yielded by the returned iterator in order
    /// for the local preservation to take full effect.
    #[must_use]
    pub fn preserve_all_locals(&mut self) -> PreservedAllLocalsIter<'_> {
        let index = self.operands.len();
        PreservedAllLocalsIter {
            operands: self,
            stack_pos: index,
        }
    }

    /// Unlinks the [`StackOperand::Local`] `operand` at `index` from `self`.
    ///
    /// Does nothing if `operand` is not a [`StackOperand::Local`].
    #[inline]
    fn try_unlink_local(&mut self, operand: StackOperand) {
        let StackOperand::Local {
            local_index,
            prev_local,
            next_local,
            ..
        } = operand
        else {
            return;
        };
        self.unlink_local(local_index, prev_local, next_local);
    }

    /// Unlinks the [`StackOperand::Local`] `operand` identified by the parameters from `self`.
    fn unlink_local(
        &mut self,
        local_index: LocalIdx,
        prev_local: Option<StackPos>,
        next_local: Option<StackPos>,
    ) {
        if let Some(prev_local) = prev_local {
            self.update_next_local(prev_local, next_local);
        } else {
            self.local_heads.replace_first(local_index, next_local);
        }
        if let Some(next_local) = next_local {
            self.update_prev_local(next_local, prev_local);
        }
        self.len_locals -= 1;
    }

    /// Updates the `prev_local` of the [`StackOperand::Local`] at `local_index` to `prev_index`.
    ///
    /// # Panics
    ///
    /// - If `local_index` does not refer to a [`StackOperand::Local`].
    /// - If `local_index` is out of bounds of the operand stack.
    fn update_prev_local(&mut self, local_pos: StackPos, prev_pos: Option<StackPos>) {
        match self.operands.get_mut(usize::from(local_pos)) {
            Some(StackOperand::Local { prev_local, .. }) => {
                *prev_local = prev_pos;
            }
            entry => panic!("expected `StackOperand::Local` but found: {entry:?}"),
        }
    }

    /// Updates the `next_local` of the [`StackOperand::Local`] at `local_index` to `prev_index`.
    ///
    /// # Panics
    ///
    /// - If `local_index` does not refer to a [`StackOperand::Local`].
    /// - If `local_index` is out of bounds of the operand stack.
    fn update_next_local(&mut self, local_pos: StackPos, next_pos: Option<StackPos>) {
        match self.operands.get_mut(usize::from(local_pos)) {
            Some(StackOperand::Local { next_local, .. }) => {
                *next_local = next_pos;
            }
            entry => panic!("expected `StackOperand::Local` but found: {entry:?}"),
        }
    }
}

/// Iterator yielding preserved local indices while preserving them.
///
/// # Note
///
/// This intentionally iterates backwards from the last pushed stack operand to the first one.
/// Together with the remaining number of local operands on the stack this achieves armortized
/// constant O(1) preservation for all locals via [`OperandStack::preserve_all_locals`].
///
/// The reason for this is that a single call to [`OperandStack::preserve_all_locals`] has the
/// effect that there are no more local operands on the stack. New locals are always pushed to the
/// top of the stack. A single Wasm `local.get` operation (or similar) may only push a single local
/// operand on the stack. This iterator yields once there are no more local operands and since
/// it iterates from the back (top-most) operand it will find the newly inserted locals in
/// armortized constant O(1) time.
#[derive(Debug)]
pub struct PreservedAllLocalsIter<'stack> {
    /// The underlying operand stack.
    operands: &'stack mut OperandStack,
    /// The current operand stack position of the next preserved local if any.
    stack_pos: usize,
}

impl PreservedAllLocalsIter<'_> {
    /// Returns `true` if there are remaining local operands on the stack.
    fn has_remaining_locals(&self) -> bool {
        self.operands.len_locals != 0
    }

    /// Returns the index of the next local operand on the stack if any.
    ///
    /// Returns `None` if there are no more local operands on the stack.
    fn find_next_local(&mut self) -> Option<usize> {
        let mut stack_pos = self.stack_pos;
        loop {
            stack_pos -= 1;
            let opd = self.operands.operands.get(stack_pos)?;
            if let StackOperand::Local { .. } = opd {
                return Some(stack_pos);
            }
        }
    }
}

impl Iterator for PreservedAllLocalsIter<'_> {
    type Item = Operand;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.has_remaining_locals() {
            return None;
        }
        self.stack_pos = self.find_next_local()?;
        let stack_pos = StackPos::from(self.stack_pos);
        let operand = self.operands.operand_to_temp_at(stack_pos);
        debug_assert!(matches!(operand, StackOperand::Local { .. }));
        Some(Operand::new(stack_pos, operand))
    }
}

/// Iterator yielding preserved local indices while preserving them.
#[derive(Debug)]
pub struct PreservedLocalsIter<'stack> {
    /// The underlying operand stack.
    operands: &'stack mut OperandStack,
    /// The current operand stack position of the next preserved local if any.
    stack_pos: Option<StackPos>,
}

impl Iterator for PreservedLocalsIter<'_> {
    type Item = Operand;

    fn next(&mut self) -> Option<Self::Item> {
        let stack_pos = self.stack_pos?;
        let operand = self.operands.operand_to_temp_at(stack_pos);
        self.stack_pos = match operand {
            StackOperand::Local { next_local, .. } => next_local,
            op => panic!("expected `StackOperand::Local` but found: {op:?}"),
        };
        Some(Operand::new(stack_pos, operand))
    }
}

/// Iterator yielding peeked stack operators.
#[derive(Debug)]
pub struct PeekedOperands<'stack> {
    /// The stack position of the next yielded operand.
    stack_pos: usize,
    /// The iterator of peeked stack operands.
    operands: slice::Iter<'stack, StackOperand>,
}

impl<'stack> PeekedOperands<'stack> {
    /// Creates a [`PeekedOperands`] iterator that yields no operands.
    pub fn empty() -> Self {
        Self {
            stack_pos: 0,
            operands: [].iter(),
        }
    }
}

impl Iterator for PeekedOperands<'_> {
    type Item = Operand;

    fn next(&mut self) -> Option<Self::Item> {
        let operand = self.operands.next().copied()?;
        let stack_pos = StackPos::from(self.stack_pos);
        self.stack_pos += 1;
        Some(Operand::new(stack_pos, operand))
    }
}

impl ExactSizeIterator for PeekedOperands<'_> {
    fn len(&self) -> usize {
        self.operands.len()
    }
}
