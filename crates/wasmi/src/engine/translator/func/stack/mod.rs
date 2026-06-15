mod control;
mod locals;
mod operand;
mod operands;

use self::{
    control::ControlStack,
    locals::LocalsHead,
    operands::{OperandStack, StackOperand, StackPos},
};
pub use self::{
    control::{
        AcquiredTarget,
        BlockControlFrame,
        BranchParamRegs,
        BranchParams,
        ControlFrame,
        ControlFrameBase,
        ControlFrameKind,
        ElseControlFrame,
        ElseReachability,
        IfControlFrame,
        IfReachability,
        LoopControlFrame,
        RegKind,
    },
    operand::{ImmediateOperand, LocalOperand, Location, Operand, ResolvedOperand, TempOperand},
    operands::{Allocation, PreservedAllLocalsIter, PreservedLocalsIter, PreservedRegs},
};
use super::{Reset, ReusableAllocations};
use crate::{
    Engine,
    Error,
    FuncType,
    ValType,
    core::TypedRawVal,
    engine::{
        BlockType,
        translator::func::{LocalIdx, Pos, labels::LabelRef, stack::operands::PeekedOperands},
    },
    ir::{self, SlotSpan},
};

#[cfg(doc)]
use crate::ir::Op;

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

    /// Slot `amount` local variables.
    ///
    /// # Errors
    ///
    /// If too many local variables are being registered.
    pub fn register_locals(&mut self, amount: usize, ty: ValType) -> Result<(), Error> {
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

    /// Returns the maximum stack offset of the [`Stack`].
    ///
    /// # Note
    ///
    /// This value is equal to the maximum number of cells a function requires to operate.
    pub fn max_stack_offset(&self) -> usize {
        self.operands.max_stack_offset()
    }

    /// Returns the next temporary [`SlotSpan`] if an operand was pushed to `self`.
    pub fn next_temp_slots(&self) -> SlotSpan {
        self.operands.next_temp_slots()
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

    /// Pushes the branch params of the control `frame` onto the [`Stack`].
    ///
    /// # Note
    ///
    /// - Before pushing the results, the [`Stack`] is truncated to the `frame`'s height.
    /// - Not all control frames have temporary results, e.g. Wasm `loop`s, Wasm `if`s with
    ///   a compile-time known branch or Wasm `block`s that are never branched to, do not
    ///   require to call this function.
    pub fn push_branch_params(&mut self, frame: &impl ControlFrameBase) -> Result<(), Error> {
        let height = frame.height();
        self.discard_local_regs();
        self.trunc(height);
        let kind = frame.kind();
        let params = frame.branch_params();
        let len_temps = usize::from(params.len_temps());
        frame
            .ty()
            .func_type_with(&self.engine, |func_ty| -> Result<(), Error> {
                let params = match kind {
                    ControlFrameKind::Loop => func_ty.params(),
                    _ => func_ty.results(),
                };
                for (n, result) in params.iter().enumerate() {
                    let alloc = match n < len_temps {
                        true => Allocation::None,
                        false => Allocation::Reg,
                    };
                    self.operands.push_temp(*result, alloc)?;
                }
                Ok(())
            })?;
        Ok(())
    }

    /// Returns the branch slots for the control frame with `len_params` operand parameters.
    fn branch_slots(&self, len_params: usize) -> SlotSpan {
        match len_params {
            0 => self.operands.next_temp_slots(),
            _ => self.operands.get(len_params - 1).temp_slots().span(),
        }
    }

    /// Creates [`BranchParamRegs`] from `tys` if any.
    ///
    /// Returns `None` if `tys` is empty.
    fn branch_params_regs(tys: &[ValType]) -> Option<BranchParamRegs> {
        let regs = match tys {
            [] => return None,
            [ty] => {
                let kind = RegKind::new(*ty)?;
                BranchParamRegs::new_one(kind)
            }
            [.., last2, last1, last0] => {
                let [kind2, kind1, kind0] = [*last2, *last1, *last0].map(RegKind::new);
                let kind0 = kind0?;
                let Some(kind1) = kind1 else {
                    return Some(BranchParamRegs::new_one(kind0));
                };
                let Some(kind2) = kind2 else {
                    match kind0 != kind1 {
                        true => return Some(BranchParamRegs::new_two([kind0, kind1])),
                        false => return Some(BranchParamRegs::new_one(kind0)),
                    }
                };
                if kind0 == kind1 {
                    return Some(BranchParamRegs::new_one(kind0));
                }
                if kind0 == kind2 || kind1 == kind2 {
                    return Some(BranchParamRegs::new_two([kind0, kind1]));
                }
                match kind0 != kind2 && kind1 != kind2 {
                    true => BranchParamRegs::new_three([kind0, kind1, kind2]),
                    false => BranchParamRegs::new_two([kind0, kind1]),
                }
            }
            [.., last1, last0] => {
                let [kind1, kind0] = [*last1, *last0].map(RegKind::new);
                let kind0 = kind0?;
                let Some(kind1) = kind1 else {
                    return Some(BranchParamRegs::new_one(kind0));
                };
                match kind0 != kind1 {
                    true => BranchParamRegs::new_two([kind0, kind1]),
                    false => BranchParamRegs::new_one(kind0),
                }
            }
        };
        Some(regs)
    }

    /// Creates [`BranchParams`] from `ty` and the control flow `kind`.
    fn branch_params_for(&self, func_ty: &FuncType, kind: ControlFrameKind) -> BranchParams {
        let len_params = func_ty.len_params();
        let temp_slots = self.branch_slots(len_params.into());
        let (tys, len_tys) = match kind {
            ControlFrameKind::Loop => (func_ty.params(), func_ty.len_params()),
            _ => (func_ty.results(), func_ty.len_results()),
        };
        let regs = Self::branch_params_regs(tys);
        let regs_len = regs.as_ref().map(BranchParamRegs::len).unwrap_or(0);
        let temp_len = len_tys - regs_len;
        BranchParams::new(temp_slots, temp_len, regs)
    }

    /// Creates [`BranchParams`] from `ty` and the control flow `kind`.
    fn branch_params(&self, block_ty: BlockType, kind: ControlFrameKind) -> BranchParams {
        block_ty.func_type_with(&self.engine, |func_ty| {
            self.branch_params_for(func_ty, kind)
        })
    }

    /// Returns the height of the control frame with `block_ty` for `self`.
    fn block_height(&self, block_ty: BlockType) -> usize {
        let len_block_params = usize::from(block_ty.len_params(&self.engine));
        self.height() - len_block_params
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
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        debug_assert!(self.controls.is_empty());
        debug_assert!(self.is_fuel_metering_enabled() == fuel_pos.is_some());
        let temp_slots = self.operands.next_temp_slots();
        let temp_len = ty.func_type_with(&self.engine, FuncType::len_results);
        let branch_params = BranchParams::new(temp_slots, temp_len, None);
        self.controls
            .push_block(ty, 0, branch_params, label, fuel_pos);
        Ok(())
    }

    /// Pushes a Wasm `block` onto the [`Stack`].
    ///
    /// # Note
    ///
    /// This inherits the `consume_fuel` [`Pos<BlockFuel>`] from the parent [`ControlFrame`].
    ///
    /// # Errors
    ///
    /// If the stack height exceeds the maximum height.
    pub fn push_block(&mut self, ty: BlockType, label: LabelRef) -> Result<(), Error> {
        debug_assert!(!self.controls.is_empty());
        let block_height = self.block_height(ty);
        let branch_params = self.branch_params(ty, ControlFrameKind::Block);
        let consume_fuel = self.fuel_pos();
        self.controls
            .push_block(ty, block_height, branch_params, label, consume_fuel);
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
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        debug_assert!(!self.controls.is_empty());
        debug_assert!(self.is_fuel_metering_enabled() == fuel_pos.is_some());
        let block_height = self.block_height(ty);
        let branch_params = self.branch_params(ty, ControlFrameKind::Loop);
        self.controls
            .push_loop(ty, block_height, branch_params, label, fuel_pos);
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
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        debug_assert!(!self.controls.is_empty());
        debug_assert!(self.is_fuel_metering_enabled() == fuel_pos.is_some());
        let block_height = self.block_height(ty);
        let len_block_params = usize::from(ty.len_params(&self.engine));
        let else_operands = self.operands.peek(len_block_params);
        let registers = self.operands.get_registers();
        debug_assert!(len_block_params == else_operands.len());
        let branch_params = self.branch_params(ty, ControlFrameKind::If);
        self.controls.push_if(
            ty,
            block_height,
            branch_params,
            label,
            fuel_pos,
            reachability,
            else_operands,
            registers,
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
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        debug_assert!(self.is_fuel_metering_enabled() == fuel_pos.is_some());
        self.push_else_operands(&if_frame)?;
        self.operands.set_registers(if_frame.registers());
        self.controls
            .push_else(if_frame, fuel_pos, is_end_of_then_reachable);
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
    pub fn push_else_operands(&mut self, frame: &IfControlFrame) -> Result<(), Error> {
        match frame.reachability() {
            IfReachability::Both { .. } => {}
            IfReachability::OnlyThen | IfReachability::OnlyElse => return Ok(()),
        };
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

    /// Registers the local at `local_index` for register of type `ty`.
    pub fn register_local_for_reg(
        &mut self,
        ty: ValType,
        local_index: LocalIdx,
    ) -> Result<(), Error> {
        self.operands.register_local_for_reg(ty, local_index)
    }

    /// Deallocates the local at `local_index` for register of type `ty`.
    pub fn dealloc_local_for_reg(
        &mut self,
        ty: ValType,
        local_index: LocalIdx,
    ) -> Result<(), Error> {
        self.operands.dealloc_local_for_reg(ty, local_index)
    }

    /// Deallocates the register of the `ty` from `self`.
    ///
    /// Returns `Some` if a copy operator is required to reflect the changes.
    ///
    /// # Note
    ///
    /// If the register operand is a register-backed local it is turned into a normal local operand
    /// and `None` is returned as no copy operator is required.
    pub fn dealloc_reg(&mut self, ty: ValType) -> Option<TempOperand> {
        self.operands.dealloc_reg(ty)
    }

    /// Pushes the [`Operand`] back to the [`Stack`].
    ///
    /// Returns the new [`StackPos`].
    ///
    /// # Errors
    ///
    /// - If too many operands have been pushed onto the [`Stack`].
    /// - If the local with `local_idx` does not exist.
    pub fn push_operand(&mut self, operand: Operand) -> Result<Operand, Error> {
        self.operands.push_operand(operand)
    }

    /// Pushes a local variable with index `local_idx` to the [`Stack`].
    ///
    /// # Errors
    ///
    /// - If too many operands have been pushed onto the [`Stack`].
    /// - If the local with `local_idx` does not exist.
    pub fn push_local(
        &mut self,
        local_index: LocalIdx,
        ty: ValType,
    ) -> Result<LocalOperand, Error> {
        self.operands.push_local(local_index, ty)
    }

    /// Pushes a temporary with type `ty` on the [`Stack`].
    ///
    /// # Errors
    ///
    /// If too many operands have been pushed onto the [`Stack`].
    #[inline]
    pub fn push_temp(&mut self, ty: ValType, alloc: Allocation) -> Result<TempOperand, Error> {
        self.operands.push_temp(ty, alloc)
    }

    /// Pushes an immediate `value` on the [`Stack`].
    ///
    /// # Errors
    ///
    /// If too many operands have been pushed onto the [`Stack`].
    #[inline]
    pub fn push_immediate(
        &mut self,
        value: impl Into<TypedRawVal>,
    ) -> Result<ImmediateOperand, Error> {
        self.operands.push_immediate(value)
    }

    /// Peeks the [`Operand`] at `depth`.
    ///
    /// # Note
    ///
    /// A `depth` of 0 peeks the top-most [`Operand`] on `self`.
    ///
    /// # Panics
    ///
    /// If `depth` is out of bounds for `self`.
    #[inline]
    pub fn peek(&self, depth: usize) -> Operand {
        self.operands.get(depth)
    }

    /// Returns an iterator yielding the top-most `len` operands from the stack.
    ///
    /// Operands are yieleded in insertion order.
    pub fn peek_n(&self, len: usize) -> PeekedOperands<'_> {
        self.operands.peek(len)
    }

    /// Pops the top-most [`Operand`] from the [`Stack`].
    ///
    /// # Panics
    ///
    /// If `self` is empty.
    #[inline]
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
    #[inline]
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
    pub fn preserve_locals(&mut self, local_index: LocalIdx) -> PreservedLocalsIter<'_> {
        self.operands.preserve_locals(local_index)
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
    pub fn preserve_all_locals(&mut self, skip: usize) -> PreservedAllLocalsIter<'_> {
        self.operands.preserve_all_locals(skip)
    }

    /// Discards any local register links from the [`OperandStack`].
    pub fn discard_local_regs(&mut self) {
        self.operands.discard_local_regs()
    }

    /// Preserve all register operands on the [`Stack`].
    ///
    /// This is done by converting those operands to [`StackOperand::Temp`] and
    /// returning their associated slots in order to emit copy operators by
    /// the caller.
    #[must_use]
    pub fn preserve_all_regs(&mut self) -> PreservedRegs {
        self.operands.preserve_all_regs()
    }

    /// Preserve temporary register operands on the [`Stack`].
    ///
    /// This is done by converting those operands to [`StackOperand::Temp`] and
    /// returning their associated slots in order to emit copy operators by
    /// the caller.
    #[must_use]
    pub fn preserve_all_temp_regs(&mut self, skip: usize) -> PreservedRegs {
        self.operands.preserve_all_temp_regs(skip)
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

    /// Returns the current [`Op::ConsumeFuel`] if fuel metering is enabled.
    ///
    /// Returns `None` otherwise.
    #[inline]
    pub fn fuel_pos(&self) -> Option<Pos<ir::BlockFuel>> {
        self.controls.fuel_pos()
    }
}
