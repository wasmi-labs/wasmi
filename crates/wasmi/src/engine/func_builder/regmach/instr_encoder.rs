use core::mem;

use super::{DefragRegister, TypedProvider};
use crate::engine::{
    bytecode::BranchOffset,
    bytecode2::{Const32, Instruction, Register, RegisterSpan, RegisterSpanIter},
    func_builder::{
        labels::{LabelRef, LabelRegistry},
        regmach::stack::ValueStack,
        Instr,
        TranslationErrorInner,
    },
    TranslationError,
};
use alloc::vec::{Drain, Vec};
use wasmi_core::{UntypedValue, ValueType};

/// Encodes `wasmi` bytecode instructions to an [`Instruction`] stream.
#[derive(Debug, Default)]
pub struct InstrEncoder {
    /// Already encoded [`Instruction`] words.
    instrs: InstrSequence,
    /// Unresolved and unpinned labels created during function translation.
    labels: LabelRegistry,
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
}

impl InstrEncoder {
    /// Resets the [`InstrEncoder`].
    pub fn reset(&mut self) {
        self.instrs.reset();
        self.labels.reset();
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
                            let mut last_results = results.iter(len as usize);
                            let mut last_values = values.iter(len as usize);
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
        Ok(())
    }

    /// Encodes the call parameters of a `wasmi` call instruction if neccessary.
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
}

impl InstrEncoder {
    /// Pushes an [`Instruction::ConsumeFuel`] with base fuel costs to the [`InstrEncoder`].
    pub fn push_consume_fuel_instr(&mut self, block_fuel: u64) -> Result<Instr, TranslationError> {
        self.instrs.push(Instruction::consume_fuel(block_fuel)?)
    }
}

impl DefragRegister for InstrEncoder {
    fn defrag_register(&mut self, _user: Instr, _reg: Register, _new_reg: Register) {
        todo!() // TODO
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
