use super::{
    visit_register::VisitInputRegisters,
    FuelInfo,
    LabelRef,
    LabelRegistry,
    TypedProvider,
};
use crate::{
    core::{UntypedVal, ValType, F32},
    engine::{
        bytecode::{
            BinInstr,
            BinInstrImm16,
            BranchComparator,
            BranchOffset,
            BranchOffset16,
            ComparatorOffsetParam,
            Const16,
            Const32,
            Instruction,
            Provider,
            Register,
            RegisterSpan,
            RegisterSpanIter,
        },
        translator::{stack::RegisterSpace, ValueStack},
        FuelCosts,
    },
    module::ModuleHeader,
    Error,
};
use core::mem;
use std::vec::{Drain, Vec};

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
    fn get(&mut self, instr: Instr) -> &Instruction {
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
    /// and `local.tee` translation to replace the result [`Register`] of the
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
    pub fn push_instr(&mut self, instr: Instruction) -> Result<Instr, Error> {
        let last_instr = self.instrs.push(instr)?;
        self.last_instr = Some(last_instr);
        Ok(last_instr)
    }

    /// Appends the [`Instruction`] to the last [`Instruction`] created via [`InstrEncoder::push_instr`].
    ///
    /// # Note
    ///
    /// This is used primarily for [`Instruction`] words that are just carrying
    /// parameters for the [`Instruction`]. An example of this is [`Instruction::Const32`]
    /// carrying the `offset` parameter for [`Instruction::I32Load`].
    pub fn append_instr(&mut self, instr: Instruction) -> Result<Instr, Error> {
        self.instrs.push(instr)
    }

    /// Tries to merge `copy(result, value)` if the last instruction is a matching copy instruction.
    ///
    /// - Returns `None` if merging of the copy instruction was not possible.
    /// - Returns the `Instr` of the merged `copy2` instruction if merging was successful.
    fn merge_copy_instrs(&mut self, result: Register, value: TypedProvider) -> Option<Instr> {
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

        let merged_copy = Instruction::copy2(RegisterSpan::new(merged_result), value0, value1);
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
        result: Register,
        value: TypedProvider,
        fuel_info: FuelInfo,
    ) -> Result<Option<Instr>, Error> {
        /// Convenience to create an [`Instruction::Copy`] to copy a constant value.
        fn copy_imm(
            stack: &mut ValueStack,
            result: Register,
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
                ValType::FuncRef => copy_imm(stack, result, value)?,
                ValType::ExternRef => copy_imm(stack, result, value)?,
            },
        };
        self.bump_fuel_consumption(fuel_info, FuelCosts::base)?;
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
        mut results: RegisterSpanIter,
        values: &[TypedProvider],
        fuel_info: FuelInfo,
    ) -> Result<Option<Instr>, Error> {
        assert_eq!(results.len(), values.len());
        if let Some((TypedProvider::Register(value), rest)) = values.split_first() {
            if results.span().head() == *value {
                // Case: `result` and `value` are equal thus this is a no-op copy which we can avoid.
                //       Applied recursively we thereby remove all no-op copies at the start of the
                //       copy sequence until the first actual copy.
                results.next();
                return self.encode_copies(stack, results, rest, fuel_info);
            }
        }
        let result = results.span().head();
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
                let reg0 = Self::provider2reg(stack, v0)?;
                let reg1 = Self::provider2reg(stack, v1)?;
                self.bump_fuel_consumption(fuel_info, FuelCosts::base)?;
                let instr = self.push_instr(Instruction::copy2(results.span(), reg0, reg1))?;
                Ok(Some(instr))
            }
            [v0, v1, rest @ ..] => {
                debug_assert!(!rest.is_empty());
                // Note: The fuel for copies might result in 0 charges if there aren't
                //       enough copies to account for at least 1 fuel. Therefore we need
                //       to also bump by `FuelCosts::base` to charge at least 1 fuel.
                self.bump_fuel_consumption(fuel_info, FuelCosts::base)?;
                self.bump_fuel_consumption(fuel_info, |costs| {
                    costs.fuel_for_copies(rest.len() as u64 + 3)
                })?;
                if let Some(values) = RegisterSpanIter::from_providers(values) {
                    let make_instr = match Self::has_overlapping_copy_spans(
                        results.span(),
                        values.span(),
                        values.len(),
                    ) {
                        true => Instruction::copy_span,
                        false => Instruction::copy_span_non_overlapping,
                    };
                    let instr = self.push_instr(make_instr(
                        results.span(),
                        values.span(),
                        values.len_as_u16(),
                    ))?;
                    return Ok(Some(instr));
                }
                let make_instr = match Self::has_overlapping_copies(results, values) {
                    true => Instruction::copy_many,
                    false => Instruction::copy_many_non_overlapping,
                };
                let reg0 = Self::provider2reg(stack, v0)?;
                let reg1 = Self::provider2reg(stack, v1)?;
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
    fn has_overlapping_copy_spans(results: RegisterSpan, values: RegisterSpan, len: usize) -> bool {
        RegisterSpanIter::has_overlapping_copies(results.iter(len), values.iter(len))
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
    fn has_overlapping_copies(results: RegisterSpanIter, values: &[TypedProvider]) -> bool {
        debug_assert_eq!(results.len(), values.len());
        if results.is_empty() {
            // Note: An empty set of copies can never have overlapping copies.
            return false;
        }
        let result0 = results.span().head();
        for (result, value) in results.zip(values) {
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
    pub fn bump_fuel_consumption<F>(&mut self, fuel_info: FuelInfo, f: F) -> Result<(), Error>
    where
        F: FnOnce(&FuelCosts) -> u64,
    {
        let FuelInfo::Some { costs, instr } = fuel_info else {
            // Fuel metering is disabled so we can bail out.
            return Ok(());
        };
        let fuel_consumed = f(&costs);
        self.instrs
            .get_mut(instr)
            .bump_fuel_consumption(fuel_consumed)?;
        Ok(())
    }

    /// Encodes an unconditional `return` instruction.
    pub fn encode_return(
        &mut self,
        stack: &mut ValueStack,
        values: &[TypedProvider],
        fuel_info: FuelInfo,
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
                ValType::F32 => Instruction::return_imm32(F32::from(*value)),
                ValType::F64 => match <Const32<f64>>::try_from(f64::from(*value)).ok() {
                    Some(value) => Instruction::return_f64imm32(value),
                    None => Instruction::return_reg(stack.alloc_const(*value)?),
                },
                ValType::FuncRef | ValType::ExternRef => {
                    Instruction::return_reg(stack.alloc_const(*value)?)
                }
            },
            [v0, v1] => {
                let reg0 = Self::provider2reg(stack, v0)?;
                let reg1 = Self::provider2reg(stack, v1)?;
                Instruction::return_reg2(reg0, reg1)
            }
            [v0, v1, v2] => {
                let reg0 = Self::provider2reg(stack, v0)?;
                let reg1 = Self::provider2reg(stack, v1)?;
                let reg2 = Self::provider2reg(stack, v2)?;
                Instruction::return_reg3(reg0, reg1, reg2)
            }
            [v0, v1, v2, rest @ ..] => {
                debug_assert!(!rest.is_empty());
                // Note: The fuel for return values might result in 0 charges if there aren't
                //       enough return values to account for at least 1 fuel. Therefore we need
                //       to also bump by `FuelCosts::base` to charge at least 1 fuel.
                self.bump_fuel_consumption(fuel_info, FuelCosts::base)?;
                self.bump_fuel_consumption(fuel_info, |costs| {
                    costs.fuel_for_copies(rest.len() as u64 + 3)
                })?;
                if let Some(span) = RegisterSpanIter::from_providers(values) {
                    self.push_instr(Instruction::return_span(span))?;
                    return Ok(());
                }
                let reg0 = Self::provider2reg(stack, v0)?;
                let reg1 = Self::provider2reg(stack, v1)?;
                let reg2 = Self::provider2reg(stack, v2)?;
                self.push_instr(Instruction::return_many(reg0, reg1, reg2))?;
                self.encode_register_list(stack, rest)?;
                return Ok(());
            }
        };
        self.bump_fuel_consumption(fuel_info, FuelCosts::base)?;
        self.push_instr(instr)?;
        Ok(())
    }

    /// Encodes an conditional `return` instruction.
    pub fn encode_return_nez(
        &mut self,
        stack: &mut ValueStack,
        condition: Register,
        values: &[TypedProvider],
        fuel_info: FuelInfo,
    ) -> Result<(), Error> {
        // Note: We bump fuel unconditionally even if the conditional return is not taken.
        //       This is very conservative and may lead to more fuel costs than
        //       actually needed for the computation. We might revisit this decision
        //       later. An alternative solution would consume fuel during execution
        //       time only when the return is taken.
        let instr = match values {
            [] => Instruction::return_nez(condition),
            [TypedProvider::Register(reg)] => Instruction::return_nez_reg(condition, *reg),
            [TypedProvider::Const(value)] => match value.ty() {
                ValType::I32 => Instruction::return_nez_imm32(condition, i32::from(*value)),
                ValType::I64 => match <Const32<i64>>::try_from(i64::from(*value)).ok() {
                    Some(value) => Instruction::return_nez_i64imm32(condition, value),
                    None => Instruction::return_nez_reg(condition, stack.alloc_const(*value)?),
                },
                ValType::F32 => Instruction::return_nez_imm32(condition, F32::from(*value)),
                ValType::F64 => match <Const32<f64>>::try_from(f64::from(*value)).ok() {
                    Some(value) => Instruction::return_nez_f64imm32(condition, value),
                    None => Instruction::return_nez_reg(condition, stack.alloc_const(*value)?),
                },
                ValType::FuncRef | ValType::ExternRef => {
                    Instruction::return_nez_reg(condition, stack.alloc_const(*value)?)
                }
            },
            [v0, v1] => {
                let reg0 = Self::provider2reg(stack, v0)?;
                let reg1 = Self::provider2reg(stack, v1)?;
                Instruction::return_nez_reg2(condition, reg0, reg1)
            }
            [v0, v1, rest @ ..] => {
                debug_assert!(!rest.is_empty());
                // Note: The fuel for return values might result in 0 charges if there aren't
                //       enough return values to account for at least 1 fuel. Therefore we need
                //       to also bump by `FuelCosts::base` to charge at least 1 fuel.
                self.bump_fuel_consumption(fuel_info, FuelCosts::base)?;
                self.bump_fuel_consumption(fuel_info, |costs| {
                    costs.fuel_for_copies(rest.len() as u64 + 3)
                })?;
                if let Some(span) = RegisterSpanIter::from_providers(values) {
                    self.push_instr(Instruction::return_nez_span(condition, span))?;
                    return Ok(());
                }
                let reg0 = Self::provider2reg(stack, v0)?;
                let reg1 = Self::provider2reg(stack, v1)?;
                self.push_instr(Instruction::return_nez_many(condition, reg0, reg1))?;
                self.encode_register_list(stack, rest)?;
                return Ok(());
            }
        };
        self.bump_fuel_consumption(fuel_info, FuelCosts::base)?;
        self.push_instr(instr)?;
        Ok(())
    }

    /// Converts a [`TypedProvider`] into a [`Register`].
    ///
    /// This allocates constant values for [`TypedProvider::Const`].
    fn provider2reg(stack: &mut ValueStack, provider: &TypedProvider) -> Result<Register, Error> {
        match provider {
            Provider::Register(register) => Ok(*register),
            Provider::Const(value) => stack.alloc_const(*value),
        }
    }

    /// Encode the given slice of [`TypedProvider`] as a list of [`Register`].
    ///
    /// # Note
    ///
    /// This is used for the following n-ary instructions:
    ///
    /// - [`Instruction::ReturnMany`]
    /// - [`Instruction::ReturnNezMany`]
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
        loop {
            match remaining {
                [] => return Ok(()),
                [v0] => {
                    let v0 = Self::provider2reg(stack, v0)?;
                    self.instrs.push(Instruction::register(v0))?;
                    return Ok(());
                }
                [v0, v1] => {
                    let v0 = Self::provider2reg(stack, v0)?;
                    let v1 = Self::provider2reg(stack, v1)?;
                    self.instrs.push(Instruction::register2(v0, v1))?;
                    return Ok(());
                }
                [v0, v1, v2] => {
                    let v0 = Self::provider2reg(stack, v0)?;
                    let v1 = Self::provider2reg(stack, v1)?;
                    let v2 = Self::provider2reg(stack, v2)?;
                    self.instrs.push(Instruction::register3(v0, v1, v2))?;
                    return Ok(());
                }
                [v0, v1, v2, rest @ ..] => {
                    let v0 = Self::provider2reg(stack, v0)?;
                    let v1 = Self::provider2reg(stack, v1)?;
                    let v2 = Self::provider2reg(stack, v2)?;
                    self.instrs.push(Instruction::register_list(v0, v1, v2))?;
                    remaining = rest;
                }
            }
        }
    }

    /// Encode a `local.set` or `local.tee` instruction.
    ///
    /// This also applies an optimization in that the previous instruction
    /// result is replaced with the `local` [`Register`] instead of encoding
    /// another `copy` instruction if the `local.set` or `local.tee` belongs
    /// to the same basic block.
    ///
    /// # Note
    ///
    /// - If `value` is a [`Register`] it usually is equal to the
    ///   result [`Register`] of the previous instruction.
    pub fn encode_local_set(
        &mut self,
        stack: &mut ValueStack,
        res: &ModuleHeader,
        local: Register,
        value: TypedProvider,
        preserved: Option<Register>,
        fuel_info: FuelInfo,
    ) -> Result<(), Error> {
        fn fallback_case(
            this: &mut InstrEncoder,
            stack: &mut ValueStack,
            local: Register,
            value: TypedProvider,
            preserved: Option<Register>,
            fuel_info: FuelInfo,
        ) -> Result<(), Error> {
            if let Some(preserved) = preserved {
                this.bump_fuel_consumption(fuel_info, FuelCosts::base)?;
                let preserve_instr = this.push_instr(Instruction::copy(preserved, local))?;
                this.notify_preserved_register(preserve_instr);
            }
            this.encode_copy(stack, local, value, fuel_info)?;
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
            RegisterSpace::Local
        ) {
            // Can only apply the optimization if the returned value of `last_instr`
            // is _NOT_ itself a local register due to observable behavior.
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
            self.bump_fuel_consumption(fuel_info, FuelCosts::base)?;
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
                "a preserve instruction is always a register copy instruction but found: {:?}",
                preserved,
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

    /// Fuses a `global.get 0` and an `i32.add_imm` if possible.
    ///
    /// Returns `true` if `Instruction` fusion was successful, `false` otherwise.
    pub fn fuse_global_get_i32_add_imm(
        &mut self,
        lhs: Register,
        rhs: i32,
        stack: &mut ValueStack,
    ) -> Result<bool, Error> {
        let Some(last_instr) = self.last_instr else {
            // Without a last instruction there is no way to fuse.
            return Ok(false);
        };
        let &Instruction::GlobalGet { result, global } = self.instrs.get(last_instr) else {
            // It is only possible to fuse an `GlobalGet` with a `I32AddImm` instruction.
            return Ok(false);
        };
        if global.to_u32() != 0 {
            // There only is an optimized instruction for a global index of 0.
            // This is because most Wasm producers use the global at index 0 for their shadow stack.
            return Ok(false);
        }
        if !matches!(stack.get_register_space(result), RegisterSpace::Dynamic) {
            // Due to observable state it is impossible to fuse `GlobalGet` that has a non-`dynamic` result.
            return Ok(false);
        };
        if result != lhs {
            // The `input` to `I32AddImm` must be the same as the result of `GetGlobal`.
            return Ok(false);
        }
        let rhs = <Const32<i32>>::from(rhs);
        let fused_instr = Instruction::i32_add_imm_from_global_0(result, rhs);
        _ = mem::replace(self.instrs.get_mut(last_instr), fused_instr);
        stack.push_register(result)?;
        Ok(true)
    }

    /// Fuses the `global.set` instruction with its previous instruction if possible.
    ///
    /// Returns `true` if `Instruction` fusion was successful, `false` otherwise.
    pub fn fuse_global_set(
        &mut self,
        global_index: u32,
        input: Register,
        stack: &mut ValueStack,
    ) -> bool {
        /// Returns `true` if the previous [`Instruction`] and
        /// its `result` can be fused with the `global.set` and its `input`.
        fn is_fusable(stack: &mut ValueStack, result: Register, input: Register) -> bool {
            if !matches!(stack.get_register_space(result), RegisterSpace::Dynamic) {
                // Due to observable state it is impossible to fuse an instruction that has a non-`dynamic` result.
                return false;
            };
            if result != input {
                // It is only possible to fuse the instructions if the `input` of `global.set`
                // matches the `result` of the previous to-be-fused instruction.
                return false;
            }
            true
        }

        if global_index != 0 {
            // There only is an optimized instruction for a global index of 0.
            // This is because most Wasm producers use the global at index 0 for their shadow stack.
            return false;
        }
        let Some(last_instr) = self.last_instr else {
            // Without a last instruction there is no way to fuse.
            return false;
        };
        let fused_instr = match self.instrs.get(last_instr) {
            Instruction::I32Add(instr) => {
                if !is_fusable(stack, instr.result, input) {
                    return false;
                }
                let Some(value) = stack.resolve_func_local_const(instr.rhs).map(i32::from) else {
                    // It is only possiblet o fuse `I32Add` if its `rhs` is a function local constant.
                    return false;
                };
                let lhs = instr.lhs;
                let rhs = <Const32<i32>>::from(value);
                Instruction::i32_add_imm_into_global_0(lhs, rhs)
            }
            Instruction::I32Sub(instr) => {
                if !is_fusable(stack, instr.result, input) {
                    return false;
                }
                let Some(value) = stack.resolve_func_local_const(instr.rhs).map(i32::from) else {
                    // It is only possiblet o fuse `I32Add` if its `rhs` is a function local constant.
                    return false;
                };
                let lhs = instr.lhs;
                let rhs = <Const32<i32>>::from(-value);
                Instruction::i32_add_imm_into_global_0(lhs, rhs)
            }
            Instruction::I32AddImm16(instr) => {
                if !is_fusable(stack, instr.result, input) {
                    return false;
                }
                let lhs = instr.reg_in;
                let rhs = <Const32<i32>>::from(i32::from(instr.imm_in));
                Instruction::i32_add_imm_into_global_0(lhs, rhs)
            }
            &Instruction::I32AddImmFromGlobal0 { result, rhs } => {
                if result != input {
                    // The `input` to `GlobalSet` must be the same as the result of `I32AddImmFromGlobal0`.
                    return false;
                }
                Instruction::i32_add_imm_inout_global_0(result, rhs)
            }
            _ => return false,
        };
        _ = mem::replace(self.instrs.get_mut(last_instr), fused_instr);
        true
    }

    /// Translates a Wasm `i32.eqz` instruction.
    ///
    /// Tries to fuse `i32.eqz` with a previous `i32.{and,or,xor}` instruction if possible.
    /// Returns `true` if it was possible to fuse the `i32.eqz` instruction.
    pub fn fuse_i32_eqz(&mut self, stack: &mut ValueStack) -> bool {
        /// Fuse a `i32.{and,or,xor}` instruction with `i32.eqz`.
        macro_rules! fuse {
            ($instr:ident, $stack:ident, $input:ident, $make_fuse:expr) => {{
                if matches!(
                    $stack.get_register_space($instr.result),
                    RegisterSpace::Local
                ) {
                    // The instruction stores its result into a local variable which
                    // is an observable side effect which we are not allowed to mutate.
                    return false;
                }
                if $instr.result != $input {
                    // The result of the instruction and the current input are not equal
                    // thus indicating that we cannot fuse the instructions.
                    return false;
                }
                $make_fuse($instr.result, $instr.lhs, $instr.rhs)
            }};
        }

        /// Fuse a `i32.{and,or,xor}` instruction with 16-bit encoded immediate parameter with `i32.eqz`.
        macro_rules! fuse_imm16 {
            ($instr:ident, $stack:ident, $input:ident, $make_fuse:expr) => {{
                if matches!(
                    $stack.get_register_space($instr.result),
                    RegisterSpace::Local
                ) {
                    // The instruction stores its result into a local variable which
                    // is an observable side effect which we are not allowed to mutate.
                    return false;
                }
                if $instr.result != $input {
                    // The result of the instruction and the current input are not equal
                    // thus indicating that we cannot fuse the instructions.
                    return false;
                }
                $make_fuse($instr.result, $instr.reg_in, $instr.imm_in)
            }};
        }

        let Provider::Register(input) = stack.peek() else {
            return false;
        };
        let Some(last_instr) = self.last_instr else {
            return false;
        };
        let fused_instr = match self.instrs.get(last_instr) {
            Instruction::I32And(instr) => fuse!(instr, stack, input, Instruction::i32_and_eqz),
            Instruction::I32AndImm16(instr) => {
                fuse_imm16!(instr, stack, input, Instruction::i32_and_eqz_imm16)
            }
            Instruction::I32Or(instr) => fuse!(instr, stack, input, Instruction::i32_or_eqz),
            Instruction::I32OrImm16(instr) => {
                fuse_imm16!(instr, stack, input, Instruction::i32_or_eqz_imm16)
            }
            Instruction::I32Xor(instr) => fuse!(instr, stack, input, Instruction::i32_xor_eqz),
            Instruction::I32XorImm16(instr) => {
                fuse_imm16!(instr, stack, input, Instruction::i32_xor_eqz_imm16)
            }
            _ => return false,
        };
        _ = mem::replace(self.instrs.get_mut(last_instr), fused_instr);
        true
    }

    /// Encodes a `branch_eqz` instruction and tries to fuse it with a previous comparison instruction.
    pub fn encode_branch_eqz(
        &mut self,
        stack: &mut ValueStack,
        condition: Register,
        label: LabelRef,
    ) -> Result<(), Error> {
        type BranchCmpConstructor = fn(Register, Register, BranchOffset16) -> Instruction;
        type BranchCmpImmConstructor<T> = fn(Register, Const16<T>, BranchOffset16) -> Instruction;

        /// Create an [`Instruction::BranchCmpFallback`].
        fn make_branch_cmp_fallback(
            stack: &mut ValueStack,
            cmp: BranchComparator,
            lhs: Register,
            rhs: Register,
            offset: BranchOffset,
        ) -> Result<Instruction, Error> {
            let params = stack.alloc_const(ComparatorOffsetParam::new(cmp, offset))?;
            Ok(Instruction::branch_cmp_fallback(lhs, rhs, params))
        }

        /// Encode an unoptimized `branch_eqz` instruction.
        ///
        /// This is used as fallback whenever fusing compare and branch instructions is not possible.
        fn encode_branch_eqz_fallback(
            this: &mut InstrEncoder,
            stack: &mut ValueStack,
            condition: Register,
            label: LabelRef,
        ) -> Result<(), Error> {
            let offset = this.try_resolve_label(label)?;
            let instr = match BranchOffset16::try_from(offset) {
                Ok(offset) => Instruction::branch_i32_eqz(condition, offset),
                Err(_) => {
                    let zero = stack.alloc_const(0_i32)?;
                    make_branch_cmp_fallback(
                        stack,
                        BranchComparator::I32Eq,
                        condition,
                        zero,
                        offset,
                    )?
                }
            };
            this.push_instr(instr)?;
            Ok(())
        }

        /// Create a fused cmp+branch instruction and wrap it in a `Some`.
        ///
        /// We wrap the returned value in `Some` to unify handling of a bunch of cases.
        #[allow(clippy::too_many_arguments)]
        fn fuse(
            this: &mut InstrEncoder,
            stack: &mut ValueStack,
            last_instr: Instr,
            condition: Register,
            instr: BinInstr,
            label: LabelRef,
            cmp: BranchComparator,
            make_instr: BranchCmpConstructor,
        ) -> Result<Option<Instruction>, Error> {
            if matches!(stack.get_register_space(instr.result), RegisterSpace::Local) {
                // We need to filter out instructions that store their result
                // into a local register slot because they introduce observable behavior
                // which a fused cmp+branch instruction would remove.
                return Ok(None);
            }
            if instr.result != condition {
                // We cannot fuse the instructions since the result of the compare instruction
                // does not match the input of the conditional branch instruction.
                return Ok(None);
            }
            let offset = this.try_resolve_label_for(label, last_instr)?;
            let instr = match BranchOffset16::try_from(offset) {
                Ok(offset) => make_instr(instr.lhs, instr.rhs, offset),
                Err(_) => make_branch_cmp_fallback(stack, cmp, instr.lhs, instr.rhs, offset)?,
            };
            Ok(Some(instr))
        }

        /// Create a fused cmp+branch instruction with a 16-bit immediate and wrap it in a `Some`.
        ///
        /// We wrap the returned value in `Some` to unify handling of a bunch of cases.
        #[allow(clippy::too_many_arguments)]
        fn fuse_imm<T>(
            this: &mut InstrEncoder,
            stack: &mut ValueStack,
            last_instr: Instr,
            condition: Register,
            instr: BinInstrImm16<T>,
            label: LabelRef,
            cmp: BranchComparator,
            make_instr: BranchCmpImmConstructor<T>,
        ) -> Result<Option<Instruction>, Error>
        where
            T: From<Const16<T>> + Into<UntypedVal>,
        {
            if matches!(stack.get_register_space(instr.result), RegisterSpace::Local) {
                // We need to filter out instructions that store their result
                // into a local register slot because they introduce observable behavior
                // which a fused cmp+branch instruction would remove.
                return Ok(None);
            }
            if instr.result != condition {
                // We cannot fuse the instructions since the result of the compare instruction
                // does not match the input of the conditional branch instruction.
                return Ok(None);
            }
            let offset = this.try_resolve_label_for(label, last_instr)?;
            let instr = match BranchOffset16::try_from(offset) {
                Ok(offset) => make_instr(instr.reg_in, instr.imm_in, offset),
                Err(_) => {
                    let rhs = stack.alloc_const(T::from(instr.imm_in))?;
                    make_branch_cmp_fallback(stack, cmp, instr.reg_in, rhs, offset)?
                }
            };
            Ok(Some(instr))
        }
        use BranchComparator as Cmp;
        use Instruction as I;

        let Some(last_instr) = self.last_instr else {
            return encode_branch_eqz_fallback(self, stack, condition, label);
        };

        #[rustfmt::skip]
        let fused_instr = match *self.instrs.get(last_instr) {
            I::I32And(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32AndEqz, I::branch_i32_and_eqz as _)?,
            I::I32Or(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32OrEqz, I::branch_i32_or_eqz as _)?,
            I::I32Xor(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32XorEqz, I::branch_i32_xor_eqz as _)?,
            I::I32AndEqz(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32And, I::branch_i32_and as _)?,
            I::I32OrEqz(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32Or, I::branch_i32_or as _)?,
            I::I32XorEqz(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32Xor, I::branch_i32_xor as _)?,
            I::I32Eq(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32Ne, I::branch_i32_ne as _)?,
            I::I32Ne(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32Eq, I::branch_i32_eq as _)?,
            I::I32LtS(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32GeS, I::branch_i32_ge_s as _)?,
            I::I32LtU(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32GeU, I::branch_i32_ge_u as _)?,
            I::I32LeS(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32GtS, I::branch_i32_gt_s as _)?,
            I::I32LeU(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32GtU, I::branch_i32_gt_u as _)?,
            I::I32GtS(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32LeS, I::branch_i32_le_s as _)?,
            I::I32GtU(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32LeU, I::branch_i32_le_u as _)?,
            I::I32GeS(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32LtS, I::branch_i32_lt_s as _)?,
            I::I32GeU(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32LtU, I::branch_i32_lt_u as _)?,
            I::I64Eq(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64Ne, I::branch_i64_ne as _)?,
            I::I64Ne(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64Eq, I::branch_i64_eq as _)?,
            I::I64LtS(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64GeS, I::branch_i64_ge_s as _)?,
            I::I64LtU(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64GeU, I::branch_i64_ge_u as _)?,
            I::I64LeS(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64GtS, I::branch_i64_gt_s as _)?,
            I::I64LeU(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64GtU, I::branch_i64_gt_u as _)?,
            I::I64GtS(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64LeS, I::branch_i64_le_s as _)?,
            I::I64GtU(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64LeU, I::branch_i64_le_u as _)?,
            I::I64GeS(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64LtS, I::branch_i64_lt_s as _)?,
            I::I64GeU(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64LtU, I::branch_i64_lt_u as _)?,
            I::F32Eq(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::F32Ne, I::branch_f32_ne as _)?,
            I::F32Ne(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::F32Eq, I::branch_f32_eq as _)?,
            // Note: We cannot fuse cmp+branch for float comparison operators due to how NaN values are treated.
            I::I32AndImm16(instr) => fuse_imm::<i32>(self, stack, last_instr, condition, instr, label, Cmp::I32AndEqz, I::branch_i32_and_eqz_imm as _)?,
            I::I32OrImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32OrEqz, I::branch_i32_or_eqz_imm as _)?,
            I::I32XorImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32XorEqz, I::branch_i32_xor_eqz_imm as _)?,
            I::I32AndEqzImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32And, I::branch_i32_and_imm as _)?,
            I::I32OrEqzImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32Or, I::branch_i32_or_imm as _)?,
            I::I32XorEqzImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32Xor, I::branch_i32_xor_imm as _)?,
            I::I32EqImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32Ne, I::branch_i32_ne_imm as _)?,
            I::I32NeImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32Eq, I::branch_i32_eq_imm as _)?,
            I::I32LtSImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32GeS, I::branch_i32_ge_s_imm as _)?,
            I::I32LtUImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32GeU, I::branch_i32_ge_u_imm as _)?,
            I::I32LeSImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32GtS, I::branch_i32_gt_s_imm as _)?,
            I::I32LeUImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32GtU, I::branch_i32_gt_u_imm as _)?,
            I::I32GtSImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32LeS, I::branch_i32_le_s_imm as _)?,
            I::I32GtUImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32LeU, I::branch_i32_le_u_imm as _)?,
            I::I32GeSImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32LtS, I::branch_i32_lt_s_imm as _)?,
            I::I32GeUImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32LtU, I::branch_i32_lt_u_imm as _)?,
            I::I64EqImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64Ne, I::branch_i64_ne_imm as _)?,
            I::I64NeImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64Eq, I::branch_i64_eq_imm as _)?,
            I::I64LtSImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64GeS, I::branch_i64_ge_s_imm as _)?,
            I::I64LtUImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64GeU, I::branch_i64_ge_u_imm as _)?,
            I::I64LeSImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64GtS, I::branch_i64_gt_s_imm as _)?,
            I::I64LeUImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64GtU, I::branch_i64_gt_u_imm as _)?,
            I::I64GtSImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64LeS, I::branch_i64_le_s_imm as _)?,
            I::I64GtUImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64LeU, I::branch_i64_le_u_imm as _)?,
            I::I64GeSImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64LtS, I::branch_i64_lt_s_imm as _)?,
            I::I64GeUImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64LtU, I::branch_i64_lt_u_imm as _)?,
            _ => None,
        };
        if let Some(fused_instr) = fused_instr {
            _ = mem::replace(self.instrs.get_mut(last_instr), fused_instr);
            return Ok(());
        }
        encode_branch_eqz_fallback(self, stack, condition, label)
    }

    /// Encodes a `branch_nez` instruction and tries to fuse it with a previous comparison instruction.
    pub fn encode_branch_nez(
        &mut self,
        stack: &mut ValueStack,
        condition: Register,
        label: LabelRef,
    ) -> Result<(), Error> {
        type BranchCmpConstructor = fn(Register, Register, BranchOffset16) -> Instruction;
        type BranchCmpImmConstructor<T> = fn(Register, Const16<T>, BranchOffset16) -> Instruction;

        /// Create an [`Instruction::BranchCmpFallback`].
        fn make_branch_cmp_fallback(
            stack: &mut ValueStack,
            cmp: BranchComparator,
            lhs: Register,
            rhs: Register,
            offset: BranchOffset,
        ) -> Result<Instruction, Error> {
            let params = stack.alloc_const(ComparatorOffsetParam::new(cmp, offset))?;
            Ok(Instruction::branch_cmp_fallback(lhs, rhs, params))
        }

        /// Encode an unoptimized `branch_nez` instruction.
        ///
        /// This is used as fallback whenever fusing compare and branch instructions is not possible.
        fn encode_branch_nez_fallback(
            this: &mut InstrEncoder,
            stack: &mut ValueStack,
            condition: Register,
            label: LabelRef,
        ) -> Result<(), Error> {
            let offset = this.try_resolve_label(label)?;
            let instr = match BranchOffset16::try_from(offset) {
                Ok(offset) => Instruction::branch_i32_nez(condition, offset),
                Err(_) => {
                    let zero = stack.alloc_const(0_i32)?;
                    make_branch_cmp_fallback(
                        stack,
                        BranchComparator::I32Ne,
                        condition,
                        zero,
                        offset,
                    )?
                }
            };
            this.push_instr(instr)?;
            Ok(())
        }

        /// Create a fused cmp+branch instruction and wrap it in a `Some`.
        ///
        /// We wrap the returned value in `Some` to unify handling of a bunch of cases.
        #[allow(clippy::too_many_arguments)]
        fn fuse(
            this: &mut InstrEncoder,
            stack: &mut ValueStack,
            last_instr: Instr,
            condition: Register,
            instr: BinInstr,
            label: LabelRef,
            cmp: BranchComparator,
            make_instr: BranchCmpConstructor,
        ) -> Result<Option<Instruction>, Error> {
            if matches!(stack.get_register_space(instr.result), RegisterSpace::Local) {
                // We need to filter out instructions that store their result
                // into a local register slot because they introduce observable behavior
                // which a fused cmp+branch instruction would remove.
                return Ok(None);
            }
            if instr.result != condition {
                // We cannot fuse the instructions since the result of the compare instruction
                // does not match the input of the conditional branch instruction.
                return Ok(None);
            }
            let offset = this.try_resolve_label_for(label, last_instr)?;
            let instr = match BranchOffset16::try_from(offset) {
                Ok(offset) => make_instr(instr.lhs, instr.rhs, offset),
                Err(_) => make_branch_cmp_fallback(stack, cmp, instr.lhs, instr.rhs, offset)?,
            };
            Ok(Some(instr))
        }

        /// Create a fused cmp+branch instruction with a 16-bit immediate and wrap it in a `Some`.
        ///
        /// We wrap the returned value in `Some` to unify handling of a bunch of cases.
        #[allow(clippy::too_many_arguments)]
        fn fuse_imm<T>(
            this: &mut InstrEncoder,
            stack: &mut ValueStack,
            last_instr: Instr,
            condition: Register,
            instr: BinInstrImm16<T>,
            label: LabelRef,
            cmp: BranchComparator,
            make_instr: BranchCmpImmConstructor<T>,
        ) -> Result<Option<Instruction>, Error>
        where
            T: From<Const16<T>> + Into<UntypedVal>,
        {
            if matches!(stack.get_register_space(instr.result), RegisterSpace::Local) {
                // We need to filter out instructions that store their result
                // into a local register slot because they introduce observable behavior
                // which a fused cmp+branch instruction would remove.
                return Ok(None);
            }
            if instr.result != condition {
                // We cannot fuse the instructions since the result of the compare instruction
                // does not match the input of the conditional branch instruction.
                return Ok(None);
            }
            let offset = this.try_resolve_label_for(label, last_instr)?;
            let instr = match BranchOffset16::try_from(offset) {
                Ok(offset) => make_instr(instr.reg_in, instr.imm_in, offset),
                Err(_) => {
                    let rhs = stack.alloc_const(T::from(instr.imm_in))?;
                    make_branch_cmp_fallback(stack, cmp, instr.reg_in, rhs, offset)?
                }
            };
            Ok(Some(instr))
        }
        use BranchComparator as Cmp;
        use Instruction as I;

        let Some(last_instr) = self.last_instr else {
            return encode_branch_nez_fallback(self, stack, condition, label);
        };

        #[rustfmt::skip]
        let fused_instr = match *self.instrs.get(last_instr) {
            I::I32And(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32And, I::branch_i32_and as _)?,
            I::I32Or(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32Or, I::branch_i32_or as _)?,
            I::I32Xor(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32Xor, I::branch_i32_xor as _)?,
            I::I32AndEqz(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32AndEqz, I::branch_i32_and_eqz as _)?,
            I::I32OrEqz(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32OrEqz, I::branch_i32_or_eqz as _)?,
            I::I32XorEqz(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32XorEqz, I::branch_i32_xor_eqz as _)?,
            I::I32Eq(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32Eq, I::branch_i32_eq as _)?,
            I::I32Ne(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32Ne, I::branch_i32_ne as _)?,
            I::I32LtS(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32LtS, I::branch_i32_lt_s as _)?,
            I::I32LtU(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32LtU, I::branch_i32_lt_u as _)?,
            I::I32LeS(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32LeS, I::branch_i32_le_s as _)?,
            I::I32LeU(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32LeU, I::branch_i32_le_u as _)?,
            I::I32GtS(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32GtS, I::branch_i32_gt_s as _)?,
            I::I32GtU(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32GtU, I::branch_i32_gt_u as _)?,
            I::I32GeS(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32GeS, I::branch_i32_ge_s as _)?,
            I::I32GeU(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I32GeU, I::branch_i32_ge_u as _)?,
            I::I64Eq(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64Eq, I::branch_i64_eq as _)?,
            I::I64Ne(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64Ne, I::branch_i64_ne as _)?,
            I::I64LtS(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64LtS, I::branch_i64_lt_s as _)?,
            I::I64LtU(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64LtU, I::branch_i64_lt_u as _)?,
            I::I64LeS(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64LeS, I::branch_i64_le_s as _)?,
            I::I64LeU(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64LeU, I::branch_i64_le_u as _)?,
            I::I64GtS(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64GtS, I::branch_i64_gt_s as _)?,
            I::I64GtU(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64GtU, I::branch_i64_gt_u as _)?,
            I::I64GeS(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64GeS, I::branch_i64_ge_s as _)?,
            I::I64GeU(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::I64GeU, I::branch_i64_ge_u as _)?,
            I::F32Eq(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::F32Eq, I::branch_f32_eq as _)?,
            I::F32Ne(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::F32Ne, I::branch_f32_ne as _)?,
            I::F32Lt(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::F32Lt, I::branch_f32_lt as _)?,
            I::F32Le(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::F32Le, I::branch_f32_le as _)?,
            I::F32Gt(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::F32Gt, I::branch_f32_gt as _)?,
            I::F32Ge(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::F32Ge, I::branch_f32_ge as _)?,
            I::F64Eq(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::F64Eq, I::branch_f64_eq as _)?,
            I::F64Ne(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::F64Ne, I::branch_f64_ne as _)?,
            I::F64Lt(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::F64Lt, I::branch_f64_lt as _)?,
            I::F64Le(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::F64Le, I::branch_f64_le as _)?,
            I::F64Gt(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::F64Gt, I::branch_f64_gt as _)?,
            I::F64Ge(instr) => fuse(self, stack, last_instr, condition, instr, label, Cmp::F64Ge, I::branch_f64_ge as _)?,
            I::I32AndImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32And, I::branch_i32_and_imm as _)?,
            I::I32OrImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32Or, I::branch_i32_or_imm as _)?,
            I::I32XorImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32Xor, I::branch_i32_xor_imm as _)?,
            I::I32AndEqzImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32AndEqz, I::branch_i32_and_eqz_imm as _)?,
            I::I32OrEqzImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32OrEqz, I::branch_i32_or_eqz_imm as _)?,
            I::I32XorEqzImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32XorEqz, I::branch_i32_xor_eqz_imm as _)?,
            I::I32EqImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32Eq, I::branch_i32_eq_imm as _)?,
            I::I32NeImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32Ne, I::branch_i32_ne_imm as _)?,
            I::I32LtSImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32LtS, I::branch_i32_lt_s_imm as _)?,
            I::I32LtUImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32LtU, I::branch_i32_lt_u_imm as _)?,
            I::I32LeSImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32LeS, I::branch_i32_le_s_imm as _)?,
            I::I32LeUImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32LeU, I::branch_i32_le_u_imm as _)?,
            I::I32GtSImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32GtS, I::branch_i32_gt_s_imm as _)?,
            I::I32GtUImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32GtU, I::branch_i32_gt_u_imm as _)?,
            I::I32GeSImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32GeS, I::branch_i32_ge_s_imm as _)?,
            I::I32GeUImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I32GeU, I::branch_i32_ge_u_imm as _)?,
            I::I64EqImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64Eq, I::branch_i64_eq_imm as _)?,
            I::I64NeImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64Ne, I::branch_i64_ne_imm as _)?,
            I::I64LtSImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64LtS, I::branch_i64_lt_s_imm as _)?,
            I::I64LtUImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64LtU, I::branch_i64_lt_u_imm as _)?,
            I::I64LeSImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64LeS, I::branch_i64_le_s_imm as _)?,
            I::I64LeUImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64LeU, I::branch_i64_le_u_imm as _)?,
            I::I64GtSImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64GtS, I::branch_i64_gt_s_imm as _)?,
            I::I64GtUImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64GtU, I::branch_i64_gt_u_imm as _)?,
            I::I64GeSImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64GeS, I::branch_i64_ge_s_imm as _)?,
            I::I64GeUImm16(instr) => fuse_imm(self, stack, last_instr, condition, instr, label, Cmp::I64GeU, I::branch_i64_ge_u_imm as _)?,
            _ => None,
        };
        if let Some(fused_instr) = fused_instr {
            _ = mem::replace(self.instrs.get_mut(last_instr), fused_instr);
            return Ok(());
        }
        encode_branch_nez_fallback(self, stack, condition, label)
    }
}

impl Instruction {
    /// Updates the [`BranchOffset`] for the branch [`Instruction].
    ///
    /// # Panics
    ///
    /// If `self` is not a branch [`Instruction`].
    #[rustfmt::skip]
    pub fn update_branch_offset(&mut self, stack: &mut ValueStack, new_offset: BranchOffset) -> Result<(), Error> {
        /// Initializes the 16-bit offset of `instr` if possible.
        /// 
        /// If `new_offset` cannot be encoded as 16-bit offset `self` is replaced with a fallback instruction.
        macro_rules! init_offset {
            ($instr:expr, $new_offset:expr, $cmp:expr) => {{
                if let Err(_) = $instr.offset.init($new_offset) {
                    let params = stack.alloc_const(ComparatorOffsetParam::new($cmp, $new_offset))?;
                    *self = Instruction::branch_cmp_fallback($instr.lhs, $instr.rhs, params);
                }
                Ok(())
            }}
        }

        macro_rules! init_offset_imm {
            ($ty:ty, $instr:expr, $new_offset:expr, $cmp:expr) => {{
                if let Err(_) = $instr.offset.init($new_offset) {
                    let rhs = stack.alloc_const(<$ty>::from($instr.rhs))?;
                    let params = stack.alloc_const(ComparatorOffsetParam::new($cmp, $new_offset))?;
                    *self = Instruction::branch_cmp_fallback($instr.lhs, rhs, params);
                }
                Ok(())
            }};
        }

        use Instruction as I;
        use BranchComparator as Cmp;
        match self {
            Instruction::Branch { offset } => {
                offset.init(new_offset);
                Ok(())
            }
            I::BranchI32And(instr) => init_offset!(instr, new_offset, Cmp::I32And),
            I::BranchI32Or(instr) => init_offset!(instr, new_offset, Cmp::I32Or),
            I::BranchI32Xor(instr) => init_offset!(instr, new_offset, Cmp::I32Xor),
            I::BranchI32AndEqz(instr) => init_offset!(instr, new_offset, Cmp::I32AndEqz),
            I::BranchI32OrEqz(instr) => init_offset!(instr, new_offset, Cmp::I32OrEqz),
            I::BranchI32XorEqz(instr) => init_offset!(instr, new_offset, Cmp::I32XorEqz),
            I::BranchI32Eq(instr) => init_offset!(instr, new_offset, Cmp::I32Eq),
            I::BranchI32Ne(instr) => init_offset!(instr, new_offset, Cmp::I32Ne),
            I::BranchI32LtS(instr) => init_offset!(instr, new_offset, Cmp::I32LtS),
            I::BranchI32LtU(instr) => init_offset!(instr, new_offset, Cmp::I32LtU),
            I::BranchI32LeS(instr) => init_offset!(instr, new_offset, Cmp::I32LeS),
            I::BranchI32LeU(instr) => init_offset!(instr, new_offset, Cmp::I32LeU),
            I::BranchI32GtS(instr) => init_offset!(instr, new_offset, Cmp::I32GtS),
            I::BranchI32GtU(instr) => init_offset!(instr, new_offset, Cmp::I32GtU),
            I::BranchI32GeS(instr) => init_offset!(instr, new_offset, Cmp::I32GeS),
            I::BranchI32GeU(instr) => init_offset!(instr, new_offset, Cmp::I32GeU),
            I::BranchI64Eq(instr) => init_offset!(instr, new_offset, Cmp::I64Eq),
            I::BranchI64Ne(instr) => init_offset!(instr, new_offset, Cmp::I64Ne),
            I::BranchI64LtS(instr) => init_offset!(instr, new_offset, Cmp::I64LtS),
            I::BranchI64LtU(instr) => init_offset!(instr, new_offset, Cmp::I64LtU),
            I::BranchI64LeS(instr) => init_offset!(instr, new_offset, Cmp::I64LeS),
            I::BranchI64LeU(instr) => init_offset!(instr, new_offset, Cmp::I64LeU),
            I::BranchI64GtS(instr) => init_offset!(instr, new_offset, Cmp::I64GtS),
            I::BranchI64GtU(instr) => init_offset!(instr, new_offset, Cmp::I64GtU),
            I::BranchI64GeS(instr) => init_offset!(instr, new_offset, Cmp::I64GeS),
            I::BranchI64GeU(instr) => init_offset!(instr, new_offset, Cmp::I64GeU),
            I::BranchF32Eq(instr) => init_offset!(instr, new_offset, Cmp::F32Eq),
            I::BranchF32Ne(instr) => init_offset!(instr, new_offset, Cmp::F32Ne),
            I::BranchF32Lt(instr) => init_offset!(instr, new_offset, Cmp::F32Lt),
            I::BranchF32Le(instr) => init_offset!(instr, new_offset, Cmp::F32Le),
            I::BranchF32Gt(instr) => init_offset!(instr, new_offset, Cmp::F32Gt),
            I::BranchF32Ge(instr) => init_offset!(instr, new_offset, Cmp::F32Ge),
            I::BranchF64Eq(instr) => init_offset!(instr, new_offset, Cmp::F64Eq),
            I::BranchF64Ne(instr) => init_offset!(instr, new_offset, Cmp::F64Ne),
            I::BranchF64Lt(instr) => init_offset!(instr, new_offset, Cmp::F64Lt),
            I::BranchF64Le(instr) => init_offset!(instr, new_offset, Cmp::F64Le),
            I::BranchF64Gt(instr) => init_offset!(instr, new_offset, Cmp::F64Gt),
            I::BranchF64Ge(instr) => init_offset!(instr, new_offset, Cmp::F64Ge),
            I::BranchI32AndImm(instr) => init_offset_imm!(i32, instr, new_offset, Cmp::I32And),
            I::BranchI32OrImm(instr) => init_offset_imm!(i32, instr, new_offset, Cmp::I32Or),
            I::BranchI32XorImm(instr) => init_offset_imm!(i32, instr, new_offset, Cmp::I32Xor),
            I::BranchI32AndEqzImm(instr) => init_offset_imm!(i32, instr, new_offset, Cmp::I32AndEqz),
            I::BranchI32OrEqzImm(instr) => init_offset_imm!(i32, instr, new_offset, Cmp::I32OrEqz),
            I::BranchI32XorEqzImm(instr) => init_offset_imm!(i32, instr, new_offset, Cmp::I32XorEqz),
            I::BranchI32EqImm(instr) => init_offset_imm!(i32, instr, new_offset, Cmp::I32Eq),
            I::BranchI32NeImm(instr) => init_offset_imm!(i32, instr, new_offset, Cmp::I32Ne),
            I::BranchI32LtSImm(instr) => init_offset_imm!(i32, instr, new_offset, Cmp::I32LtS),
            I::BranchI32LeSImm(instr) => init_offset_imm!(i32, instr, new_offset, Cmp::I32LeS),
            I::BranchI32GtSImm(instr) => init_offset_imm!(i32, instr, new_offset, Cmp::I32GtS),
            I::BranchI32GeSImm(instr) => init_offset_imm!(i32, instr, new_offset, Cmp::I32GeS),
            I::BranchI32LtUImm(instr) => init_offset_imm!(u32, instr, new_offset, Cmp::I32LtU),
            I::BranchI32LeUImm(instr) => init_offset_imm!(u32, instr, new_offset, Cmp::I32LeU),
            I::BranchI32GtUImm(instr) => init_offset_imm!(u32, instr, new_offset, Cmp::I32GtU),
            I::BranchI32GeUImm(instr) => init_offset_imm!(u32, instr, new_offset, Cmp::I32GeU),
            I::BranchI64EqImm(instr) => init_offset_imm!(i64, instr, new_offset, Cmp::I64Eq),
            I::BranchI64NeImm(instr) => init_offset_imm!(i64, instr, new_offset, Cmp::I64Ne),
            I::BranchI64LtSImm(instr) => init_offset_imm!(i64, instr, new_offset, Cmp::I64LtS),
            I::BranchI64LeSImm(instr) => init_offset_imm!(i64, instr, new_offset, Cmp::I64LeS),
            I::BranchI64GtSImm(instr) => init_offset_imm!(i64, instr, new_offset, Cmp::I64GtS),
            I::BranchI64GeSImm(instr) => init_offset_imm!(i64, instr, new_offset, Cmp::I64GeS),
            I::BranchI64LtUImm(instr) => init_offset_imm!(u64, instr, new_offset, Cmp::I64LtU),
            I::BranchI64LeUImm(instr) => init_offset_imm!(u64, instr, new_offset, Cmp::I64LeU),
            I::BranchI64GtUImm(instr) => init_offset_imm!(u64, instr, new_offset, Cmp::I64GtU),
            I::BranchI64GeUImm(instr) => init_offset_imm!(u64, instr, new_offset, Cmp::I64GeU),
            _ => panic!("tried to update branch offset of a non-branch instruction: {self:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::translator::typed_value::TypedVal;

    #[test]
    fn has_overlapping_copies_works() {
        assert!(!InstrEncoder::has_overlapping_copies(
            RegisterSpan::new(Register::from_i16(0)).iter(0),
            &[],
        ));
        assert!(!InstrEncoder::has_overlapping_copies(
            RegisterSpan::new(Register::from_i16(0)).iter(2),
            &[TypedProvider::register(0), TypedProvider::register(1),],
        ));
        assert!(!InstrEncoder::has_overlapping_copies(
            RegisterSpan::new(Register::from_i16(0)).iter(2),
            &[
                TypedProvider::Const(TypedVal::from(10_i32)),
                TypedProvider::Const(TypedVal::from(20_i32)),
            ],
        ));
        assert!(InstrEncoder::has_overlapping_copies(
            RegisterSpan::new(Register::from_i16(0)).iter(2),
            &[
                TypedProvider::Const(TypedVal::from(10_i32)),
                TypedProvider::register(0),
            ],
        ));
        assert!(InstrEncoder::has_overlapping_copies(
            RegisterSpan::new(Register::from_i16(0)).iter(2),
            &[TypedProvider::register(0), TypedProvider::register(0),],
        ));
        assert!(InstrEncoder::has_overlapping_copies(
            RegisterSpan::new(Register::from_i16(3)).iter(3),
            &[
                TypedProvider::register(2),
                TypedProvider::register(3),
                TypedProvider::register(2),
            ],
        ));
        assert!(InstrEncoder::has_overlapping_copies(
            RegisterSpan::new(Register::from_i16(3)).iter(4),
            &[
                TypedProvider::register(-1),
                TypedProvider::register(10),
                TypedProvider::register(2),
                TypedProvider::register(4),
            ],
        ));
    }
}
