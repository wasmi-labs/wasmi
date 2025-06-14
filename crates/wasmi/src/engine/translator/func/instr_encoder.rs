use crate::{
    core::{FuelCostsProvider, UntypedVal, ValType},
    engine::translator::{
        comparator::{
            CompareResult,
            LogicalizeCmpInstr,
            NegateCmpInstr,
            TryIntoCmpBranchFallbackInstr,
            TryIntoCmpBranchInstr,
            TryIntoCmpSelectInstr,
        },
        func::{
            stack::RegisterSpace,
            utils::FromProviders as _,
            LabelRef,
            LabelRegistry,
            TypedProvider,
            ValueStack,
        },
        relink_result::RelinkResult as _,
        utils::{FuelInfo, WasmInteger},
        visit_register::VisitInputRegisters as _,
        BumpFuelConsumption as _,
    },
    ir::{
        BoundedRegSpan,
        BranchOffset,
        BranchOffset16,
        Comparator,
        ComparatorAndOffset,
        Const32,
        Instruction,
        Reg,
        RegSpan,
    },
    module::ModuleHeader,
    Error,
};
use alloc::vec::{Drain, Vec};
use core::mem;

/// A reference to an instruction of the partially
/// constructed function body of the [`InstrEncoder`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instr(u32);

impl Instr {
    /// Creates an [`Instr`] from the given `usize` value.
    ///
    /// # Note
    ///
    /// This intentionally is an API intended for test purposes only.
    ///
    /// # Panics
    ///
    /// If the `value` exceeds limitations for [`Instr`].
    pub fn from_usize(value: usize) -> Self {
        let value = value.try_into().unwrap_or_else(|error| {
            panic!("invalid index {value} for instruction reference: {error}")
        });
        Self(value)
    }

    /// Returns an `usize` representation of the instruction index.
    pub fn into_usize(self) -> usize {
        self.0 as usize
    }

    /// Creates an [`Instr`] form the given `u32` value.
    pub fn from_u32(value: u32) -> Self {
        Self(value)
    }

    /// Returns an `u32` representation of the instruction index.
    pub fn into_u32(self) -> u32 {
        self.0
    }

    /// Returns the absolute distance between `self` and `other`.
    ///
    /// - Returns `0` if `self == other`.
    /// - Returns `1` if `self` is adjacent to `other` in the sequence of instructions.
    /// - etc..
    pub fn distance(self, other: Self) -> u32 {
        self.0.abs_diff(other.0)
    }
}

/// Encodes Wasmi bytecode instructions to an [`Instruction`] stream.
#[derive(Debug, Default)]
pub struct InstrEncoder {
    /// Already encoded [`Instruction`] words.
    instrs: InstrSequence,
    /// Unresolved and unpinned labels created during function translation.
    labels: LabelRegistry,
    /// The last [`Instruction`] created via [`InstrEncoder::push_instr`].
    last_instr: Option<Instr>,
    /// The first encoded [`Instr`] that is affected by a `local.set` preservation.
    ///
    /// # Note
    ///
    /// This is an optimization to reduce the amount of work performed during
    /// defragmentation of the register space due to `local.set` register
    /// preservations.
    notified_preservation: Option<Instr>,
}

/// The sequence of encoded [`Instruction`].
#[derive(Debug, Default)]
pub struct InstrSequence {
    /// Already encoded [`Instruction`] words.
    instrs: Vec<Instruction>,
}

impl InstrSequence {
    /// Resets the [`InstrSequence`].
    pub fn reset(&mut self) {
        self.instrs.clear();
    }

    /// Returns the next [`Instr`].
    fn next_instr(&self) -> Instr {
        Instr::from_usize(self.instrs.len())
    }

    /// Pushes an [`Instruction`] to the instruction sequence and returns its [`Instr`].
    ///
    /// # Errors
    ///
    /// If there are too many instructions in the instruction sequence.
    fn push(&mut self, instruction: Instruction) -> Result<Instr, Error> {
        let instr = self.next_instr();
        self.instrs.push(instruction);
        Ok(instr)
    }

    /// Pushes an [`Instruction`] before the [`Instruction`] at [`Instr`].
    ///
    /// Returns the [`Instr`] of the [`Instruction`] that was at [`Instr`] before this operation.
    ///
    /// # Note
    ///
    /// - This operation might be costly. Callers are advised to only insert
    ///   instructions near the end of the sequence in order to avoid massive
    ///   copy overhead since all following instructions are required to be
    ///   shifted in memory.
    /// - The `instr` will refer to the inserted [`Instruction`] after this operation.
    ///
    /// # Errors
    ///
    /// If there are too many instructions in the instruction sequence.
    fn push_before(&mut self, instr: Instr, instruction: Instruction) -> Result<Instr, Error> {
        self.instrs.insert(instr.into_usize(), instruction);
        let shifted_instr = instr
            .into_u32()
            .checked_add(1)
            .map(Instr::from_u32)
            .unwrap_or_else(|| panic!("pushed to many instructions to a single function"));
        Ok(shifted_instr)
    }

    /// Returns the [`Instruction`] associated to the [`Instr`] for this [`InstrSequence`].
    ///
    /// # Panics
    ///
    /// If no [`Instruction`] is associated to the [`Instr`] for this [`InstrSequence`].
    fn get(&self, instr: Instr) -> &Instruction {
        &self.instrs[instr.into_usize()]
    }

    /// Returns the [`Instruction`] associated to the [`Instr`] for this [`InstrSequence`].
    ///
    /// # Panics
    ///
    /// If no [`Instruction`] is associated to the [`Instr`] for this [`InstrSequence`].
    fn get_mut(&mut self, instr: Instr) -> &mut Instruction {
        &mut self.instrs[instr.into_usize()]
    }

    /// Return an iterator over the sequence of generated [`Instruction`].
    ///
    /// # Note
    ///
    /// The [`InstrSequence`] will be in an empty state after this operation.
    pub fn drain(&mut self) -> Drain<Instruction> {
        self.instrs.drain(..)
    }

    /// Returns a slice to the sequence of [`Instruction`] starting at `start`.
    ///
    /// # Panics
    ///
    /// If `start` is out of bounds for [`InstrSequence`].
    pub fn get_slice_at_mut(&mut self, start: Instr) -> &mut [Instruction] {
        &mut self.instrs[start.into_usize()..]
    }
}

impl<'a> IntoIterator for &'a mut InstrSequence {
    type Item = &'a mut Instruction;
    type IntoIter = core::slice::IterMut<'a, Instruction>;

    fn into_iter(self) -> Self::IntoIter {
        self.instrs.iter_mut()
    }
}

impl InstrEncoder {
    /// Resets the [`InstrEncoder`].
    pub fn reset(&mut self) {
        self.instrs.reset();
        self.labels.reset();
        self.reset_last_instr();
        self.notified_preservation = None;
    }

    /// Resets the [`Instr`] last created via [`InstrEncoder::push_instr`].
    ///
    /// # Note
    ///
    /// The `last_instr` information is used for an optimization with `local.set`
    /// and `local.tee` translation to replace the result [`Reg`] of the
    /// last created [`Instruction`] instead of creating another copy [`Instruction`].
    ///
    /// Whenever ending a control block during Wasm translation the `last_instr`
    /// information needs to be reset so that a `local.set` or `local.tee` does
    /// not invalidly optimize across control flow boundaries.
    pub fn reset_last_instr(&mut self) {
        self.last_instr = None;
    }

    /// Return an iterator over the sequence of generated [`Instruction`].
    ///
    /// # Note
    ///
    /// The [`InstrEncoder`] will be in an empty state after this operation.
    pub fn drain_instrs(&mut self) -> Drain<Instruction> {
        self.instrs.drain()
    }

    /// Creates a new unresolved label and returns its [`LabelRef`].
    pub fn new_label(&mut self) -> LabelRef {
        self.labels.new_label()
    }

    /// Resolve the label at the current instruction position.
    ///
    /// Does nothing if the label has already been resolved.
    ///
    /// # Note
    ///
    /// This is used at a position of the Wasm bytecode where it is clear that
    /// the given label can be resolved properly.
    /// This usually takes place when encountering the Wasm `End` operand for example.
    pub fn pin_label_if_unpinned(&mut self, label: LabelRef) {
        self.labels.try_pin_label(label, self.instrs.next_instr())
    }

    /// Resolve the label at the current instruction position.
    ///
    /// # Note
    ///
    /// This is used at a position of the Wasm bytecode where it is clear that
    /// the given label can be resolved properly.
    /// This usually takes place when encountering the Wasm `End` operand for example.
    ///
    /// # Panics
    ///
    /// If the label has already been resolved.
    pub fn pin_label(&mut self, label: LabelRef) {
        self.labels
            .pin_label(label, self.instrs.next_instr())
            .unwrap_or_else(|err| panic!("failed to pin label: {err}"));
    }

    /// Try resolving the [`LabelRef`] for the currently constructed instruction.
    ///
    /// Returns an uninitialized [`BranchOffset`] if the `label` cannot yet
    /// be resolved and defers resolution to later.
    pub fn try_resolve_label(&mut self, label: LabelRef) -> Result<BranchOffset, Error> {
        let user = self.instrs.next_instr();
        self.try_resolve_label_for(label, user)
    }

    /// Try resolving the [`LabelRef`] for the given [`Instr`].
    ///
    /// Returns an uninitialized [`BranchOffset`] if the `label` cannot yet
    /// be resolved and defers resolution to later.
    ///
    /// # Errors
    ///
    /// If the [`BranchOffset`] cannot be encoded in 32 bits.
    pub fn try_resolve_label_for(
        &mut self,
        label: LabelRef,
        instr: Instr,
    ) -> Result<BranchOffset, Error> {
        self.labels.try_resolve_label(label, instr)
    }

    /// Updates the branch offsets of all branch instructions inplace.
    ///
    /// # Panics
    ///
    /// If this is used before all branching labels have been pinned.
    pub fn update_branch_offsets(&mut self, stack: &mut ValueStack) -> Result<(), Error> {
        for (user, offset) in self.labels.resolved_users() {
            self.instrs
                .get_mut(user)
                .update_branch_offset(stack, offset?)?;
        }
        Ok(())
    }

    /// Push the [`Instruction`] to the [`InstrEncoder`].
    fn push_instr(&mut self, instr: Instruction) -> Result<Instr, Error> {
        let last_instr = self.instrs.push(instr)?;
        self.last_instr = Some(last_instr);
        Ok(last_instr)
    }

    /// Pushes a [`Instruction::ConsumeFuel`] with base costs if fuel metering is enabled.
    ///
    /// Returns `None` if fuel metering is disabled.
    pub fn push_fuel_instr(
        &mut self,
        fuel_costs: Option<&FuelCostsProvider>,
    ) -> Result<Option<Instr>, Error> {
        let Some(fuel_costs) = fuel_costs else {
            // Fuel metering is disabled so there is no need to create an `Instruction::ConsumeFuel`.
            return Ok(None);
        };
        let base = u32::try_from(fuel_costs.base())
            .expect("base fuel must be valid for creating `Instruction::ConsumeFuel`");
        let fuel_instr = Instruction::consume_fuel(base);
        let instr = self.push_instr(fuel_instr)?;
        Ok(Some(instr))
    }

    /// Utility function for pushing a new [`Instruction`] with fuel costs.
    ///
    /// # Note
    ///
    /// Fuel metering is only encoded or adjusted if it is enabled.
    pub fn push_fueled_instr<F>(
        &mut self,
        instr: Instruction,
        fuel_info: &FuelInfo,
        f: F,
    ) -> Result<Instr, Error>
    where
        F: FnOnce(&FuelCostsProvider) -> u64,
    {
        self.bump_fuel_consumption(fuel_info, f)?;
        self.push_instr(instr)
    }

    /// Appends the [`Instruction`] to the last [`Instruction`] created via [`InstrEncoder::push_instr`].
    ///
    /// # Note
    ///
    /// This is used primarily for [`Instruction`] words that are just carrying
    /// parameters for the [`Instruction`]. An example of this is [`Instruction::RegisterAndImm32`]
    /// carrying the `ptr` and `offset` parameters for [`Instruction::Load32`].
    pub fn append_instr(&mut self, instr: Instruction) -> Result<Instr, Error> {
        self.instrs.push(instr)
    }

    /// Tries to merge `copy(result, value)` if the last instruction is a matching copy instruction.
    ///
    /// - Returns `None` if merging of the copy instruction was not possible.
    /// - Returns the `Instr` of the merged `copy2` instruction if merging was successful.
    fn merge_copy_instrs(&mut self, result: Reg, value: TypedProvider) -> Option<Instr> {
        let TypedProvider::Register(mut value) = value else {
            // Case: cannot merge copies with immediate values at the moment.
            //
            // Note: we could implement this but it would require us to allocate
            //       function local constants which we want to avoid generally.
            return None;
        };
        let Some(last_instr) = self.last_instr else {
            // There is no last instruction, e.g. when ending a `block`.
            return None;
        };
        let Instruction::Copy {
            result: last_result,
            value: last_value,
        } = *self.instrs.get(last_instr)
        else {
            // Case: last instruction was not a copy instruction, so we cannot merge anything.
            return None;
        };
        if !(result == last_result.next() || result == last_result.prev()) {
            // Case: cannot merge copy instructions as `copy2` since result registers are not contiguous.
            return None;
        }

        // Propagate values according to the order of the merged copies.
        if value == last_result {
            value = last_value;
        }

        let (merged_result, value0, value1) = if last_result < result {
            (last_result, last_value, value)
        } else {
            (result, value, last_value)
        };

        let merged_copy = Instruction::copy2_ext(RegSpan::new(merged_result), value0, value1);
        *self.instrs.get_mut(last_instr) = merged_copy;
        Some(last_instr)
    }

    /// Encode a `copy result <- value` instruction.
    ///
    /// # Note
    ///
    /// Applies optimizations for `copy x <- x` and properly selects the
    /// most optimized `copy` instruction variant for the given `value`.
    pub fn encode_copy(
        &mut self,
        stack: &mut ValueStack,
        result: Reg,
        value: TypedProvider,
        fuel_info: &FuelInfo,
    ) -> Result<Option<Instr>, Error> {
        /// Convenience to create an [`Instruction::Copy`] to copy a constant value.
        fn copy_imm(
            stack: &mut ValueStack,
            result: Reg,
            value: impl Into<UntypedVal>,
        ) -> Result<Instruction, Error> {
            let cref = stack.alloc_const(value.into())?;
            Ok(Instruction::copy(result, cref))
        }
        if let Some(merged_instr) = self.merge_copy_instrs(result, value) {
            return Ok(Some(merged_instr));
        }
        let instr = match value {
            TypedProvider::Register(value) => {
                if result == value {
                    // Optimization: copying from register `x` into `x` is a no-op.
                    return Ok(None);
                }
                Instruction::copy(result, value)
            }
            TypedProvider::Const(value) => match value.ty() {
                ValType::I32 => Instruction::copy_imm32(result, i32::from(value)),
                ValType::F32 => Instruction::copy_imm32(result, f32::from(value)),
                ValType::I64 => match <Const32<i64>>::try_from(i64::from(value)).ok() {
                    Some(value) => Instruction::copy_i64imm32(result, value),
                    None => copy_imm(stack, result, value)?,
                },
                ValType::F64 => match <Const32<f64>>::try_from(f64::from(value)).ok() {
                    Some(value) => Instruction::copy_f64imm32(result, value),
                    None => copy_imm(stack, result, value)?,
                },
                ValType::V128 | ValType::FuncRef | ValType::ExternRef => {
                    copy_imm(stack, result, value)?
                }
            },
        };
        self.bump_fuel_consumption(fuel_info, FuelCostsProvider::base)?;
        let instr = self.push_instr(instr)?;
        Ok(Some(instr))
    }

    /// Encode a generic `copy` instruction.
    ///
    /// # Note
    ///
    /// Avoids no-op copies such as `copy x <- x` and properly selects the
    /// most optimized `copy` instruction variant for the given `value`.
    pub fn encode_copies(
        &mut self,
        stack: &mut ValueStack,
        mut results: BoundedRegSpan,
        values: &[TypedProvider],
        fuel_info: &FuelInfo,
    ) -> Result<Option<Instr>, Error> {
        assert_eq!(usize::from(results.len()), values.len());
        let result = results.span().head();
        if let Some((TypedProvider::Register(value), rest)) = values.split_first() {
            if result == *value {
                // Case: `result` and `value` are equal thus this is a no-op copy which we can avoid.
                //       Applied recursively we thereby remove all no-op copies at the start of the
                //       copy sequence until the first actual copy.
                results = BoundedRegSpan::new(RegSpan::new(result.next()), results.len() - 1);
                return self.encode_copies(stack, results, rest, fuel_info);
            }
        }
        match values {
            [] => {
                // The copy sequence is empty, nothing to encode in this case.
                Ok(None)
            }
            [v0] => self.encode_copy(stack, result, *v0, fuel_info),
            [v0, v1] => {
                if TypedProvider::Register(result.next()) == *v1 {
                    // Case: the second of the 2 copies is a no-op which we can avoid
                    // Note: we already asserted that the first copy is not a no-op
                    return self.encode_copy(stack, result, *v0, fuel_info);
                }
                let reg0 = stack.provider2reg(v0)?;
                let reg1 = stack.provider2reg(v1)?;
                self.bump_fuel_consumption(fuel_info, FuelCostsProvider::base)?;
                let instr = self.push_instr(Instruction::copy2_ext(results.span(), reg0, reg1))?;
                Ok(Some(instr))
            }
            [v0, v1, rest @ ..] => {
                debug_assert!(!rest.is_empty());
                // Note: The fuel for copies might result in 0 charges if there aren't
                //       enough copies to account for at least 1 fuel. Therefore we need
                //       to also bump by `FuelCosts::base` to charge at least 1 fuel.
                self.bump_fuel_consumption(fuel_info, FuelCostsProvider::base)?;
                self.bump_fuel_consumption(fuel_info, |costs| {
                    costs.fuel_for_copying_values(rest.len() as u64 + 3)
                })?;
                if let Some(values) = BoundedRegSpan::from_providers(values) {
                    let make_instr = match Self::has_overlapping_copy_spans(
                        results.span(),
                        values.span(),
                        values.len(),
                    ) {
                        true => Instruction::copy_span,
                        false => Instruction::copy_span_non_overlapping,
                    };
                    let instr =
                        self.push_instr(make_instr(results.span(), values.span(), values.len()))?;
                    return Ok(Some(instr));
                }
                let make_instr = match Self::has_overlapping_copies(results, values) {
                    true => Instruction::copy_many_ext,
                    false => Instruction::copy_many_non_overlapping_ext,
                };
                let reg0 = stack.provider2reg(v0)?;
                let reg1 = stack.provider2reg(v1)?;
                let instr = self.push_instr(make_instr(results.span(), reg0, reg1))?;
                self.encode_register_list(stack, rest)?;
                Ok(Some(instr))
            }
        }
    }

    /// Returns `true` if `copy_span results <- values` has overlapping copies.
    ///
    /// # Examples
    ///
    /// - `[ ]`: empty never overlaps
    /// - `[ 1 <- 0 ]`: single element never overlaps
    /// - `[ 0 <- 1, 1 <- 2, 2 <- 3 ]``: no overlap
    /// - `[ 1 <- 0, 2 <- 1 ]`: overlaps!
    pub fn has_overlapping_copy_spans(results: RegSpan, values: RegSpan, len: u16) -> bool {
        RegSpan::has_overlapping_copies(results, values, len)
    }

    /// Returns `true` if the `copy results <- values` instruction has overlaps.
    ///
    /// # Examples
    ///
    /// - The sequence `[ 0 <- 1, 1 <- 1, 2 <- 4 ]` has no overlapping copies.
    /// - The sequence `[ 0 <- 1, 1 <- 0 ]` has overlapping copies since register `0`
    ///   is written to in the first copy but read from in the next.
    /// - The sequence `[ 3 <- 1, 4 <- 2, 5 <- 3 ]` has overlapping copies since register `3`
    ///   is written to in the first copy but read from in the third.
    pub fn has_overlapping_copies(results: BoundedRegSpan, values: &[TypedProvider]) -> bool {
        debug_assert_eq!(usize::from(results.len()), values.len());
        if results.is_empty() {
            // Note: An empty set of copies can never have overlapping copies.
            return false;
        }
        let result0 = results.span().head();
        for (result, value) in results.iter().zip(values) {
            // Note: We only have to check the register case since constant value
            //       copies can never overlap.
            if let TypedProvider::Register(value) = *value {
                // If the register `value` index is within range of `result0..result`
                // then its value has been overwritten by previous copies.
                if result0 <= value && value < result {
                    return true;
                }
            }
        }
        false
    }

    /// Bumps consumed fuel for [`Instruction::ConsumeFuel`] of `instr` by `delta`.
    ///
    /// # Errors
    ///
    /// If consumed fuel is out of bounds after this operation.
    pub fn bump_fuel_consumption<F>(&mut self, fuel_info: &FuelInfo, f: F) -> Result<(), Error>
    where
        F: FnOnce(&FuelCostsProvider) -> u64,
    {
        let FuelInfo::Some { costs, instr } = fuel_info else {
            // Fuel metering is disabled so we can bail out.
            return Ok(());
        };
        let fuel_consumed = f(costs);
        self.instrs
            .get_mut(*instr)
            .bump_fuel_consumption(fuel_consumed)?;
        Ok(())
    }

    /// Encodes an unconditional `return` instruction.
    pub fn encode_return(
        &mut self,
        stack: &mut ValueStack,
        values: &[TypedProvider],
        fuel_info: &FuelInfo,
    ) -> Result<(), Error> {
        let instr = match values {
            [] => Instruction::Return,
            [TypedProvider::Register(reg)] => Instruction::return_reg(*reg),
            [TypedProvider::Const(value)] => match value.ty() {
                ValType::I32 => Instruction::return_imm32(i32::from(*value)),
                ValType::I64 => match <Const32<i64>>::try_from(i64::from(*value)).ok() {
                    Some(value) => Instruction::return_i64imm32(value),
                    None => Instruction::return_reg(stack.alloc_const(*value)?),
                },
                ValType::F32 => Instruction::return_imm32(f32::from(*value)),
                ValType::F64 => match <Const32<f64>>::try_from(f64::from(*value)).ok() {
                    Some(value) => Instruction::return_f64imm32(value),
                    None => Instruction::return_reg(stack.alloc_const(*value)?),
                },
                ValType::V128 | ValType::FuncRef | ValType::ExternRef => {
                    Instruction::return_reg(stack.alloc_const(*value)?)
                }
            },
            [v0, v1] => {
                let reg0 = stack.provider2reg(v0)?;
                let reg1 = stack.provider2reg(v1)?;
                Instruction::return_reg2_ext(reg0, reg1)
            }
            [v0, v1, v2] => {
                let reg0 = stack.provider2reg(v0)?;
                let reg1 = stack.provider2reg(v1)?;
                let reg2 = stack.provider2reg(v2)?;
                Instruction::return_reg3_ext(reg0, reg1, reg2)
            }
            [v0, v1, v2, rest @ ..] => {
                debug_assert!(!rest.is_empty());
                // Note: The fuel for return values might result in 0 charges if there aren't
                //       enough return values to account for at least 1 fuel. Therefore we need
                //       to also bump by `FuelCostsProvider::base` to charge at least 1 fuel.
                self.bump_fuel_consumption(fuel_info, FuelCostsProvider::base)?;
                self.bump_fuel_consumption(fuel_info, |costs| {
                    costs.fuel_for_copying_values(rest.len() as u64 + 3)
                })?;
                if let Some(span) = BoundedRegSpan::from_providers(values) {
                    self.push_instr(Instruction::return_span(span))?;
                    return Ok(());
                }
                let reg0 = stack.provider2reg(v0)?;
                let reg1 = stack.provider2reg(v1)?;
                let reg2 = stack.provider2reg(v2)?;
                self.push_instr(Instruction::return_many_ext(reg0, reg1, reg2))?;
                self.encode_register_list(stack, rest)?;
                return Ok(());
            }
        };
        self.bump_fuel_consumption(fuel_info, FuelCostsProvider::base)?;
        self.push_instr(instr)?;
        Ok(())
    }

    /// Encode the given slice of [`TypedProvider`] as a list of [`Reg`].
    ///
    /// # Note
    ///
    /// This is used for the following n-ary instructions:
    ///
    /// - [`Instruction::ReturnMany`]
    /// - [`Instruction::CopyMany`]
    /// - [`Instruction::CallInternal`]
    /// - [`Instruction::CallImported`]
    /// - [`Instruction::CallIndirect`]
    /// - [`Instruction::ReturnCallInternal`]
    /// - [`Instruction::ReturnCallImported`]
    /// - [`Instruction::ReturnCallIndirect`]
    pub fn encode_register_list(
        &mut self,
        stack: &mut ValueStack,
        inputs: &[TypedProvider],
    ) -> Result<(), Error> {
        let mut remaining = inputs;
        let instr = loop {
            match remaining {
                [] => return Ok(()),
                [v0] => break Instruction::register(stack.provider2reg(v0)?),
                [v0, v1] => {
                    break Instruction::register2_ext(
                        stack.provider2reg(v0)?,
                        stack.provider2reg(v1)?,
                    )
                }
                [v0, v1, v2] => {
                    break Instruction::register3_ext(
                        stack.provider2reg(v0)?,
                        stack.provider2reg(v1)?,
                        stack.provider2reg(v2)?,
                    );
                }
                [v0, v1, v2, rest @ ..] => {
                    self.instrs.push(Instruction::register_list_ext(
                        stack.provider2reg(v0)?,
                        stack.provider2reg(v1)?,
                        stack.provider2reg(v2)?,
                    ))?;
                    remaining = rest;
                }
            };
        };
        self.instrs.push(instr)?;
        Ok(())
    }

    /// Encode a `local.set` or `local.tee` instruction.
    ///
    /// This also applies an optimization in that the previous instruction
    /// result is replaced with the `local` [`Reg`] instead of encoding
    /// another `copy` instruction if the `local.set` or `local.tee` belongs
    /// to the same basic block.
    ///
    /// # Note
    ///
    /// - If `value` is a [`Reg`] it usually is equal to the
    ///   result [`Reg`] of the previous instruction.
    pub fn encode_local_set(
        &mut self,
        stack: &mut ValueStack,
        res: &ModuleHeader,
        local: Reg,
        value: TypedProvider,
        preserved: Option<Reg>,
        fuel_info: FuelInfo,
    ) -> Result<(), Error> {
        fn fallback_case(
            this: &mut InstrEncoder,
            stack: &mut ValueStack,
            local: Reg,
            value: TypedProvider,
            preserved: Option<Reg>,
            fuel_info: FuelInfo,
        ) -> Result<(), Error> {
            if let Some(preserved) = preserved {
                this.bump_fuel_consumption(&fuel_info, FuelCostsProvider::base)?;
                let preserve_instr = this.push_instr(Instruction::copy(preserved, local))?;
                this.notify_preserved_register(preserve_instr);
            }
            this.encode_copy(stack, local, value, &fuel_info)?;
            Ok(())
        }

        debug_assert!(matches!(
            stack.get_register_space(local),
            RegisterSpace::Local
        ));
        let TypedProvider::Register(returned_value) = value else {
            // Cannot apply the optimization for `local.set C` where `C` is a constant value.
            return fallback_case(self, stack, local, value, preserved, fuel_info);
        };
        if matches!(
            stack.get_register_space(returned_value),
            RegisterSpace::Local | RegisterSpace::Preserve
        ) {
            // Can only apply the optimization if the returned value of `last_instr`
            // is _NOT_ itself a local register due to observable behavior or already preserved.
            return fallback_case(self, stack, local, value, preserved, fuel_info);
        }
        let Some(last_instr) = self.last_instr else {
            // Can only apply the optimization if there is a previous instruction
            // to replace its result register instead of emitting a copy.
            return fallback_case(self, stack, local, value, preserved, fuel_info);
        };
        if preserved.is_some() && last_instr.distance(self.instrs.next_instr()) >= 4 {
            // We avoid applying the optimization if the last instruction
            // has a very large encoding, e.g. for function calls with lots
            // of parameters. This is because the optimization while also
            // preserving a local register requires costly shifting all
            // instruction words of the last instruction.
            // Thankfully most instructions are small enough.
            return fallback_case(self, stack, local, value, preserved, fuel_info);
        }
        if let Some(preserved) = preserved {
            let mut last_instr_uses_preserved = false;
            for instr in self.instrs.get_slice_at_mut(last_instr).iter_mut() {
                instr.visit_input_registers(|input| {
                    if *input == preserved {
                        last_instr_uses_preserved = true;
                    }
                });
            }
            if last_instr_uses_preserved {
                // Note: we cannot apply the local.set preservation optimization since this would
                //       siltently overwrite inputs of the last encoded instruction.
                return fallback_case(self, stack, local, value, Some(preserved), fuel_info);
            }
        }
        if !self
            .instrs
            .get_mut(last_instr)
            .relink_result(res, local, returned_value)?
        {
            // It was not possible to relink the result of `last_instr` therefore we fallback.
            return fallback_case(self, stack, local, value, preserved, fuel_info);
        }
        if let Some(preserved) = preserved {
            // We were able to apply the optimization.
            // Preservation requires the copy to be before the optimized last instruction.
            // Therefore we need to push the preservation `copy` instruction before it.
            self.bump_fuel_consumption(&fuel_info, FuelCostsProvider::base)?;
            let shifted_last_instr = self
                .instrs
                .push_before(last_instr, Instruction::copy(preserved, local))?;
            self.notify_preserved_register(last_instr);
            self.last_instr = Some(shifted_last_instr);
        }
        Ok(())
    }

    /// Notifies the [`InstrEncoder`] that a local variable has been preserved.
    ///
    /// # Note
    ///
    /// This is an optimization that we perform to avoid or minimize the work
    /// done in [`InstrEncoder::defrag_registers`] by either avoiding defragmentation
    /// entirely if no local preservations took place or by at least only defragmenting
    /// the slice of instructions that could have been affected by it but not all
    /// encoded instructions.
    /// Only instructions that are encoded after the preservation could have been affected.
    ///
    /// This will ignore any preservation notifications after the first one.
    pub fn notify_preserved_register(&mut self, preserve_instr: Instr) {
        {
            let preserved = self.instrs.get(preserve_instr);
            debug_assert!(
                matches!(
                    preserved,
                    Instruction::Copy { .. }
                        | Instruction::Copy2 { .. }
                        | Instruction::CopySpanNonOverlapping { .. }
                        | Instruction::CopyManyNonOverlapping { .. }
                ),
                "a preserve instruction is always a register copy instruction but found: {preserved:?}",
            );
        }
        if self.notified_preservation.is_none() {
            self.notified_preservation = Some(preserve_instr);
        }
    }

    /// Defragments storage-space registers of all encoded [`Instruction`].
    pub fn defrag_registers(&mut self, stack: &mut ValueStack) -> Result<(), Error> {
        stack.finalize_alloc();
        if let Some(notified_preserved) = self.notified_preservation {
            for instr in self.instrs.get_slice_at_mut(notified_preserved) {
                instr.visit_input_registers(|reg| *reg = stack.defrag_register(*reg));
            }
        }
        Ok(())
    }

    /// Tries to fuse a Wasm `i32.eqz` (or `i32.eq` with 0 `rhs` value) instruction.
    ///
    /// Returns
    ///
    /// - `Ok(true)` if the intruction fusion was successful.
    /// - `Ok(false)` if instruction fusion could not be applied.
    /// - `Err(_)` if an error occurred.
    pub fn fuse_eqz<T: WasmInteger>(
        &mut self,
        stack: &mut ValueStack,
        lhs: Reg,
        rhs: T,
    ) -> Result<bool, Error> {
        if !rhs.eq_zero() {
            // Case: `rhs` needs to be zero to apply this optimization.
            return Ok(false);
        }
        let Some(last_instr) = self.last_instr else {
            // If there is no last instruction there is no comparison instruction to negate.
            return Ok(false);
        };
        let last_instruction = *self.instrs.get(last_instr);
        let Some(result) = last_instruction.result() else {
            // All negatable instructions have a single result register.
            return Ok(false);
        };
        if matches!(stack.get_register_space(result), RegisterSpace::Local) {
            // The instruction stores its result into a local variable which
            // is an observable side effect which we are not allowed to mutate.
            return Ok(false);
        }
        if result != lhs {
            // The result of the instruction and the `lhs` are not equal
            // thus indicating that we cannot fuse the instructions.
            return Ok(false);
        }
        let Some(negated) = last_instruction.negate_cmp_instr() else {
            // Last instruction is unable to be negated.
            return Ok(false);
        };
        _ = mem::replace(self.instrs.get_mut(last_instr), negated);
        stack.push_register(result)?;
        Ok(true)
    }

    /// Tries to fuse a Wasm `i32.ne` instruction with 0 `rhs` value.
    ///
    /// Returns
    ///
    /// - `Ok(true)` if the intruction fusion was successful.
    /// - `Ok(false)` if instruction fusion could not be applied.
    /// - `Err(_)` if an error occurred.
    pub fn fuse_nez<T: WasmInteger>(
        &mut self,
        stack: &mut ValueStack,
        lhs: Reg,
        rhs: T,
    ) -> Result<bool, Error> {
        if !rhs.eq_zero() {
            // Case: `rhs` needs to be zero to apply this optimization.
            return Ok(false);
        }
        let Some(last_instr) = self.last_instr else {
            // If there is no last instruction there is no comparison instruction to negate.
            return Ok(false);
        };
        let last_instruction = *self.instrs.get(last_instr);
        let Some(result) = last_instruction.result() else {
            // All negatable instructions have a single result register.
            return Ok(false);
        };
        if matches!(stack.get_register_space(result), RegisterSpace::Local) {
            // The instruction stores its result into a local variable which
            // is an observable side effect which we are not allowed to mutate.
            return Ok(false);
        }
        if result != lhs {
            // The result of the instruction and the `lhs` are not equal
            // thus indicating that we cannot fuse the instructions.
            return Ok(false);
        }
        let Some(logicalized) = last_instruction.logicalize_cmp_instr() else {
            // Last instruction is unable to be negated.
            return Ok(false);
        };
        _ = mem::replace(self.instrs.get_mut(last_instr), logicalized);
        stack.push_register(result)?;
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
        stack: &mut ValueStack,
        select_result: Reg,
        select_condition: Reg,
    ) -> Option<(Instruction, bool)> {
        let Some(last_instr) = self.last_instr else {
            // If there is no last instruction there is no comparison instruction to negate.
            return None;
        };
        let last_instruction = *self.instrs.get(last_instr);
        let Some(last_result) = last_instruction.result() else {
            // All negatable instructions have a single result register.
            return None;
        };
        if matches!(stack.get_register_space(last_result), RegisterSpace::Local) {
            // The instruction stores its result into a local variable which
            // is an observable side effect which we are not allowed to mutate.
            return None;
        }
        if last_result != select_condition {
            // The result of the last instruction and the select's `condition`
            // are not equal thus indicating that we cannot fuse the instructions.
            return None;
        }
        let (fused_select, swap_operands) =
            last_instruction.try_into_cmp_select_instr(select_result)?;
        _ = mem::replace(self.instrs.get_mut(last_instr), fused_select);
        Some((fused_select, swap_operands))
    }

    /// Create an [`Instruction::BranchCmpFallback`].
    fn make_branch_cmp_fallback(
        stack: &mut ValueStack,
        cmp: Comparator,
        lhs: Reg,
        rhs: Reg,
        offset: BranchOffset,
    ) -> Result<Instruction, Error> {
        let params = stack.alloc_const(ComparatorAndOffset::new(cmp, offset))?;
        Ok(Instruction::branch_cmp_fallback(lhs, rhs, params))
    }

    /// Encodes a `branch_eqz` instruction and tries to fuse it with a previous comparison instruction.
    pub fn encode_branch_eqz(
        &mut self,
        stack: &mut ValueStack,
        condition: Reg,
        label: LabelRef,
    ) -> Result<(), Error> {
        let Some(last_instr) = self.last_instr else {
            return self.encode_branch_eqz_unopt(stack, condition, label);
        };
        let fused_instr =
            self.try_fuse_branch_cmp_for_instr(stack, last_instr, condition, label, true)?;
        if let Some(fused_instr) = fused_instr {
            _ = mem::replace(self.instrs.get_mut(last_instr), fused_instr);
            return Ok(());
        }
        self.encode_branch_eqz_unopt(stack, condition, label)
    }

    /// Encode an unoptimized `branch_eqz` instruction.
    ///
    /// This is used as fallback whenever fusing compare and branch instructions is not possible.
    fn encode_branch_eqz_unopt(
        &mut self,
        stack: &mut ValueStack,
        condition: Reg,
        label: LabelRef,
    ) -> Result<(), Error> {
        let offset = self.try_resolve_label(label)?;
        let instr = match BranchOffset16::try_from(offset) {
            Ok(offset) => Instruction::branch_i32_eq_imm16(condition, 0, offset),
            Err(_) => {
                let zero = stack.alloc_const(0_i32)?;
                InstrEncoder::make_branch_cmp_fallback(
                    stack,
                    Comparator::I32Eq,
                    condition,
                    zero,
                    offset,
                )?
            }
        };
        self.push_instr(instr)?;
        Ok(())
    }

    /// Encodes a `branch_nez` instruction and tries to fuse it with a previous comparison instruction.
    pub fn encode_branch_nez(
        &mut self,
        stack: &mut ValueStack,
        condition: Reg,
        label: LabelRef,
    ) -> Result<(), Error> {
        let Some(last_instr) = self.last_instr else {
            return self.encode_branch_nez_unopt(stack, condition, label);
        };
        let fused_instr =
            self.try_fuse_branch_cmp_for_instr(stack, last_instr, condition, label, false)?;
        if let Some(fused_instr) = fused_instr {
            _ = mem::replace(self.instrs.get_mut(last_instr), fused_instr);
            return Ok(());
        }
        self.encode_branch_nez_unopt(stack, condition, label)
    }

    /// Try to fuse [`Instruction`] at `instr` into a branch+cmp instruction.
    ///
    /// Returns `Ok(Some)` if successful.
    fn try_fuse_branch_cmp_for_instr(
        &mut self,
        stack: &mut ValueStack,
        last_instr: Instr,
        condition: Reg,
        label: LabelRef,
        negate: bool,
    ) -> Result<Option<Instruction>, Error> {
        let last_instruction = *self.instrs.get(last_instr);
        let Some(result) = last_instruction.compare_result() else {
            return Ok(None);
        };
        if matches!(stack.get_register_space(result), RegisterSpace::Local) {
            // We need to filter out instructions that store their result
            // into a local register slot because they introduce observable behavior
            // which a fused cmp+branch instruction would remove.
            return Ok(None);
        }
        if result != condition {
            // We cannot fuse the instructions since the result of the compare instruction
            // does not match the input of the conditional branch instruction.
            return Ok(None);
        }
        let last_instruction = match negate {
            true => match last_instruction.negate_cmp_instr() {
                Some(negated) => negated,
                None => return Ok(None),
            },
            false => last_instruction,
        };
        let offset = self.try_resolve_label_for(label, last_instr)?;
        last_instruction.try_into_cmp_branch_instr(offset, stack)
    }

    /// Encode an unoptimized `branch_nez` instruction.
    ///
    /// This is used as fallback whenever fusing compare and branch instructions is not possible.
    fn encode_branch_nez_unopt(
        &mut self,
        stack: &mut ValueStack,
        condition: Reg,
        label: LabelRef,
    ) -> Result<(), Error> {
        let offset = self.try_resolve_label(label)?;
        let instr = match BranchOffset16::try_from(offset) {
            Ok(offset) => Instruction::branch_i32_ne_imm16(condition, 0, offset),
            Err(_) => {
                let zero = stack.alloc_const(0_i32)?;
                InstrEncoder::make_branch_cmp_fallback(
                    stack,
                    Comparator::I32Ne,
                    condition,
                    zero,
                    offset,
                )?
            }
        };
        self.push_instr(instr)?;
        Ok(())
    }
}

/// Extension trait to update the branch offset of an [`Instruction`].
trait UpdateBranchOffset {
    /// Updates the [`BranchOffset`] for the branch [`Instruction].
    ///
    /// # Panics
    ///
    /// If `self` is not a branch [`Instruction`].
    fn update_branch_offset(
        &mut self,
        stack: &mut ValueStack,
        new_offset: BranchOffset,
    ) -> Result<(), Error>;
}

impl UpdateBranchOffset for Instruction {
    #[rustfmt::skip]
    fn update_branch_offset(&mut self, stack: &mut ValueStack, new_offset: BranchOffset) -> Result<(), Error> {
        use Instruction as I;
        match self {
            Instruction::Branch { offset } |
            Instruction::BranchTableTarget { offset, .. } |
            Instruction::BranchTableTargetNonOverlapping { offset, .. } => {
                offset.init(new_offset);
                return Ok(())
            }
            _ => {}
        };
        let update_status = match self {
            I::BranchI32And { offset, .. } |
            I::BranchI32Or { offset, .. } |
            I::BranchI32Xor { offset, .. } |
            I::BranchI32Nand { offset, .. } |
            I::BranchI32Nor { offset, .. } |
            I::BranchI32Xnor { offset, .. } |
            I::BranchI32Eq { offset, .. } |
            I::BranchI32Ne { offset, .. } |
            I::BranchI32LtS { offset, .. } |
            I::BranchI32LtU { offset, .. } |
            I::BranchI32LeS { offset, .. } |
            I::BranchI32LeU { offset, .. } |
            I::BranchI64And { offset, .. } |
            I::BranchI64Or { offset, .. } |
            I::BranchI64Xor { offset, .. } |
            I::BranchI64Nand { offset, .. } |
            I::BranchI64Nor { offset, .. } |
            I::BranchI64Xnor { offset, .. } |
            I::BranchI64Eq { offset, .. } |
            I::BranchI64Ne { offset, .. } |
            I::BranchI64LtS { offset, .. } |
            I::BranchI64LtU { offset, .. } |
            I::BranchI64LeS { offset, .. } |
            I::BranchI64LeU { offset, .. } |
            I::BranchF32Eq { offset, .. } |
            I::BranchF32Ne { offset, .. } |
            I::BranchF32Lt { offset, .. } |
            I::BranchF32Le { offset, .. } |
            I::BranchF32NotLt { offset, .. } |
            I::BranchF32NotLe { offset, .. } |
            I::BranchF64Eq { offset, .. } |
            I::BranchF64Ne { offset, .. } |
            I::BranchF64Lt { offset, .. } |
            I::BranchF64Le { offset, .. } |
            I::BranchF64NotLt { offset, .. } |
            I::BranchF64NotLe { offset, .. } |
            I::BranchI32AndImm16 { offset, .. } |
            I::BranchI32OrImm16 { offset, .. } |
            I::BranchI32XorImm16 { offset, .. } |
            I::BranchI32NandImm16 { offset, .. } |
            I::BranchI32NorImm16 { offset, .. } |
            I::BranchI32XnorImm16 { offset, .. } |
            I::BranchI32EqImm16 { offset, .. } |
            I::BranchI32NeImm16 { offset, .. } |
            I::BranchI32LtSImm16Lhs { offset, .. } |
            I::BranchI32LtSImm16Rhs { offset, .. } |
            I::BranchI32LeSImm16Lhs { offset, .. } |
            I::BranchI32LeSImm16Rhs { offset, .. } |
            I::BranchI32LtUImm16Lhs { offset, .. } |
            I::BranchI32LtUImm16Rhs { offset, .. } |
            I::BranchI32LeUImm16Lhs { offset, .. } |
            I::BranchI32LeUImm16Rhs { offset, .. } |
            I::BranchI64AndImm16 { offset, .. } |
            I::BranchI64OrImm16 { offset, .. } |
            I::BranchI64XorImm16 { offset, .. } |
            I::BranchI64NandImm16 { offset, .. } |
            I::BranchI64NorImm16 { offset, .. } |
            I::BranchI64XnorImm16 { offset, .. } |
            I::BranchI64EqImm16 { offset, .. } |
            I::BranchI64NeImm16 { offset, .. } |
            I::BranchI64LtSImm16Lhs { offset, .. } |
            I::BranchI64LtSImm16Rhs { offset, .. } |
            I::BranchI64LeSImm16Lhs { offset, .. } |
            I::BranchI64LeSImm16Rhs { offset, .. } |
            I::BranchI64LtUImm16Lhs { offset, .. } |
            I::BranchI64LtUImm16Rhs { offset, .. } |
            I::BranchI64LeUImm16Lhs { offset, .. } |
            I::BranchI64LeUImm16Rhs { offset, .. } => {
                offset.init(new_offset)
            }
            unexpected => {
                panic!("expected a Wasmi branch+cmp instruction but found: {unexpected:?}")
            }
        };
        if update_status.is_err() {
            if let Some(fallback) = self.try_into_cmp_branch_fallback_instr(new_offset, stack)? {
                *self = fallback;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::TypedVal;

    fn bspan(reg: i16, len: u16) -> BoundedRegSpan {
        BoundedRegSpan::new(RegSpan::new(Reg::from(reg)), len)
    }

    #[test]
    fn has_overlapping_copies_works() {
        assert!(!InstrEncoder::has_overlapping_copies(bspan(0, 0), &[],));
        assert!(!InstrEncoder::has_overlapping_copies(
            bspan(0, 2),
            &[TypedProvider::register(0), TypedProvider::register(1),],
        ));
        assert!(!InstrEncoder::has_overlapping_copies(
            bspan(0, 2),
            &[
                TypedProvider::Const(TypedVal::from(10_i32)),
                TypedProvider::Const(TypedVal::from(20_i32)),
            ],
        ));
        assert!(InstrEncoder::has_overlapping_copies(
            bspan(0, 2),
            &[
                TypedProvider::Const(TypedVal::from(10_i32)),
                TypedProvider::register(0),
            ],
        ));
        assert!(InstrEncoder::has_overlapping_copies(
            bspan(0, 2),
            &[TypedProvider::register(0), TypedProvider::register(0),],
        ));
        assert!(InstrEncoder::has_overlapping_copies(
            bspan(3, 3),
            &[
                TypedProvider::register(2),
                TypedProvider::register(3),
                TypedProvider::register(2),
            ],
        ));
        assert!(InstrEncoder::has_overlapping_copies(
            bspan(3, 4),
            &[
                TypedProvider::register(-1),
                TypedProvider::register(10),
                TypedProvider::register(2),
                TypedProvider::register(4),
            ],
        ));
    }
}
