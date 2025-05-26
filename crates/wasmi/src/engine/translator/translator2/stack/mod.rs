mod consts;
mod control;
mod layout;
mod locals;
mod operand;

use self::{
    consts::ConstRegistry,
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
    locals::{LocalIdx, LocalsRegistry},
};
pub use self::{
    layout::{StackLayout, StackSpace},
    operand::Operand,
};
use crate::{
    core::{TypedVal, UntypedVal, ValType},
    engine::{
        translator::{BlockType, LabelRef},
        Instr,
        TranslationError,
    },
    ir::Reg,
    Engine,
    Error,
};
use alloc::vec::Vec;
use core::{array, mem, num::NonZero};

/// Implemented by types that can be reset for reuse.
trait Reset {
    /// Resets `self` for reuse.
    fn reset(&mut self);
}

/// The Wasm value stack during translation from Wasm to Wasmi bytecode.
#[derive(Debug, Default)]
pub struct Stack {
    /// The underlying [`Engine`].
    engine: Engine,
    /// The Wasm value stack.
    operands: Vec<StackOperand>,
    /// All function locals and their associated types.
    locals: LocalsRegistry,
    /// The Wasm control stack.
    controls: ControlStack,
    /// The maximim number of operands on the [`Stack`] at the same time.
    max_height: usize,
}

impl Stack {
    /// Resets the [`Stack`] for reuse.
    pub fn reset(&mut self) {
        self.operands.clear();
        self.locals.reset();
        self.controls.reset();
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
    fn next_operand_index(&self) -> OperandIdx {
        OperandIdx::from(self.operands.len())
    }

    /// Returns the [`OperandIdx`] of the operand at `depth`.
    fn operand_index(&self, depth: usize) -> OperandIdx {
        OperandIdx::from(self.height() - depth - 1)
    }

    /// Updates the `prev_local` of the [`StackOperand::Local`] at `local_index` to `prev_index`.
    ///
    /// # Panics
    ///
    /// - If `local_index` does not refer to a [`StackOperand::Local`].
    /// - If `local_index` is out of bounds of the operand stack.
    fn update_prev_local(&mut self, local_index: OperandIdx, prev_index: Option<OperandIdx>) {
        match self.get_mut_at(local_index) {
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
        match self.get_mut_at(local_index) {
            StackOperand::Local { next_local, .. } => {
                *next_local = prev_index;
            }
            operand => panic!("expected `StackOperand::Local` but found: {operand:?}"),
        }
    }

    /// Pushes a Wasm `block` onto the [`Stack`].
    ///
    /// # Note
    ///
    /// If `consume_fuel` is `None` and fuel metering is enabled this will infer
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
        let fuel_metering = self.engine.config().get_consume_fuel();
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
    /// Calls `f` for every non [`StackOperand::Temp`] operand on the [`Stack`]
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
        debug_assert!(self.engine.config().get_consume_fuel() == consume_fuel.is_some());
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
        debug_assert!(self.engine.config().get_consume_fuel() == consume_fuel.is_some());
        let len_params = usize::from(ty.len_params(&self.engine));
        let block_height = self.height() - len_params;
        let else_operands = self.operands.get(block_height..).unwrap_or(&[]);
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
        debug_assert!(self.engine.config().get_consume_fuel() == consume_fuel.is_some());
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
            // TODO: push `operand` to stack, resolve borrow issues
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
        let operand_index = self.next_operand_index();
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
        let operand_index = self.next_operand_index();
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
        let operand_index = self.next_operand_index();
        self.operands
            .push(StackOperand::Immediate { val: value.into() });
        self.update_max_stack_height();
        Ok(operand_index)
    }

    /// Peeks the top-most [`Operand`] on the [`Stack`].
    ///
    /// Returns `None` if the [`Stack`] is empty.
    pub fn peek(&self) -> Operand {
        let index = self.operand_index(0);
        let operand = self.get_at(index);
        Operand::new(index, operand, &self.locals)
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

    /// Returns the [`StackOperand`] at `index`.
    ///
    /// # Panics
    ///
    /// If `depth` is out of bounds for `self`.
    fn get_at(&self, index: OperandIdx) -> StackOperand {
        self.operands[usize::from(index)]
    }

    /// Returns an exlusive reference to the [`StackOperand`] at `index`.
    ///
    /// # Panics
    ///
    /// If `depth` is out of bounds for `self`.
    fn get_mut_at(&mut self, index: OperandIdx) -> &mut StackOperand {
        &mut self.operands[usize::from(index)]
    }

    /// Sets the [`StackOperand`] at `index` to `operand`.
    ///
    /// # Panics
    ///
    /// If `depth` is out of bounds for `self`.
    fn set_at(&mut self, index: OperandIdx, operand: StackOperand) {
        self.operands[usize::from(index)] = operand;
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
        let index = self.operand_index(depth);
        let operand = match self.get_at(index) {
            StackOperand::Local {
                local_index,
                prev_local,
                next_local,
            } => {
                if prev_local.is_none() {
                    // Note: if `prev_local` is `None` then this local is the first
                    //       in the linked list of locals and must be updated.
                    debug_assert_eq!(self.locals.first_operand(local_index), Some(index));
                    self.locals.replace_first_operand(local_index, next_local);
                }
                if let Some(prev_local) = prev_local {
                    self.update_next_local(prev_local, next_local);
                }
                if let Some(next_local) = next_local {
                    self.update_prev_local(next_local, prev_local);
                }
                Operand::local(index, local_index, &self.locals)
            }
            StackOperand::Immediate { val } => Operand::immediate(index, val),
            StackOperand::Temp { .. } => return None,
        };
        self.set_at(
            index,
            StackOperand::Temp {
                ty: operand.ty(),
                instr: None,
            },
        );
        Some(operand)
    }

    /// Converts the [`Operand`] at `index` to a [`Reg`] if possible.
    ///
    /// # Panics
    ///
    /// If the `index` is out of bounds.
    pub fn operand_to_reg(&mut self, depth: usize, layout: &mut StackLayout) -> Result<Reg, Error> {
        let index = self.operand_index(depth);
        match self.get_at(index) {
            StackOperand::Local { local_index, .. } => layout.local_to_reg(local_index),
            StackOperand::Temp { .. } => layout.temp_to_reg(index),
            StackOperand::Immediate { val } => layout.const_to_reg(val),
        }
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
