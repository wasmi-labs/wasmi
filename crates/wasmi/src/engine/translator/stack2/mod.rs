#![expect(dead_code)]

mod consts;
mod locals;
mod operand;

pub use self::operand::Operand;
use self::{
    consts::ConstRegistry,
    locals::{LocalIdx, LocalsRegistry},
};
use crate::{
    core::{TypedVal, UntypedVal, ValType},
    engine::{Instr, TranslationError},
    ir::Reg,
    Error,
};
use alloc::vec::Vec;
use core::{array, mem, num::NonZero};

/// The Wasm value stack during translation from Wasm to Wasmi bytecode.
#[derive(Debug, Default)]
pub struct Stack {
    /// The stack of operands.
    operands: Vec<StackOperand>,
    /// All function locals and their associated types.
    locals: LocalsRegistry,
    /// All function local constants.
    consts: ConstRegistry,
    /// The index of the first [`StackOperand::Local`] on the [`Stack`].
    max_height: usize,
}

impl Stack {
    /// Resets the [`Stack`] for reuse.
    pub fn reset(&mut self) {
        self.operands.clear();
        self.locals.reset();
        self.consts.reset();
        self.max_height = 0;
    }

    /// Register `amount` local variables of common type `ty`.
    ///
    /// # Errors
    ///
    /// If too many local variables are being registered.
    pub fn register_locals(&mut self, amount: u32, ty: ValType) -> Result<(), Error> {
        self.locals.register(amount, ty)?;
        Ok(())
    }

    /// Returns the current height of the [`Stack`].
    ///
    /// # Note
    ///
    /// The height is equal to the number of [`Operand`]s on the [`Stack`].
    pub fn height(&self) -> usize {
        self.operands.len()
    }

    /// Returns the maximum height of the [`Stack`].
    ///
    /// # Note
    ///
    /// The height is equal to the number of [`Operand`]s on the [`Stack`].
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
    fn next_operand_idx(&self) -> OperandIdx {
        OperandIdx::from(self.operands.len())
    }

    /// Updates the `prev_local` of the [`StackOperand::Local`] at `local_index` to `prev_index`.
    ///
    /// # Panics
    ///
    /// - If `local_index` does not refer to a [`StackOperand::Local`].
    /// - If `local_index` is out of bounds of the operand stack.
    fn update_prev_local(&mut self, local_index: OperandIdx, prev_index: Option<OperandIdx>) {
        match self.operands.get_mut(usize::from(local_index)) {
            Some(StackOperand::Local { prev_local, .. }) => {
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
        match self.operands.get_mut(usize::from(local_index)) {
            Some(StackOperand::Local { next_local, .. }) => {
                *next_local = prev_index;
            }
            operand => panic!("expected `StackOperand::Local` but found: {operand:?}"),
        }
    }

    /// Pushes a local variable with index `local_idx` to the [`Stack`].
    ///
    /// # Errors
    ///
    /// - If too many operands have been pushed onto the [`Stack`].
    /// - If the local with `local_idx` does not exist.
    pub fn push_local(&mut self, local_index: LocalIdx) -> Result<OperandIdx, Error> {
        let operand_index = self.next_operand_idx();
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

    /// Pushes a temporary with type `ty` on the [`Stack`].
    ///
    /// # Errors
    ///
    /// If too many operands have been pushed onto the [`Stack`].
    pub fn push_temp(&mut self, ty: ValType, instr: Option<Instr>) -> Result<OperandIdx, Error> {
        let operand_index = self.next_operand_idx();
        self.operands.push(StackOperand::Temp { ty, instr });
        self.update_max_stack_height();
        Ok(operand_index)
    }

    /// Pushes an immediate `value` on the [`Stack`].
    ///
    /// # Errors
    ///
    /// If too many operands have been pushed onto the [`Stack`].
    pub fn push_immediate(&mut self, value: impl Into<TypedVal>) -> Result<OperandIdx, Error> {
        let operand_index = self.next_operand_idx();
        self.operands
            .push(StackOperand::Immediate { val: value.into() });
        self.update_max_stack_height();
        Ok(operand_index)
    }

    /// Peeks the top-most [`Operand`] on the [`Stack`].
    ///
    /// Returns `None` if the [`Stack`] is empty.
    pub fn peek(&self) -> Option<Operand> {
        let operand = self.operands.last().copied()?;
        let index = OperandIdx::from(self.operands.len() - 1);
        Some(Operand::new(index, operand, &self.locals))
    }

    /// Pops the top-most [`Operand`] from the [`Stack`].
    ///
    /// Returns `None` if the [`Stack`] is empty.
    pub fn pop(&mut self) -> Option<Operand> {
        let operand = self.operands.pop()?;
        let index = OperandIdx::from(self.operands.len());
        Some(Operand::new(index, operand, &self.locals))
    }

    /// Pops the two top-most [`Operand`] from the [`Stack`].
    ///
    /// - Returns `None` if the [`Stack`] is empty.
    /// - The last returned [`Operand`] is the top-most one.
    pub fn pop2(&mut self) -> Option<(Operand, Operand)> {
        let [o1, o2] = self.pop_some::<2>()?;
        Some((o1, o2))
    }

    /// Pops the three top-most [`Operand`] from the [`Stack`].
    ///
    /// - Returns `None` if the [`Stack`] is empty.
    /// - The last returned [`Operand`] is the top-most one.
    pub fn pop3(&mut self) -> Option<(Operand, Operand, Operand)> {
        let [o1, o2, o3] = self.pop_some::<3>()?;
        Some((o1, o2, o3))
    }

    /// Pops the top-most `N` [`Operand`]s from the [`Stack`].
    ///
    /// - Returns `None` if the [`Stack`] is empty.
    /// - The last returned [`Operand`] is the top-most one.
    fn pop_some<const N: usize>(&mut self) -> Option<[Operand; N]> {
        if N >= self.height() {
            return None;
        }
        let start = self.height() - N;
        let drained = self.operands.drain(start..);
        let popped: [Operand; N] = array::from_fn(|i| {
            let index = OperandIdx::from(start + i);
            let operand = drained.as_slice()[i];
            Operand::new(index, operand, &self.locals)
        });
        Some(popped)
    }

    /// Returns the [`RegSpace`] of the [`Reg`].
    ///
    /// Returns `None` if the [`Reg`] is unknown to the [`Stack`].
    #[must_use]
    pub fn stack_space(&self, reg: Reg) -> RegSpace {
        let index = i16::from(reg);
        if index.is_negative() {
            return RegSpace::Const;
        }
        let index = index as u16;
        if usize::from(index) < self.locals.len() {
            return RegSpace::Local;
        }
        RegSpace::Temp
    }

    /// Preserve all locals on the [`Stack`] that refer to `local_index`.
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
        let operands = &mut self.operands[..];
        PreservedLocalsIter {
            operands,
            index,
            ty,
        }
    }

    /// Converts and returns the [`StackOperand`] at `depth` into a [`StackOperand::Temp`].
    ///
    /// # Note
    ///
    /// Returns `None` if operand at `depth` is [`StackOperand::Temp`] already.
    ///
    /// # Panics
    ///
    /// - If `depth` is out of bounds for the [`Stack`] of operands.
    #[must_use]
    pub fn operand_to_temp(&mut self, depth: usize) -> Option<Operand> {
        let len = self.height();
        if depth >= len {
            panic!(
                "out of bounds access: tried to access `Stack` with length {len} at depth {depth}"
            );
        }
        let index = len - depth - 1;
        let operand_index = OperandIdx::from(index);
        let operand = match self.operands[index] {
            StackOperand::Local {
                local_index,
                prev_local,
                next_local,
            } => {
                if prev_local.is_none() {
                    // Note: if `prev_local` is `None` then this local is the first
                    //       in the linked list of locals and must be updated.
                    debug_assert_eq!(self.locals.first_operand(local_index), Some(operand_index));
                    self.locals.replace_first_operand(local_index, next_local);
                }
                if let Some(prev_local) = prev_local {
                    self.update_next_local(prev_local, next_local);
                }
                if let Some(next_local) = next_local {
                    self.update_prev_local(next_local, prev_local);
                }
                Operand::local(operand_index, local_index, &self.locals)
            }
            StackOperand::Immediate { val } => Operand::immediate(operand_index, val),
            StackOperand::Temp { .. } => return None,
        };
        self.operands[index] = StackOperand::Temp {
            ty: operand.ty(),
            instr: None,
        };
        Some(operand)
    }

    /// Converts the [`Operand`] at `index` to a [`Reg`] if possible.
    ///
    /// # Panics
    ///
    /// If the `index` is out of bounds.
    pub fn operand_to_reg(&mut self, depth: usize) -> Result<Reg, Error> {
        let len = self.height();
        if depth >= len {
            panic!(
                "out of bounds access: tried to access `Stack` with length {len} at depth {depth}"
            );
        }
        let index = len - depth - 1;
        let operand = self.operands[index];
        match operand {
            StackOperand::Local { local_index, .. } => self.local_to_reg(local_index),
            StackOperand::Temp { .. } => self.temp_to_reg(OperandIdx::from(index)),
            StackOperand::Immediate { val } => self.const_to_reg(val),
        }
    }

    /// Allocates a function local constant `value`.
    ///
    /// # Errors
    ///
    /// If too many function local constants have been allocated already.
    pub fn const_to_reg(&mut self, value: impl Into<UntypedVal>) -> Result<Reg, Error> {
        self.consts.alloc(value.into())
    }

    /// Converts the local `index` into the associated [`Reg`].
    ///
    /// # Errors
    ///
    /// If `index` cannot be converted into a [`Reg`].
    fn local_to_reg(&self, index: LocalIdx) -> Result<Reg, Error> {
        let Ok(index) = i16::try_from(u32::from(index)) else {
            return Err(Error::from(TranslationError::AllocatedTooManyRegisters));
        };
        Ok(Reg::from(index))
    }

    /// Converts the [`Operand::Temp`] `index` into the associated [`Reg`].
    ///
    /// # Errors
    ///
    /// If `index` cannot be converted into a [`Reg`].
    pub fn temp_to_reg(&self, index: OperandIdx) -> Result<Reg, Error> {
        let index = usize::from(index);
        debug_assert!(matches!(&self.operands[index], StackOperand::Temp { .. },));
        let Some(index) = index.checked_add(self.locals.len()) else {
            return Err(Error::from(TranslationError::AllocatedTooManyRegisters));
        };
        let Ok(index) = i16::try_from(index) else {
            return Err(Error::from(TranslationError::AllocatedTooManyRegisters));
        };
        Ok(Reg::from(index))
    }
}

/// Iterator yielding preserved local indices while preserving them.
#[derive(Debug)]
pub struct PreservedLocalsIter<'stack> {
    /// The underlying operand stack.
    operands: &'stack mut [StackOperand],
    /// The current operand index of the next preserved local if any.
    index: Option<OperandIdx>,
    /// Type of local at preserved `local_index`.
    ty: ValType,
}

impl Iterator for PreservedLocalsIter<'_> {
    type Item = OperandIdx;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index?;
        let operand = mem::replace(
            &mut self.operands[usize::from(index)],
            StackOperand::Temp {
                ty: self.ty,
                instr: None,
            },
        );
        self.index = match operand {
            StackOperand::Local { next_local, .. } => next_local,
            operand => panic!("expected `StackOperand::Local` but found: {operand:?}"),
        };
        Some(index)
    }
}

/// The [`RegSpace`] of a [`Reg`].
#[derive(Debug, Copy, Clone)]
pub enum RegSpace {
    /// Stack slot referring to a local variable.
    Local,
    /// Stack slot referring to a function local constant value.
    Const,
    /// Stack slot referring to a temporary stack operand.
    Temp,
}

impl RegSpace {
    /// Returns `true` if `self` is [`RegSpace::Local`].
    #[inline]
    pub fn is_local(self) -> bool {
        matches!(self, Self::Local)
    }

    /// Returns `true` if `self` is [`RegSpace::Temp`].
    #[inline]
    pub fn is_temp(self) -> bool {
        matches!(self, Self::Temp)
    }

    /// Returns `true` if `self` is [`RegSpace::Const`].
    #[inline]
    pub fn is_const(self) -> bool {
        matches!(self, Self::Const)
    }
}

/// A [`StackOperand`] or [`Operand`] index on the [`Stack`].
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

/// An [`Operand`] on the [`Stack`].
///
/// This is the internal version of [`Operand`] with information that shall remain
/// hidden to the outside.
#[derive(Debug, Copy, Clone)]
enum StackOperand {
    /// A local variable.
    Local {
        /// The index of the local variable.
        local_index: LocalIdx,
        /// The previous [`StackOperand::Local`] on the [`Stack`].
        prev_local: Option<OperandIdx>,
        /// The next [`StackOperand::Local`] on the [`Stack`].
        next_local: Option<OperandIdx>,
    },
    /// A temporary value on the [`Stack`].
    Temp {
        /// The type of the temporary value.
        ty: ValType,
        /// The instruction which has this [`StackOperand`] as result if any.
        instr: Option<Instr>,
    },
    /// An immediate value on the [`Stack`].
    Immediate {
        /// The value (and type) of the immediate value.
        val: TypedVal,
    },
}
