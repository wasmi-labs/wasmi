use super::{Reset, ReusableAllocations};
use crate::{
    core::FuelCostsProvider,
    engine::translator::utils::{BumpFuelConsumption as _, Instr, IsInstructionParameter as _},
    ir::Instruction,
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

    /// Pushes an [`Instruction`] parameter to the [`InstrEncoder`].
    ///
    /// The parameter is associated to the last pushed [`Instruction`].
    pub fn push_param(&mut self, instruction: Instruction) {
        debug_assert!(
            instruction.is_instruction_parameter(),
            "non-parameter: {instruction:?}"
        );
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

    /// Returns an iterator yielding all [`Instruction`]s of the [`InstrEncoder`].
    ///
    /// # Note
    ///
    /// The [`InstrEncoder`] will be empty after this operation.
    pub fn drain(&mut self) -> InstrEncoderIter {
        InstrEncoderIter {
            iter: self.instrs.drain(..),
        }
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
