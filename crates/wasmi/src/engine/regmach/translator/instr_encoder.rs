use super::{visit_register::VisitInputRegisters, TypedProvider};
use crate::{
    engine::{
        bytecode::BranchOffset,
        func_builder::{
            labels::{LabelRef, LabelRegistry},
            Instr,
        },
        regmach::{
            bytecode::{Const32, Instruction, Provider, Register, RegisterSpanIter},
            translator::ValueStack,
        },
        TranslationError,
    },
    module::ModuleResources,
};
use alloc::vec::{Drain, Vec};
use wasmi_core::{UntypedValue, ValueType, F32};

/// Encodes `wasmi` bytecode instructions to an [`Instruction`] stream.
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
    fn push(&mut self, instruction: Instruction) -> Result<Instr, TranslationError> {
        let instr = self.next_instr();
        self.instrs.push(instruction);
        Ok(instr)
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
    pub fn try_resolve_label(&mut self, label: LabelRef) -> Result<BranchOffset, TranslationError> {
        let user = self.instrs.next_instr();
        self.try_resolve_label_for(label, user)
    }

    /// Try resolving the [`LabelRef`] for the given [`Instr`].
    ///
    /// Returns an uninitialized [`BranchOffset`] if the `label` cannot yet
    /// be resolved and defers resolution to later.
    pub fn try_resolve_label_for(
        &mut self,
        label: LabelRef,
        instr: Instr,
    ) -> Result<BranchOffset, TranslationError> {
        self.labels.try_resolve_label(label, instr)
    }

    /// Updates the branch offsets of all branch instructions inplace.
    ///
    /// # Panics
    ///
    /// If this is used before all branching labels have been pinned.
    pub fn update_branch_offsets(&mut self) -> Result<(), TranslationError> {
        for (user, offset) in self.labels.resolved_users() {
            self.instrs.get_mut(user).update_branch_offset(offset?);
        }
        Ok(())
    }

    /// Bumps consumed fuel for [`Instruction::ConsumeFuel`] of `instr` by `delta`.
    ///
    /// # Errors
    ///
    /// If consumed fuel is out of bounds after this operation.
    #[allow(dead_code)] // TODO: remove
    pub fn bump_fuel_consumption(
        &mut self,
        instr: Instr,
        delta: u64,
    ) -> Result<(), TranslationError> {
        self.instrs.get_mut(instr).bump_fuel_consumption(delta)
    }

    /// Push the [`Instruction`] to the [`InstrEncoder`].
    pub fn push_instr(&mut self, instr: Instruction) -> Result<Instr, TranslationError> {
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
    pub fn append_instr(&mut self, instr: Instruction) -> Result<Instr, TranslationError> {
        self.instrs.push(instr)
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
    ) -> Result<Option<Instr>, TranslationError> {
        /// Convenience to create an [`Instruction::Copy`] to copy a constant value.
        fn copy_imm(
            stack: &mut ValueStack,
            result: Register,
            value: impl Into<UntypedValue>,
        ) -> Result<Instruction, TranslationError> {
            let cref = stack.alloc_const(value.into())?;
            Ok(Instruction::copy(result, cref))
        }
        match value {
            TypedProvider::Register(value) => {
                if result == value {
                    // Optimization: copying from register `x` into `x` is a no-op.
                    return Ok(None);
                }
                let instr = self.push_instr(Instruction::copy(result, value))?;
                Ok(Some(instr))
            }
            TypedProvider::Const(value) => {
                let instruction = match value.ty() {
                    ValueType::I32 => Instruction::copy_imm32(result, i32::from(value)),
                    ValueType::F32 => Instruction::copy_imm32(result, f32::from(value)),
                    ValueType::I64 => match <Const32<i64>>::from_i64(i64::from(value)) {
                        Some(value) => Instruction::copy_i64imm32(result, value),
                        None => copy_imm(stack, result, value)?,
                    },
                    ValueType::F64 => match <Const32<f64>>::from_f64(f64::from(value)) {
                        Some(value) => Instruction::copy_f64imm32(result, value),
                        None => copy_imm(stack, result, value)?,
                    },
                    ValueType::FuncRef => copy_imm(stack, result, value)?,
                    ValueType::ExternRef => copy_imm(stack, result, value)?,
                };
                let instr = self.push_instr(instruction)?;
                Ok(Some(instr))
            }
        }
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
    ) -> Result<(), TranslationError> {
        assert_eq!(results.len(), values.len());
        if let Some((TypedProvider::Register(value), rest)) = values.split_first() {
            if results.span().head() == *value {
                // Case: `result` and `value` are equal thus this is a no-op copy which we can avoid.
                //       Applied recursively we thereby remove all no-op copies at the start of the
                //       copy sequence until the first actual copy.
                results.next();
                return self.encode_copies(stack, results, rest);
            }
        }
        let result = results.span().head();
        let instr = match values {
            [] => {
                // The copy sequence is empty, nothing to encode in this case.
                return Ok(());
            }
            [TypedProvider::Register(reg)] => Instruction::copy(result, *reg),
            [TypedProvider::Const(value)] => match value.ty() {
                ValueType::I32 => Instruction::copy_imm32(result, i32::from(*value)),
                ValueType::I64 => match <Const32<i64>>::from_i64(i64::from(*value)) {
                    Some(value) => Instruction::copy_i64imm32(result, value),
                    None => Instruction::copy(result, stack.alloc_const(*value)?),
                },
                ValueType::F32 => Instruction::copy_imm32(result, F32::from(*value)),
                ValueType::F64 => match <Const32<f64>>::from_f64(f64::from(*value)) {
                    Some(value) => Instruction::copy_f64imm32(result, value),
                    None => Instruction::copy(result, stack.alloc_const(*value)?),
                },
                ValueType::FuncRef | ValueType::ExternRef => {
                    Instruction::copy(result, stack.alloc_const(*value)?)
                }
            },
            [v0, v1] => {
                let reg0 = Self::provider2reg(stack, v0)?;
                let reg1 = Self::provider2reg(stack, v1)?;
                if result.next() == reg1 {
                    // Case: the second of the 2 copies is a no-op which we can avoid
                    // Note: we already asserted that the first copy is not a no-op
                    Instruction::copy(result, reg0)
                } else {
                    Instruction::copy2(results.span(), reg0, reg1)
                }
            }
            [v0, v1, rest @ ..] => {
                debug_assert!(!rest.is_empty());
                if let Some(values) = RegisterSpanIter::from_providers(values) {
                    let make_instr = match results.is_overlapping(&values) {
                        true => Instruction::copy_span,
                        false => Instruction::copy_span_non_overlapping,
                    };
                    self.push_instr(make_instr(
                        results.span(),
                        values.span(),
                        values.len_as_u16(),
                    ))?;
                    return Ok(());
                }
                let make_instr = match Self::has_overlapping_copies(results, values) {
                    true => Instruction::copy_many,
                    false => Instruction::copy_many_non_overlapping,
                };
                let reg0 = Self::provider2reg(stack, v0)?;
                let reg1 = Self::provider2reg(stack, v1)?;
                self.push_instr(make_instr(results.span(), reg0, reg1))?;
                self.encode_register_list(stack, rest)?;
                return Ok(());
            }
        };
        self.push_instr(instr)?;
        Ok(())
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

    /// Encodes an unconditional `return` instruction.
    pub fn encode_return(
        &mut self,
        stack: &mut ValueStack,
        values: &[TypedProvider],
    ) -> Result<(), TranslationError> {
        let instr = match values {
            [] => Instruction::Return,
            [TypedProvider::Register(reg)] => Instruction::return_reg(*reg),
            [TypedProvider::Const(value)] => match value.ty() {
                ValueType::I32 => Instruction::return_imm32(i32::from(*value)),
                ValueType::I64 => match <Const32<i64>>::from_i64(i64::from(*value)) {
                    Some(value) => Instruction::return_i64imm32(value),
                    None => Instruction::return_reg(stack.alloc_const(*value)?),
                },
                ValueType::F32 => Instruction::return_imm32(F32::from(*value)),
                ValueType::F64 => match <Const32<f64>>::from_f64(f64::from(*value)) {
                    Some(value) => Instruction::return_f64imm32(value),
                    None => Instruction::return_reg(stack.alloc_const(*value)?),
                },
                ValueType::FuncRef | ValueType::ExternRef => {
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
        self.push_instr(instr)?;
        Ok(())
    }

    /// Encodes an conditional `return` instruction.
    pub fn encode_return_nez(
        &mut self,
        stack: &mut ValueStack,
        condition: Register,
        values: &[TypedProvider],
    ) -> Result<(), TranslationError> {
        let instr = match values {
            [] => Instruction::return_nez(condition),
            [TypedProvider::Register(reg)] => Instruction::return_nez_reg(condition, *reg),
            [TypedProvider::Const(value)] => match value.ty() {
                ValueType::I32 => Instruction::return_nez_imm32(condition, i32::from(*value)),
                ValueType::I64 => match <Const32<i64>>::from_i64(i64::from(*value)) {
                    Some(value) => Instruction::return_nez_i64imm32(condition, value),
                    None => Instruction::return_nez_reg(condition, stack.alloc_const(*value)?),
                },
                ValueType::F32 => Instruction::return_nez_imm32(condition, F32::from(*value)),
                ValueType::F64 => match <Const32<f64>>::from_f64(f64::from(*value)) {
                    Some(value) => Instruction::return_nez_f64imm32(condition, value),
                    None => Instruction::return_nez_reg(condition, stack.alloc_const(*value)?),
                },
                ValueType::FuncRef | ValueType::ExternRef => {
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
        self.push_instr(instr)?;
        Ok(())
    }

    /// Converts a [`TypedProvider`] into a [`Register`].
    ///
    /// This allocates constant values for [`TypedProvider::Const`].
    fn provider2reg(
        stack: &mut ValueStack,
        provider: &TypedProvider,
    ) -> Result<Register, TranslationError> {
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
    ) -> Result<(), TranslationError> {
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
    /// # Note
    ///
    /// This also applies an optimization in that the previous instruction
    /// result is replaced with the `local` [`Register`] instead of encoding
    /// another `copy` instruction if the `local.set` or `local.tee` belongs
    /// to the same basic block.
    pub fn encode_local_set(
        &mut self,
        res: &ModuleResources,
        local: Register,
        value: Register,
    ) -> Result<(), TranslationError> {
        if let Some(last_instr) = self.last_instr {
            if let Some(result) = self.instrs.get_mut(last_instr).result_mut(res) {
                // Case: we can replace the `result` register of the previous
                //       instruction instead of emitting a copy instruction.
                if *result == value {
                    // TODO: Find out in what cases `result != value`. Is this a bug or an edge case?
                    //       Generally `result` should be equal to `value` since `value` refers to the
                    //       `result` of the previous instruction.
                    //       Therefore, instead of an `if` we originally had a `debug_assert`.
                    //       (Note: the spidermonkey bench test failed without this change.)
                    *result = local;
                    return Ok(());
                }
            }
        }
        // Case: we need to encode a copy instruction to encode the `local.set` or `local.tee`.
        self.push_instr(Instruction::copy(local, value))?;
        Ok(())
    }

    /// Pushes an [`Instruction::ConsumeFuel`] with base fuel costs to the [`InstrEncoder`].
    pub fn push_consume_fuel_instr(&mut self, block_fuel: u64) -> Result<Instr, TranslationError> {
        self.instrs.push(Instruction::consume_fuel(block_fuel)?)
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
        debug_assert!(
            matches!(self.instrs.get(preserve_instr), Instruction::Copy { .. }),
            "a preserve instruction is always a register copy instruction"
        );
        if self.notified_preservation.is_none() {
            self.notified_preservation = Some(preserve_instr);
        }
    }

    /// Defragments storage-space registers of all encoded [`Instruction`].
    pub fn defrag_registers(&mut self, stack: &mut ValueStack) -> Result<(), TranslationError> {
        stack.finalize_alloc();
        if let Some(notified_preserved) = self.notified_preservation {
            for instr in self.instrs.get_slice_at_mut(notified_preserved) {
                instr.visit_input_registers(|reg| *reg = stack.defrag_register(*reg));
            }
        }
        Ok(())
    }
}

impl Instruction {
    /// Updates the [`BranchOffset`] for the branch [`Instruction].
    ///
    /// # Panics
    ///
    /// If `self` is not a branch [`Instruction`].
    pub fn update_branch_offset(&mut self, new_offset: BranchOffset) {
        match self {
            Instruction::Branch { offset }
            | Instruction::BranchEqz { offset, .. }
            | Instruction::BranchNez { offset, .. } => offset.init(new_offset),
            _ => panic!("tried to update branch offset of a non-branch instruction: {self:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::regmach::{bytecode::RegisterSpan, translator::typed_value::TypedValue};

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
                TypedProvider::Const(TypedValue::from(10_i32)),
                TypedProvider::Const(TypedValue::from(20_i32)),
            ],
        ));
        assert!(InstrEncoder::has_overlapping_copies(
            RegisterSpan::new(Register::from_i16(0)).iter(2),
            &[
                TypedProvider::Const(TypedValue::from(10_i32)),
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
