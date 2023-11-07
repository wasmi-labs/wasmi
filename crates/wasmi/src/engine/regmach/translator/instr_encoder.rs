use super::{visit_register::VisitInputRegisters, TypedProvider};
use crate::{
    engine::{
        bytecode::BranchOffset,
        func_builder::{
            labels::{LabelRef, LabelRegistry},
            Instr,
            TranslationErrorInner,
        },
        regmach::{
            bytecode::{Const32, Instruction, Register, RegisterSpan, RegisterSpanIter},
            translator::ValueStack,
        },
        TranslationError,
    },
    module::ModuleResources,
};
use alloc::vec::{Drain, Vec};
use core::{cmp, mem, ops::Range};
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

    /// Returns an exclusive reference to the last [`Instruction`] of the [`InstrSequence`].
    ///
    /// # Panics
    ///
    /// If the [`InstrSequence`] is empty.
    #[track_caller]
    fn last_mut(&mut self) -> &mut Instruction {
        self.instrs
            .last_mut()
            .expect("expected non-empty instruction sequence")
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

    /// Encode a set of `copy result <- value` instructions.
    ///
    /// # Note
    ///
    /// Applies optimizations for `copy x <- x` and properly selects the
    /// most optimized `copy` instruction variants for the given `value`.
    pub fn encode_copies(
        &mut self,
        stack: &mut ValueStack,
        results: RegisterSpanIter,
        values: &[TypedProvider],
    ) -> Result<(), TranslationError> {
        if values.len() >= 2 {
            if let Some(values) = RegisterSpanIter::from_providers(values) {
                if results == values {
                    // Case: both spans are equal so there is no need to copy anything.
                    return Ok(());
                }
                // Optimization: we can encode the entire copy as [`Instruction::CopySpan`]
                self.push_instr(Instruction::copy_span(
                    results.span(),
                    values.span(),
                    results.len_as_u16(),
                ))?;
                return Ok(());
            }
        }
        let start = self.instrs.next_instr();
        let mut last_copy: Option<Instruction> = None;
        for (copy_result, copy_input) in results.zip(values.iter().copied()) {
            // Note: we should refactor this code one if-let-chains are stabilized.
            if let Some(last) = last_copy {
                if let TypedProvider::Register(copy_input) = copy_input {
                    // We might be able to merge the two last copy instructions together.
                    let merged_copy = match last {
                        Instruction::Copy { result, value } => {
                            let can_merge =
                                result.next() == copy_result && value.next() == copy_input;
                            can_merge.then(|| {
                                Instruction::copy_span(
                                    RegisterSpan::new(result),
                                    RegisterSpan::new(value),
                                    2,
                                )
                            })
                        }
                        Instruction::CopySpan {
                            results,
                            values,
                            len,
                        } => {
                            let mut last_results = results.iter_u16(len);
                            let mut last_values = values.iter_u16(len);
                            let last_result = last_results
                                .next_back()
                                .expect("CopySpan must not be empty");
                            let last_value =
                                last_values.next_back().expect("CopySpan must not be empty");
                            let can_merge = last_result.next() == copy_result
                                && last_value.next() == copy_input;
                            let new_len = len.checked_add(1).ok_or_else(|| {
                                TranslationError::new(TranslationErrorInner::RegisterOutOfBounds)
                            })?;
                            can_merge.then(|| Instruction::copy_span(results, values, new_len))
                        }
                        _ => unreachable!("must have copy instruction here"),
                    };
                    last_copy = merged_copy;
                    if let Some(merged_copy) = merged_copy {
                        let last_instr = self.instrs.last_mut();
                        _ = mem::replace(last_instr, merged_copy);
                        continue;
                    }
                }
            }
            if self.encode_copy(stack, copy_result, copy_input)?.is_some() {
                if let TypedProvider::Register(copy_input) = copy_input {
                    // At this point we know that a new register-to-register copy has been
                    // encoded and thus we can update the `last_copy` variable.
                    last_copy = Some(Instruction::copy(copy_result, copy_input));
                }
            }
        }
        let copy_instrs = self.instrs.get_slice_at_mut(start);
        if Self::is_copy_overwriting(copy_instrs) {
            // We need to sort the encoded copy instructions so that they are not overwriting themselves.
            //
            // An example is `copy 1 <- 0, copy 2 < 1` where the first `copy 1 <- 0`
            // overwrites the input of the second.
            //
            // To further circumvent this, all `copy` instructions copying immediate values always go last
            // and `copy_span` instructions come after `copy` instructions.
            copy_instrs.sort_by(|lhs, rhs| {
                use Instruction as I;
                match (lhs, rhs) {
                    (
                        I::Copy {
                            result: r0,
                            value: v0,
                        },
                        I::Copy {
                            result: r1,
                            value: v1,
                        },
                    ) => {
                        if v0 <= v1 {
                            r0.cmp(r1)
                        } else {
                            r1.cmp(r0)
                        }
                    }
                    (I::Copy { .. }, _) => cmp::Ordering::Less,
                    (_, I::Copy { .. }) => cmp::Ordering::Greater,
                    // TODO: Maybe there is a need to also order `CopySpan` amongst
                    // themselves just as we did for the `Copy` instruction.
                    (I::CopySpan { .. }, _) => cmp::Ordering::Less,
                    (_, I::CopySpan { .. }) => cmp::Ordering::Greater,
                    _ => cmp::Ordering::Equal,
                }
            });
        }
        Ok(())
    }

    /// Returns `true` if any of the `copies` overwrite the inputs of following copies.
    fn is_copy_overwriting(copies: &[Instruction]) -> bool {
        impl Register {
            /// Returns `true` if `self` is contained within the [`Register`] `range`.
            fn within_range(&self, range: Range<Register>) -> bool {
                range.contains(self)
            }
        }
        impl Instruction {
            /// Returns the first result [`Register`] of the copy [`Instruction`].
            ///
            /// # Panics
            ///
            /// If `self` is not a copy [`Instruction`].
            fn copy_result(&self) -> Register {
                match self {
                    I::Copy { result, value: _ }
                    | I::CopyImm32 { result, value: _ }
                    | I::CopyI64Imm32 { result, value: _ }
                    | I::CopyF64Imm32 { result, value: _ } => *result,
                    I::CopySpan {
                        results,
                        values: _,
                        len: _,
                    } => results.head(),
                    unexpected => panic!("encountered non-copy instruction: {unexpected:?}"),
                }
            }
        }

        use Instruction as I;
        let (head, rest) = match copies.split_first() {
            Some((head, rest)) => (head, rest),
            None => return false,
        };
        let start = head.copy_result();
        for instr in rest {
            let end = instr.copy_result();
            match instr {
                I::Copy { result: _, value } => {
                    if value.within_range(start..end) {
                        return true;
                    }
                }
                I::CopyImm32 { .. } | I::CopyI64Imm32 { .. } | I::CopyF64Imm32 { .. } => {
                    // Since the input `value` is not a register it cannot be overwritten.
                }
                I::CopySpan {
                    results: _,
                    values,
                    len,
                } => {
                    for value in values.iter_u16(*len) {
                        if value.within_range(start..end) {
                            return true;
                        }
                    }
                }
                unexpected => panic!("encountered non-copy instruction: {unexpected:?}"),
            }
        }
        // No overwrites detected so far thus we return `false`.
        false
    }

    /// Encodes an unconditional `return` instruction.
    pub fn encode_return(
        &mut self,
        stack: &mut ValueStack,
        types: &[ValueType],
        values: &[TypedProvider],
    ) -> Result<(), TranslationError> {
        assert_eq!(types.len(), values.len());
        let instr = match types {
            [] => {
                // Case: Function returns nothing therefore all return statements must return nothing.
                Instruction::Return
            }
            [ValueType::I32] => match values[0] {
                // Case: Function returns a single `i32` value which allows for special operator.
                TypedProvider::Register(value) => Instruction::return_reg(value),
                TypedProvider::Const(value) => Instruction::return_imm32(i32::from(value)),
            },
            [ValueType::I64] => match values[0] {
                // Case: Function returns a single `i64` value which allows for special operator.
                TypedProvider::Register(value) => Instruction::return_reg(value),
                TypedProvider::Const(value) => {
                    if let Some(value) = <Const32<i64>>::from_i64(i64::from(value)) {
                        Instruction::return_i64imm32(value)
                    } else {
                        Instruction::return_reg(stack.alloc_const(value)?)
                    }
                }
            },
            [ValueType::F32] => match values[0] {
                // Case: Function returns a single `f32` value which allows for special operator.
                TypedProvider::Register(value) => Instruction::return_reg(value),
                TypedProvider::Const(value) => Instruction::return_imm32(F32::from(value)),
            },
            [ValueType::F64] => match values[0] {
                // Case: Function returns a single `f64` value which may allow for special operator.
                TypedProvider::Register(value) => Instruction::return_reg(value),
                TypedProvider::Const(value) => {
                    if let Some(value) = <Const32<f64>>::from_f64(f64::from(value)) {
                        Instruction::return_f64imm32(value)
                    } else {
                        Instruction::return_reg(stack.alloc_const(value)?)
                    }
                }
            },
            [ValueType::FuncRef | ValueType::ExternRef] => {
                // Case: Function returns a single `externref` or `funcref`.
                match values[0] {
                    TypedProvider::Register(value) => Instruction::return_reg(value),
                    TypedProvider::Const(value) => {
                        Instruction::return_reg(stack.alloc_const(value)?)
                    }
                }
            }
            _ => {
                let values = self.encode_call_params(stack, values)?;
                Instruction::return_many(values)
            }
        };
        self.push_instr(instr)?;
        Ok(())
    }

    /// Encodes the call parameters of a `wasmi` `call` instruction if necessary.
    ///
    /// Returns the contiguous [`RegisterSpanIter`] that makes up the call parameters post encoding.
    ///
    /// # Errors
    ///
    /// If the translation runs out of register space during this operation.
    pub fn encode_call_params(
        &mut self,
        stack: &mut ValueStack,
        params: &[TypedProvider],
    ) -> Result<RegisterSpanIter, TranslationError> {
        if let Some(register_span) = RegisterSpanIter::from_providers(params) {
            // Case: we are on the happy path were the providers on the
            //       stack already are registers with contiguous indices.
            //
            //       This allows us to avoid copying over the registers
            //       to where the call instruction expects them on the stack.
            return Ok(register_span);
        }
        // Case: the providers on the stack need to be copied to the
        //       location where the call instruction expects its parameters
        //       before executing the call.
        let copy_results = stack.peek_dynamic_n(params.len())?.iter(params.len());
        self.encode_copies(stack, copy_results, params)?;
        Ok(copy_results)
    }

    /// Encodes the call parameters of a `wasmi` `call_indirect` instruction if necessary.
    ///
    /// Returns the contiguous [`RegisterSpanIter`] that makes up the call parameters post encoding.
    ///
    /// # Errors
    ///
    /// If the translation runs out of register space during this operation.
    pub fn encode_call_indirect_params(
        &mut self,
        stack: &mut ValueStack,
        mut index: TypedProvider,
        params: &[TypedProvider],
    ) -> Result<(TypedProvider, RegisterSpanIter), TranslationError> {
        if let Some(register_span) = RegisterSpanIter::from_providers(params) {
            // Case: we are on the happy path were the providers on the
            //       stack already are registers with contiguous indices.
            //
            //       This allows us to avoid copying over the registers
            //       to where the call instruction expects them on the stack.
            return Ok((index, register_span));
        }
        // Case: the providers on the stack need to be copied to the
        //       location where the call instruction expects its parameters
        //       before executing the call.
        let copy_results = stack.push_dynamic_n(params.len())?.iter(params.len());
        if let TypedProvider::Register(index_reg) = index {
            if copy_results.contains(index_reg) {
                // Case: the parameters are copied over to a contiguous span of registers
                //       that overwrites the `index` register. Thus we are required to copy
                //       the `index` register to a protected register.
                let copy_index = stack.push_dynamic()?;
                self.encode_copy(stack, copy_index, index)?;
                stack.pop();
                index = TypedProvider::Register(copy_index);
            }
        }
        self.encode_copies(stack, copy_results, params)?;
        stack.remove_n(params.len());
        Ok((index, copy_results))
    }

    /// Encode conditional branch parameters for `br_if` and `return_if` instructions.
    ///
    /// In contrast to [`InstrEncoder::encode_call_params`] this routine adds back original
    /// [`TypedProvider`] on the stack in case no copies are needed for them. This way the stack
    /// may not only contain dynamic registers after this operation.
    pub fn encode_conditional_branch_params(
        &mut self,
        stack: &mut ValueStack,
        params: &[TypedProvider],
    ) -> Result<RegisterSpanIter, TranslationError> {
        if let Some(register_span) = RegisterSpanIter::from_providers(params) {
            // Case: we are on the happy path were the providers on the
            //       stack already are registers with contiguous indices.
            //
            // This allows us to avoid copying over the registers
            // to where the call instruction expects them on the stack.
            //
            // Since we are translating conditional branches we have to
            // put the original providers back on the stack since no copies
            // were needed and nothing has changed.
            for param in params.iter().copied() {
                stack.push_provider(param)?;
            }
            return Ok(register_span);
        }
        // Case: the providers on the stack need to be copied to the
        //       location where the call instruction expects its parameters
        //       before executing the call.
        let copy_results = stack.push_dynamic_n(params.len())?.iter(params.len());
        self.encode_copies(stack, copy_results, params)?;
        Ok(copy_results)
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
