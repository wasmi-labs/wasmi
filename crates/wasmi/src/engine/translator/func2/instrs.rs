use super::{Reset, ReusableAllocations};
use crate::{
    core::{FuelCostsProvider, ValType},
    engine::translator::{
        comparator::{
            CmpSelectFusion,
            CompareResult as _,
            TryIntoCmpSelectInstr as _,
            UpdateBranchOffset as _,
        },
        func2::{Operand, Stack, StackLayout, StackSpace},
        relink_result::RelinkResult,
        utils::{BumpFuelConsumption as _, Instr, IsInstructionParameter as _},
    },
    ir::{BranchOffset, Instruction, Reg},
    module::ModuleHeader,
    Engine,
    Error,
};
use alloc::vec::{self, Vec};

/// Creates and encodes the list of [`Instruction`]s for a function.
#[derive(Debug, Default)]
pub struct InstrEncoder {
    /// The list of constructed instructions and their parameters.
    instrs: Vec<Instruction>,
    /// The fuel costs of instructions.
    ///
    /// This is `Some` if fuel metering is enabled, otherwise `None`.
    fuel_costs: Option<FuelCostsProvider>,
    /// The last pushed non-parameter [`Instruction`].
    last_instr: Option<Instr>,
}

impl ReusableAllocations for InstrEncoder {
    type Allocations = InstrEncoderAllocations;

    fn into_allocations(self) -> Self::Allocations {
        Self::Allocations {
            instrs: self.instrs,
        }
    }
}

/// The reusable heap allocations of the [`InstrEncoder`].
#[derive(Debug, Default)]
pub struct InstrEncoderAllocations {
    /// The list of constructed instructions and their parameters.
    instrs: Vec<Instruction>,
}

impl Reset for InstrEncoderAllocations {
    fn reset(&mut self) {
        self.instrs.clear();
    }
}

impl InstrEncoder {
    /// Creates a new [`InstrEncoder`].
    pub fn new(engine: &Engine, alloc: InstrEncoderAllocations) -> Self {
        let config = engine.config();
        let fuel_costs = config
            .get_consume_fuel()
            .then(|| config.fuel_costs())
            .cloned();
        Self {
            instrs: alloc.instrs,
            fuel_costs,
            last_instr: None,
        }
    }

    /// Returns the next [`Instr`].
    #[must_use]
    pub fn next_instr(&self) -> Instr {
        Instr::from_usize(self.instrs.len())
    }

    /// Pushes an [`Instruction::ConsumeFuel`] instruction to `self`.
    ///
    /// # Note
    ///
    /// The pushes [`Instruction::ConsumeFuel`] is initialized with base fuel costs.
    pub fn push_consume_fuel_instr(&mut self) -> Result<Option<Instr>, Error> {
        let Some(fuel_costs) = &self.fuel_costs else {
            return Ok(None);
        };
        let base_costs = fuel_costs.base();
        let Ok(base_costs) = u32::try_from(base_costs) else {
            panic!("out of  bounds base fuel costs: {base_costs}");
        };
        let instr = self.push_instr_impl(Instruction::consume_fuel(base_costs))?;
        Ok(Some(instr))
    }

    /// Pushes a non-parameter [`Instruction`] to the [`InstrEncoder`].
    ///
    /// Returns an [`Instr`] that refers to the pushed [`Instruction`].
    pub fn push_instr(
        &mut self,
        instruction: Instruction,
        consume_fuel: Option<Instr>,
        f: impl FnOnce(&FuelCostsProvider) -> u64,
    ) -> Result<Instr, Error> {
        self.bump_fuel_consumption(consume_fuel, f)?;
        self.push_instr_impl(instruction)
    }

    /// Pushes a non-parameter [`Instruction`] to the [`InstrEncoder`].
    fn push_instr_impl(&mut self, instruction: Instruction) -> Result<Instr, Error> {
        debug_assert!(
            !instruction.is_instruction_parameter(),
            "parameter: {instruction:?}"
        );
        let instr = self.next_instr();
        self.instrs.push(instruction);
        self.last_instr = Some(instr);
        Ok(instr)
    }

    /// Replaces `instr` with `new_instr` in `self`.
    ///
    /// - Returns `Ok(true)` if replacement was successful.
    /// - Returns `Ok(false)` if replacement was unsuccessful.
    ///
    /// # Panics (Debug)
    ///
    /// If `instr` or `new_instr` are [`Instruction`] parameters.
    pub fn try_replace_instr(
        &mut self,
        instr: Instr,
        new_instr: Instruction,
    ) -> Result<bool, Error> {
        debug_assert!(
            !new_instr.is_instruction_parameter(),
            "parameter: {new_instr:?}"
        );
        let Some(last_instr) = self.last_instr else {
            return Ok(false);
        };
        let replace = self.get_mut(instr);
        debug_assert!(!replace.is_instruction_parameter(), "parameter: {instr:?}");
        if instr != last_instr {
            return Ok(false);
        }
        *replace = new_instr;
        Ok(true)
    }

    /// Tries to replace the result of the last instruction with `new_result` if possible.
    ///
    /// # Note
    ///
    /// - `old_result`:
    ///   just required for additional safety to check if the last instruction
    ///   really is the source of the `local.set` or `local.tee`.
    /// - `new_result`:
    ///   the new result which shall replace the `old_result`.
    pub fn try_replace_result(
        &mut self,
        new_result: Reg,
        old_result: Reg,
        layout: &StackLayout,
        module: &ModuleHeader,
    ) -> Result<bool, Error> {
        if !matches!(layout.stack_space(new_result), StackSpace::Local) {
            // Case: cannot replace result if `new_result` isn't a local.
            return Ok(false);
        }
        let Some(last_instr) = self.last_instr else {
            // Case: cannot replace result without last instruction.
            return Ok(false);
        };
        if !self
            .get_mut(last_instr)
            .relink_result(module, new_result, old_result)?
        {
            // Case: it was impossible to relink the result of `last_instr.
            return Ok(false);
        }
        Ok(true)
    }

    /// Tries to fuse a compare instruction with a Wasm `select` instruction.
    ///
    /// # Returns
    ///
    /// - Returns `Some` if fusion was successful.
    /// - Returns `None` if fusion could not be applied.
    pub fn try_fuse_select(
        &mut self,
        ty: ValType,
        select_condition: Reg,
        layout: &StackLayout,
        stack: &mut Stack,
    ) -> Result<Option<bool>, Error> {
        let Some(last_instr) = self.last_instr else {
            // If there is no last instruction there is no comparison instruction to negate.
            return Ok(None);
        };
        let last_instruction = self.get(last_instr);
        let Some(last_result) = last_instruction.compare_result() else {
            // All negatable instructions have a single result register.
            return Ok(None);
        };
        if matches!(layout.stack_space(last_result), StackSpace::Local) {
            // The instruction stores its result into a local variable which
            // is an observable side effect which we are not allowed to mutate.
            return Ok(None);
        }
        if last_result != select_condition {
            // The result of the last instruction and the select's `condition`
            // are not equal thus indicating that we cannot fuse the instructions.
            return Ok(None);
        }
        let CmpSelectFusion::Applied {
            fused,
            swap_operands,
        } = last_instruction.try_into_cmp_select_instr(|| {
            let select_result = stack.push_temp(ty, Some(last_instr))?;
            let select_result = layout.temp_to_reg(select_result)?;
            Ok(select_result)
        })?
        else {
            return Ok(None);
        };
        let last_instr = self.get_mut(last_instr);
        *last_instr = fused;
        Ok(Some(swap_operands))
    }

    /// Pushes an [`Instruction`] parameter to the [`InstrEncoder`].
    ///
    /// The parameter is associated to the last pushed [`Instruction`].
    pub fn push_param(&mut self, instruction: Instruction) {
        self.instrs.push(instruction);
    }

    /// Returns a shared reference to the [`Instruction`] associated to [`Instr`].
    ///
    /// # Panics
    ///
    /// If `instr` is out of bounds for `self`.
    pub fn get(&self, instr: Instr) -> &Instruction {
        &self.instrs[instr.into_usize()]
    }

    /// Returns an exclusive reference to the [`Instruction`] associated to [`Instr`].
    ///
    /// # Panics
    ///
    /// If `instr` is out of bounds for `self`.
    fn get_mut(&mut self, instr: Instr) -> &mut Instruction {
        &mut self.instrs[instr.into_usize()]
    }

    /// Resets the [`Instr`] last created via [`InstrEncoder::push_instr`].
    ///
    /// # Note
    ///
    /// The `last_instr` info is used for op-code fusion of `local.set`
    /// `local.tee`, compare, conditional branch and `select` instructions.
    ///
    /// Whenever ending a control frame during Wasm translation `last_instr`
    /// needs to be reset to `None` to signal that no such optimization is
    /// valid across control flow boundaries.
    pub fn reset_last_instr(&mut self) {
        self.last_instr = None;
    }

    /// Updates the branch offset of `instr` to `offset`.
    ///
    /// # Errors
    ///
    /// If the branch offset could not be updated for `instr`.
    pub fn update_branch_offset(
        &mut self,
        instr: Instr,
        offset: BranchOffset,
        layout: &mut StackLayout,
    ) -> Result<(), Error> {
        self.get_mut(instr).update_branch_offset(layout, offset)?;
        Ok(())
    }

    /// Bumps consumed fuel for [`Instruction::ConsumeFuel`] of `instr` by `delta`.
    ///
    /// # Errors
    ///
    /// If consumed fuel is out of bounds after this operation.
    pub fn bump_fuel_consumption(
        &mut self,
        consume_fuel: Option<Instr>,
        f: impl FnOnce(&FuelCostsProvider) -> u64,
    ) -> Result<(), Error> {
        let (fuel_costs, consume_fuel) = match (&self.fuel_costs, consume_fuel) {
            (None, None) => return Ok(()),
            (Some(fuel_costs), Some(consume_fuel)) => (fuel_costs, consume_fuel),
            _ => {
                panic!(
                    "fuel metering state mismatch: fuel_costs: {:?}, fuel_instr: {:?}",
                    self.fuel_costs, consume_fuel,
                );
            }
        };
        let fuel_consumed = f(fuel_costs);
        self.get_mut(consume_fuel)
            .bump_fuel_consumption(fuel_consumed)?;
        Ok(())
    }

    /// Encode the top-most `len` operands on the stack as register list.
    ///
    /// # Note
    ///
    /// This is used for the following n-ary instructions:
    ///
    /// - [`Instruction::ReturnMany`]
    /// - [`Instruction::CopyManyNonOverlapping`]
    /// - [`Instruction::CallInternal`]
    /// - [`Instruction::CallImported`]
    /// - [`Instruction::CallIndirect`]
    /// - [`Instruction::ReturnCallInternal`]
    /// - [`Instruction::ReturnCallImported`]
    /// - [`Instruction::ReturnCallIndirect`]
    pub fn encode_register_list(
        &mut self,
        operands: &[Operand],
        layout: &mut StackLayout,
    ) -> Result<(), Error> {
        let mut remaining = operands;
        let mut operand_to_reg =
            |operand: &Operand| -> Result<Reg, Error> { layout.operand_to_reg(*operand) };
        let instr = loop {
            match remaining {
                [] => return Ok(()),
                [v0] => {
                    let v0 = operand_to_reg(v0)?;
                    break Instruction::register(v0);
                }
                [v0, v1] => {
                    let v0 = operand_to_reg(v0)?;
                    let v1 = operand_to_reg(v1)?;
                    break Instruction::register2_ext(v0, v1);
                }
                [v0, v1, v2] => {
                    let v0 = operand_to_reg(v0)?;
                    let v1 = operand_to_reg(v1)?;
                    let v2 = operand_to_reg(v2)?;
                    break Instruction::register3_ext(v0, v1, v2);
                }
                [v0, v1, v2, rest @ ..] => {
                    let v0 = operand_to_reg(v0)?;
                    let v1 = operand_to_reg(v1)?;
                    let v2 = operand_to_reg(v2)?;
                    let instr = Instruction::register_list_ext(v0, v1, v2);
                    self.push_param(instr);
                    remaining = rest;
                }
            };
        };
        self.push_param(instr);
        Ok(())
    }

    /// Returns an iterator yielding all [`Instruction`]s of the [`InstrEncoder`].
    ///
    /// # Note
    ///
    /// The [`InstrEncoder`] will be empty after this operation.
    pub fn drain(&mut self) -> InstrEncoderIter<'_> {
        InstrEncoderIter {
            iter: self.instrs.drain(..),
        }
    }

    /// Returns the last instruction of the [`InstrEncoder`] if any.
    pub fn last_instr(&self) -> Option<Instr> {
        self.last_instr
    }
}

/// Iterator yielding all [`Instruction`]s of the [`InstrEncoder`].
#[derive(Debug)]
pub struct InstrEncoderIter<'a> {
    /// The underlying iterator.
    iter: vec::Drain<'a, Instruction>,
}

impl<'a> Iterator for InstrEncoderIter<'a> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl ExactSizeIterator for InstrEncoderIter<'_> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}
