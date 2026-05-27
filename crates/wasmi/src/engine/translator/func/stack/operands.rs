use super::{
    ImmediateOperand,
    LocalIdx,
    LocalOperand,
    LocalsHead,
    Operand,
    RegOperand,
    Reset,
    TempOperand,
};
use crate::{
    Error,
    ValType,
    core::{RawVal, TypedRawVal},
    engine::{TranslationError, translator::utils::required_cells_for_ty},
    ir::{Slot, SlotSpan},
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
    /// A register operand.
    Reg {
        /// The temporary stack offset of the operand.
        temp_slots: SlotSpan,
        /// The type of the register operand.
        ///
        /// This does not have to be the type of the associated operand but
        /// might be a type overwrite. This is useful for Wasm `reinterpret`
        /// operators with local operand inputs.
        ty: ValType,
    },
    /// A local variable.
    Local {
        /// The temporary stack offset of the operand.
        temp_slots: SlotSpan,
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
        /// This is `true` if the [`StackOperand::Local`] is also stored in a register.
        in_reg: bool,
    },
    /// A temporary value on the [`OperandStack`].
    Temp {
        /// The temporary stack offset of the operand.
        temp_slots: SlotSpan,
        /// The type of the temporary operand.
        ty: ValType,
    },
    /// An immediate value on the [`OperandStack`].
    Immediate {
        /// The temporary stack offset of the operand.
        temp_slots: SlotSpan,
        /// The type of the immediate operand.
        ty: ValType,
        /// The value of the immediate operand.
        val: RawVal,
    },
}

impl StackOperand {
    /// Returns the [`ValType`] of the [`StackOperand`].
    pub fn ty(&self) -> ValType {
        match self {
            Self::Reg { ty, .. }
            | Self::Temp { ty, .. }
            | Self::Immediate { ty, .. }
            | Self::Local { ty, .. } => *ty,
        }
    }

    /// Returns the temporary [`SlotSpan`] of the [`StackOperand`].
    pub fn temp_slots(&self) -> SlotSpan {
        match self {
            | Self::Reg { temp_slots, .. }
            | Self::Temp { temp_slots, .. }
            | Self::Immediate { temp_slots, .. }
            | Self::Local { temp_slots, .. } => *temp_slots,
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
    /// The position of the general purpose (integer) [`Operand::Reg`] on the stack if any.
    ireg: Option<StackPos>,
    /// The position of the `f32` [`Operand::Reg`] on the stack if any.
    freg32: Option<StackPos>,
    /// The position of the `f64` [`Operand::Reg`] on the stack if any.
    freg64: Option<StackPos>,
}

impl Reset for OperandStack {
    fn reset(&mut self) {
        self.operands.clear();
        self.local_heads.reset();
        self.len_locals = 0;
        self.temp_offset = 0;
        self.max_offset = 0;
        self.ireg = None;
        self.freg32 = None;
        self.freg64 = None;
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
        let Ok(amount) = u16::try_from(amount) else {
            return Err(Error::from(TranslationError::TooManyLocalVariables));
        };
        let required_cells = amount
            .checked_mul(cells_per_item)
            .ok_or_else(|| Error::from(TranslationError::AllocatedTooManySlots))?;
        self.push_temp_offset(required_cells)?;
        Ok(())
    }

    /// Replace the typed register operand on the stack with a temporary operand if any.
    ///
    /// Returns `None` if no `reg` operand exists on the stack of type `ty` that needs to be copied.
    ///
    /// # Note
    ///
    /// If the register operand is a register-backed local it is turned into a normal local operand
    /// and `None` is returned as no copy operator is required.
    pub fn reg_to_temp(&mut self, ty: ValType) -> Option<Operand> {
        let pos = self.reg_pos_mut(ty).take()?;
        let index = usize::from(pos);
        let operand = self.get_at(pos);
        let (new_operand, returned) = match operand {
            StackOperand::Reg { temp_slots, ty } => {
                let new_operand = StackOperand::Temp { temp_slots, ty };
                let returned = Some(Operand::new(pos, operand));
                (new_operand, returned)
            }
            StackOperand::Local {
                in_reg: true,
                temp_slots,
                ty,
                local_index,
                prev_local,
                next_local,
            } => {
                let new_operand = StackOperand::Local {
                    in_reg: false,
                    temp_slots,
                    ty,
                    local_index,
                    prev_local,
                    next_local,
                };
                (new_operand, None)
            }
            _ => unreachable!(),
        };
        self.operands[index] = new_operand;
        returned
    }

    /// Pushes the offset for temporary operands by `delta`.
    ///
    /// Returns the temporary offset before this operation.
    ///
    /// # Errors
    ///
    /// Returns an error if the new temporary offset is out of bounds.
    fn push_temp_offset(&mut self, delta: u16) -> Result<SlotSpan, Error> {
        let old_offset = self.temp_offset;
        self.temp_offset = old_offset
            .checked_add(delta)
            .ok_or_else(|| Error::from(TranslationError::AllocatedTooManySlots))?;
        self.max_offset = self.max_offset.max(self.temp_offset);
        Ok(SlotSpan::new(Slot::from(old_offset)))
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
    pub fn next_temp_slots(&self) -> SlotSpan {
        SlotSpan::new(Slot::from(self.temp_offset))
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
            Operand::Reg(op) => self.push_reg(op.ty()).map(Operand::from),
            Operand::Local(op) => self
                .push_local(op.local_index(), op.ty())
                .map(Operand::from),
            Operand::Temp(op) => self.push_temp(op.ty()).map(Operand::from),
            Operand::Immediate(op) => self.push_immediate(op.val()).map(Operand::from),
        }
    }

    /// Pushes a register backed local variable with index `local_idx` to the [`OperandStack`].
    ///
    /// # Errors
    ///
    /// - If too many operands have been pushed onto the [`OperandStack`].
    /// - If the local with `local_idx` does not exist.
    #[inline]
    pub fn push_reg_backed_local(
        &mut self,
        local_index: LocalIdx,
        ty: ValType,
    ) -> Result<LocalOperand, Error> {
        self.push_local_impl(local_index, ty, true)
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
        self.push_local_impl(local_index, ty, false)
    }

    #[inline]
    fn push_local_impl(
        &mut self,
        local_index: LocalIdx,
        ty: ValType,
        in_reg: bool,
    ) -> Result<LocalOperand, Error> {
        let stack_pos = self.next_stack_pos();
        if in_reg {
            self.link_reg(stack_pos, ty);
        }
        let next_local = self.local_heads.replace_first(local_index, Some(stack_pos));
        if let Some(next_local) = next_local {
            self.update_prev_local(next_local, Some(stack_pos));
        }
        let temp_slots = self.push_temp_offset(required_cells_for_ty(ty))?;
        self.operands.push(StackOperand::Local {
            temp_slots,
            ty,
            local_index,
            prev_local: None,
            next_local,
            in_reg,
        });
        self.len_locals += 1;
        Ok(LocalOperand::new(temp_slots, ty, local_index))
    }

    /// Pushes a register operand with type `ty` on the [`OperandStack`].
    ///
    /// # Errors
    ///
    /// If too many operands have been pushed onto the [`OperandStack`].
    #[inline]
    pub fn push_reg(&mut self, ty: ValType) -> Result<RegOperand, Error> {
        let stack_pos = self.next_stack_pos();
        self.link_reg(stack_pos, ty);
        let temp_slots = self.push_temp_offset(required_cells_for_ty(ty))?;
        self.operands.push(StackOperand::Reg { temp_slots, ty });
        Ok(RegOperand::new(temp_slots, ty, stack_pos))
    }

    /// Pushes a temporary with type `ty` on the [`OperandStack`].
    ///
    /// # Errors
    ///
    /// If too many operands have been pushed onto the [`OperandStack`].
    #[inline]
    pub fn push_temp(&mut self, ty: ValType) -> Result<TempOperand, Error> {
        let stack_pos = self.next_stack_pos();
        let temp_slots = self.push_temp_offset(required_cells_for_ty(ty))?;
        self.operands.push(StackOperand::Temp { temp_slots, ty });
        Ok(TempOperand::new(temp_slots, ty, stack_pos))
    }

    /// Pushes an immediate `value` on the [`OperandStack`].
    ///
    /// # Errors
    ///
    /// If too many operands have been pushed onto the [`OperandStack`].
    #[inline]
    pub fn push_immediate(
        &mut self,
        value: impl Into<TypedRawVal>,
    ) -> Result<ImmediateOperand, Error> {
        let value = value.into();
        let ty = value.ty();
        let val = value.raw();
        let temp_slots = self.push_temp_offset(required_cells_for_ty(ty))?;
        self.operands.push(StackOperand::Immediate {
            temp_slots,
            ty,
            val,
        });
        Ok(ImmediateOperand::new(temp_slots, ty, val))
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
        if let Some(reg_pos) = self.try_unlink_reg(operand) {
            debug_assert_eq!(stack_pos, reg_pos);
        }
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
        let temp_slots = operand.temp_slots();
        let ty = operand.ty();
        self.try_unlink_local(operand);
        self.try_unlink_reg(operand);
        self.operands[usize::from(index)] = StackOperand::Temp { temp_slots, ty };
        operand
    }

    fn preserve_local_at(&mut self, pos: StackPos) -> StackOperand {
        let operand = self.get_at(pos);
        self.try_unlink_local(operand);
        let StackOperand::Local {
            temp_slots,
            ty,
            in_reg,
            ..
        } = operand
        else {
            unreachable!()
        };
        let opd = match in_reg {
            true => {
                self.try_unlink_reg(operand);
                StackOperand::Reg { temp_slots, ty }
            }
            false => StackOperand::Temp { temp_slots, ty },
        };
        self.operands[usize::from(pos)] = opd;
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
            stack: self,
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
            stack: self,
            stack_pos: index,
        }
    }
}

#[derive(Copy, Clone)]
pub struct PreservedRegs {
    pub ireg: Option<Slot>,
    pub freg32: Option<Slot>,
    pub freg64: Option<Slot>,
}

impl OperandStack {
    /// Preserve all register operands on the [`OperandStack`].
    ///
    /// This is done by converting those operands to [`StackOperand::Temp`] and
    /// returning their associated [`Slot`] in order to emit copy operators by
    /// the caller.
    #[must_use]
    pub fn preserve_all_regs(&mut self) -> PreservedRegs {
        fn preserve_reg(this: &mut OperandStack, reg: StackPos) -> Slot {
            this.operand_to_temp_at(reg).temp_slots().head()
        }
        let ireg = self.ireg.take().map(|reg| preserve_reg(self, reg));
        let freg32 = self.freg32.take().map(|reg| preserve_reg(self, reg));
        let freg64 = self.freg64.take().map(|reg| preserve_reg(self, reg));
        PreservedRegs {
            ireg,
            freg32,
            freg64,
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

    /// Links the operand at `pos` to the register associated to `ty`.
    ///
    /// # Panics (Debug)
    ///
    /// If the register with `ty` is already occupied.
    fn link_reg(&mut self, pos: StackPos, ty: ValType) {
        let prev_pos = self.reg_pos_mut(ty).replace(pos);
        debug_assert!(
            prev_pos.is_none(),
            "a register operand already exists on the stack",
        );
    }

    /// Unlinks the [`StackOperand::Reg`] `operand` at `index` from `self`.
    ///
    /// Does nothing if `operand` is not a [`StackOperand::Local`].
    #[inline]
    fn try_unlink_reg(&mut self, operand: StackOperand) -> Option<StackPos> {
        let ty = match operand {
            StackOperand::Reg { ty, .. } => ty,
            StackOperand::Local {
                ty, in_reg: true, ..
            } => ty,
            _ => return None,
        };
        self.reg_pos_mut(ty).take()
    }

    /// Returns a `&mut` to the [`StackPos`] of the register operand on the stack if any.
    fn reg_pos_mut(&mut self, ty: ValType) -> &mut Option<StackPos> {
        match ty {
            | ValType::I32 | ValType::I64 | ValType::FuncRef | ValType::ExternRef => &mut self.ireg,
            | ValType::F32 => &mut self.freg32,
            | ValType::F64 => &mut self.freg64,
            | ValType::V128 => unreachable!(),
        }
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
    stack: &'stack mut OperandStack,
    /// The current operand stack position of the next preserved local if any.
    stack_pos: usize,
}

impl PreservedAllLocalsIter<'_> {
    /// Returns `true` if there are remaining local operands on the stack.
    fn has_remaining_locals(&self) -> bool {
        self.stack.len_locals != 0
    }

    /// Returns the index of the next local operand on the stack if any.
    ///
    /// Returns `None` if there are no more local operands on the stack.
    fn find_next_local(&mut self) -> Option<usize> {
        let mut stack_pos = self.stack_pos;
        loop {
            stack_pos -= 1;
            let opd = self.stack.operands.get(stack_pos)?;
            if let StackOperand::Local { .. } = opd {
                return Some(stack_pos);
            }
        }
    }
}

impl Iterator for PreservedAllLocalsIter<'_> {
    type Item = LocalOperand;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if !self.has_remaining_locals() {
                return None;
            }
            self.stack_pos = self.find_next_local()?;
            let stack_pos = StackPos::from(self.stack_pos);
            let operand = self.stack.preserve_local_at(stack_pos);
            let StackOperand::Local {
                temp_slots,
                ty,
                local_index,
                in_reg,
                ..
            } = operand
            else {
                unreachable!("expected `StackOperand::Local` but found: {operand:?}")
            };
            if in_reg {
                continue;
            }
            return Some(LocalOperand::new(temp_slots, ty, local_index));
        }
    }
}

/// Iterator yielding preserved local indices while preserving them.
#[derive(Debug)]
pub struct PreservedLocalsIter<'stack> {
    /// The underlying operand stack.
    stack: &'stack mut OperandStack,
    /// The current operand stack position of the next preserved local if any.
    stack_pos: Option<StackPos>,
}

impl Iterator for PreservedLocalsIter<'_> {
    type Item = LocalOperand;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let stack_pos = self.stack_pos?;
            let operand = self.stack.preserve_local_at(stack_pos);
            let StackOperand::Local {
                temp_slots,
                ty,
                local_index,
                next_local,
                in_reg,
                ..
            } = operand
            else {
                unreachable!("expected `StackOperand::Local` but found: {operand:?}")
            };
            self.stack_pos = next_local;
            if in_reg {
                continue;
            }
            return Some(LocalOperand::new(temp_slots, ty, local_index));
        }
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
