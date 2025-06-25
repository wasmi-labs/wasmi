use super::{LocalIdx, LocalsRegistry, Operand, Reset};
use crate::{
    core::{TypedVal, ValType},
    engine::translator::utils::Instr,
    Error,
};
use alloc::vec::Vec;
use core::{num::NonZero, slice};

/// A [`StackOperand`] or [`Operand`] index on the [`OperandStack`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OperandIdx(NonZero<usize>);

impl From<OperandIdx> for usize {
    fn from(value: OperandIdx) -> Self {
        value.0.get().wrapping_sub(1)
    }
}

impl From<usize> for OperandIdx {
    fn from(value: usize) -> Self {
        let Some(operand_idx) = NonZero::new(value.wrapping_add(1)) else {
            panic!("out of bounds `OperandIdx`: {value}")
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
        /// The index of the local variable.
        local_index: LocalIdx,
        /// The previous [`StackOperand::Local`] on the [`OperandStack`].
        prev_local: Option<OperandIdx>,
        /// The next [`StackOperand::Local`] on the [`OperandStack`].
        next_local: Option<OperandIdx>,
    },
    /// A temporary value on the [`OperandStack`].
    Temp {
        /// The type of the temporary value.
        ty: ValType,
        /// The instruction which has this [`StackOperand`] as result if any.
        instr: Option<Instr>,
    },
    /// An immediate value on the [`OperandStack`].
    Immediate {
        /// The value (and type) of the immediate value.
        val: TypedVal,
    },
}

impl StackOperand {
    /// Returns the [`ValType`] of the [`StackOperand`].
    pub fn ty(&self, locals: &LocalsRegistry) -> ValType {
        match self {
            StackOperand::Temp { ty, .. } => *ty,
            StackOperand::Immediate { val } => val.ty(),
            StackOperand::Local { local_index, .. } => locals.ty(*local_index),
        }
    }
}

/// The Wasm operand (or value) stack.
#[derive(Debug, Default)]
pub struct OperandStack {
    /// The current set of operands on the [`OperandStack`].
    operands: Vec<StackOperand>,
    /// All function locals and their associated types.
    ///
    /// Used to query types of locals and their first local on the [`OperandStack`].
    locals: LocalsRegistry,
    /// The maximum height of the [`OperandStack`].
    max_height: usize,
}

impl Reset for OperandStack {
    fn reset(&mut self) {
        self.operands.clear();
        self.locals.reset();
        self.max_height = 0;
    }
}

impl OperandStack {
    /// Register `amount` local variables of common type `ty`.
    ///
    /// # Errors
    ///
    /// If too many local variables are being registered.
    pub fn register_locals(&mut self, amount: u32, ty: ValType) -> Result<(), Error> {
        self.locals.register(amount, ty)?;
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

    /// Returns the maximum height of `self`.
    ///
    /// # Note
    ///
    /// The height is equal to the number of [`Operand`]s on `self`.
    pub fn max_height(&self) -> usize {
        self.max_height
    }

    /// Truncates `self` to the target `height`.
    ///
    /// All operands above `height` are dropped.
    ///
    /// # Panic
    ///
    /// If `height` is greater than the current height of `self`.
    pub fn trunc(&mut self, height: usize) {
        assert!(height <= self.height());
        self.operands.truncate(height);
    }

    /// Updates the maximum stack height if needed.
    fn update_max_stack_height(&mut self) {
        self.max_height = core::cmp::max(self.max_height, self.height());
    }

    /// Returns the [`OperandIdx`] of the next pushed operand.
    fn next_index(&self) -> OperandIdx {
        OperandIdx::from(self.operands.len())
    }

    /// Returns the [`OperandIdx`] of the operand at `depth`.
    fn depth_to_index(&self, depth: usize) -> OperandIdx {
        OperandIdx::from(self.height() - depth - 1)
    }

    /// Pushes a local variable with index `local_idx` to the [`OperandStack`].
    ///
    /// # Errors
    ///
    /// - If too many operands have been pushed onto the [`OperandStack`].
    /// - If the local with `local_idx` does not exist.
    pub fn push_local(&mut self, local_index: LocalIdx) -> Result<OperandIdx, Error> {
        let operand_index = self.next_index();
        let next_local = self
            .locals
            .replace_first_operand(local_index, Some(operand_index));
        if let Some(next_local) = next_local {
            self.update_prev_local(next_local, Some(operand_index));
        }
        self.operands.push(StackOperand::Local {
            local_index,
            prev_local: None,
            next_local,
        });
        self.update_max_stack_height();
        Ok(operand_index)
    }

    /// Pushes a temporary with type `ty` on the [`OperandStack`].
    ///
    /// # Errors
    ///
    /// If too many operands have been pushed onto the [`OperandStack`].
    pub fn push_temp(&mut self, ty: ValType, instr: Option<Instr>) -> Result<OperandIdx, Error> {
        let idx = self.next_index();
        self.operands.push(StackOperand::Temp { ty, instr });
        self.update_max_stack_height();
        Ok(idx)
    }

    /// Pushes an immediate `value` on the [`OperandStack`].
    ///
    /// # Errors
    ///
    /// If too many operands have been pushed onto the [`OperandStack`].
    pub fn push_immediate(&mut self, value: impl Into<TypedVal>) -> Result<OperandIdx, Error> {
        let idx = self.next_index();
        self.operands
            .push(StackOperand::Immediate { val: value.into() });
        self.update_max_stack_height();
        Ok(idx)
    }

    /// Returns an iterator that yields all [`Operand`]s up to `depth`.
    ///
    /// # Panics
    ///
    /// If `depth` is out of bounds for `self`.
    pub fn peek(&self, depth: usize) -> PeekedOperands {
        let index = self.depth_to_index(depth);
        let operands = &self.operands[usize::from(index)..];
        PeekedOperands {
            index: usize::from(index),
            operands: operands.iter(),
            locals: &self.locals,
        }
    }

    /// Pops the top-most [`StackOperand`] from `self` if any.
    ///
    /// # Panics
    ///
    /// If `self` is empty.
    pub fn pop(&mut self) -> Operand {
        let Some(operand) = self.operands.pop() else {
            panic!("tried to pop operand from empty stack");
        };
        let index = self.next_index();
        self.unlink_local(operand);
        Operand::new(index, operand, &self.locals)
    }

    /// Returns the [`Operand`] at `depth`.
    ///
    /// # Panics
    ///
    /// If `depth` is out of bounds for `self`.
    pub fn get(&self, depth: usize) -> Operand {
        let index = self.depth_to_index(depth);
        let operand = self.get_at(index);
        Operand::new(index, operand, &self.locals)
    }

    /// Returns the [`StackOperand`] at `index`.
    ///
    /// # Panics
    ///
    /// If `index` is out of bounds for `self`.
    fn get_at(&self, index: OperandIdx) -> StackOperand {
        self.operands[usize::from(index)]
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
        let index = self.depth_to_index(depth);
        let operand = self.operand_to_temp_at(index);
        Operand::new(index, operand, &self.locals)
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
    fn operand_to_temp_at(&mut self, index: OperandIdx) -> StackOperand {
        let operand = self.get_at(index);
        let ty = operand.ty(&self.locals);
        self.unlink_local(operand);
        self.operands[usize::from(index)] = StackOperand::Temp { ty, instr: None };
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
    pub fn preserve_locals(&mut self, local_index: LocalIdx) -> PreservedLocalsIter {
        let ty = self.locals.ty(local_index);
        let index = self.locals.replace_first_operand(local_index, None);
        PreservedLocalsIter {
            operands: self,
            index,
            ty,
        }
    }

    /// Unlinks the [`StackOperand::Local`] `operand` at `index` from `self`.
    ///
    /// Does nothing if `operand` is not a [`StackOperand::Local`].
    fn unlink_local(&mut self, operand: StackOperand) {
        let StackOperand::Local {
            local_index,
            prev_local,
            next_local,
        } = operand
        else {
            return;
        };
        if prev_local.is_none() {
            self.locals.replace_first_operand(local_index, next_local);
        }
        if let Some(prev_local) = prev_local {
            self.update_next_local(prev_local, next_local);
        }
        if let Some(next_local) = next_local {
            self.update_prev_local(next_local, prev_local);
        }
    }

    /// Updates the `prev_local` of the [`StackOperand::Local`] at `local_index` to `prev_index`.
    ///
    /// # Panics
    ///
    /// - If `local_index` does not refer to a [`StackOperand::Local`].
    /// - If `local_index` is out of bounds of the operand stack.
    fn update_prev_local(&mut self, local_index: OperandIdx, prev_index: Option<OperandIdx>) {
        match &mut self.operands[usize::from(local_index)] {
            StackOperand::Local { prev_local, .. } => {
                *prev_local = prev_index;
            }
            operand => panic!("expected `StackOperand::Local` but found: {operand:?}"),
        }
    }

    /// Updates the `next_local` of the [`StackOperand::Local`] at `local_index` to `prev_index`.
    ///
    /// # Panics
    ///
    /// - If `local_index` does not refer to a [`StackOperand::Local`].
    /// - If `local_index` is out of bounds of the operand stack.
    fn update_next_local(&mut self, local_index: OperandIdx, prev_index: Option<OperandIdx>) {
        match &mut self.operands[usize::from(local_index)] {
            StackOperand::Local { next_local, .. } => {
                *next_local = prev_index;
            }
            operand => panic!("expected `StackOperand::Local` but found: {operand:?}"),
        }
    }
}

/// Iterator yielding preserved local indices while preserving them.
#[derive(Debug)]
pub struct PreservedLocalsIter<'stack> {
    /// The underlying operand stack.
    operands: &'stack mut OperandStack,
    /// The current operand index of the next preserved local if any.
    index: Option<OperandIdx>,
    /// Type of local at preserved `local_index`.
    ty: ValType,
}

impl Iterator for PreservedLocalsIter<'_> {
    type Item = OperandIdx;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index?;
        let operand = self.operands.operand_to_temp_at(index);
        self.index = match operand {
            StackOperand::Local { next_local, .. } => next_local,
            op => panic!("expected `StackOperand::Local` but found: {op:?}"),
        };
        Some(index)
    }
}

/// Iterator yielding peeked stack operators.
#[derive(Debug)]
pub struct PeekedOperands<'stack> {
    /// The index of the next yielded operand.
    index: usize,
    /// The iterator of peeked stack operands.
    operands: slice::Iter<'stack, StackOperand>,
    /// Used to query types of local operands.
    locals: &'stack LocalsRegistry,
}

impl Iterator for PeekedOperands<'_> {
    type Item = Operand;

    fn next(&mut self) -> Option<Self::Item> {
        let operand = self.operands.next().copied()?;
        let index = OperandIdx::from(self.index);
        self.index += 1;
        Some(Operand::new(index, operand, self.locals))
    }
}

impl ExactSizeIterator for PeekedOperands<'_> {
    fn len(&self) -> usize {
        self.operands.len()
    }
}
