#[macro_use]
mod utils;
mod encoder;
mod labels;
mod layout;
mod locals;
mod op;
#[cfg(feature = "simd")]
mod simd;
mod stack;
mod visit;

use self::{
    encoder::{OpEncoder, OpEncoderAllocations, Pos},
    labels::{LabelRef, LabelRegistry},
    layout::{StackLayout, StackSpace},
    locals::{LocalIdx, LocalsRegistry},
    stack::{
        BlockControlFrame,
        ControlFrame,
        ControlFrameBase,
        ControlFrameKind,
        ElseControlFrame,
        ElseReachability,
        IfControlFrame,
        IfReachability,
        ImmediateOperand,
        LocalOperand,
        LoopControlFrame,
        Operand,
        Stack,
        StackAllocations,
        StackPos,
        TempOperand,
    },
    utils::{Input, Reset, ReusableAllocations, UpdateResultSlot},
};
#[cfg(feature = "simd")]
use crate::V128;
use crate::{
    Engine,
    Error,
    FuncType,
    TrapCode,
    ValType,
    core::{FuelCostsProvider, IndexType, Typed, TypedVal, UntypedVal},
    engine::{
        BlockType,
        CompiledFuncEntity,
        TranslationError,
        translator::{
            WasmTranslator,
            comparator::{
                CmpSelectFusion,
                LogicalizeCmpInstr,
                NegateCmpInstr,
                TryIntoCmpBranchInstr as _,
                TryIntoCmpSelectInstr as _,
                UpdateBranchOffset as _,
            },
            utils::{IntoShiftAmount, ToBits, WasmFloat, WasmInteger},
        },
    },
    ir::{
        self,
        Address,
        BoundedSlotSpan,
        BranchOffset,
        FixedSlotSpan,
        Offset16,
        Op,
        Sign,
        Slot,
        SlotSpan,
        index,
    },
    module::{FuncIdx, FuncTypeIdx, MemoryIdx, ModuleHeader, WasmiValueType},
};
use alloc::vec::Vec;
use core::convert::identity;
use wasmparser::{MemArg, WasmFeatures};

/// Type concerned with translating from Wasm bytecode to Wasmi bytecode.
#[derive(Debug)]
pub struct FuncTranslator {
    /// The reference to the Wasm module function under construction.
    func: FuncIdx,
    /// The engine for which the function is compiled.
    ///
    /// # Note
    ///
    /// Technically this is not needed since the information is redundant given via
    /// the `module` field. However, this acts like a faster access since `module`
    /// only holds a weak reference to the engine.
    engine: Engine,
    /// The immutable Wasmi module resources.
    module: ModuleHeader,
    /// This represents the reachability of the currently translated code.
    ///
    /// - `true`: The currently translated code is reachable.
    /// - `false`: The currently translated code is unreachable and can be skipped.
    ///
    /// # Note
    ///
    /// Visiting the Wasm `Else` or `End` control flow operator resets
    /// reachability to `true` again.
    reachable: bool,
    /// Wasm value and control stack.
    stack: Stack,
    /// Types of local variables and function parameters.
    locals: LocalsRegistry,
    /// Wasm layout to map stack slots to Wasmi registers.
    layout: StackLayout,
    /// Constructs and encodes function instructions.
    instrs: OpEncoder,
    /// Temporary buffer for operands.
    operands: Vec<Operand>,
    /// Temporary buffer for immediate values.
    immediates: Vec<TypedVal>,
}

/// Heap allocated data structured used by the [`FuncTranslator`].
#[derive(Debug, Default)]
pub struct FuncTranslatorAllocations {
    /// Wasm value and control stack.
    stack: StackAllocations,
    /// Types of local variables and function parameters.
    locals: LocalsRegistry,
    /// Wasm layout to map stack slots to Wasmi registers.
    layout: StackLayout,
    /// Constructs and encodes function instructions.
    instrs: OpEncoderAllocations,
    /// Temporary buffer for operands.
    operands: Vec<Operand>,
    /// Temporary buffer for immediate values.
    immediates: Vec<TypedVal>,
}

impl Reset for FuncTranslatorAllocations {
    fn reset(&mut self) {
        self.stack.reset();
        self.locals.reset();
        self.layout.reset();
        self.instrs.reset();
        self.operands.clear();
        self.immediates.clear();
    }
}

impl WasmTranslator<'_> for FuncTranslator {
    type Allocations = FuncTranslatorAllocations;

    fn setup(&mut self, _bytes: &[u8]) -> Result<bool, Error> {
        Ok(false)
    }

    fn features(&self) -> WasmFeatures {
        self.engine.config().wasm_features()
    }

    fn translate_locals(
        &mut self,
        amount: u32,
        value_type: wasmparser::ValType,
    ) -> Result<(), Error> {
        let ty = WasmiValueType::from(value_type).into_inner();
        self.register_locals(amount, ty)?;
        Ok(())
    }

    fn finish_translate_locals(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn update_pos(&mut self, _pos: usize) {}

    fn finish(
        mut self,
        finalize: impl FnOnce(CompiledFuncEntity),
    ) -> Result<Self::Allocations, Error> {
        // Note: `update_branch_offsets` might change `frame_size` so we need to compute it prior.
        //
        // Context:
        // This only happens if the function has so many instructions that some conditional branch
        // operators need to be encoded as their fallbacks which requires to allocate more function
        // local constant values, thus increasing the size of the function frame.
        self.instrs.update_branch_offsets()?;
        let Some(frame_size) = self.frame_size() else {
            return Err(Error::from(TranslationError::AllocatedTooManySlots));
        };
        finalize(CompiledFuncEntity::new(
            frame_size,
            self.instrs.encoded_ops(),
        ));
        Ok(self.into_allocations())
    }
}

impl ReusableAllocations for FuncTranslator {
    type Allocations = FuncTranslatorAllocations;

    fn into_allocations(self) -> Self::Allocations {
        Self::Allocations {
            stack: self.stack.into_allocations(),
            locals: self.locals,
            layout: self.layout,
            instrs: self.instrs.into_allocations(),
            operands: self.operands,
            immediates: self.immediates,
        }
    }
}

impl FuncTranslator {
    /// Creates a new [`FuncTranslator`].
    pub fn new(
        func: FuncIdx,
        module: ModuleHeader,
        alloc: FuncTranslatorAllocations,
    ) -> Result<Self, Error> {
        let Some(engine) = module.engine().upgrade() else {
            panic!(
                "cannot compile function since engine does no longer exist: {:?}",
                module.engine()
            )
        };
        let FuncTranslatorAllocations {
            stack,
            locals,
            layout,
            instrs,
            operands,
            immediates,
        } = alloc.into_reset();
        let stack = Stack::new(&engine, stack);
        let instrs = OpEncoder::new(&engine, instrs);
        let mut translator = Self {
            func,
            engine,
            module,
            reachable: true,
            stack,
            locals,
            layout,
            instrs,
            operands,
            immediates,
        };
        translator.init_func_body_block()?;
        translator.init_func_params()?;
        Ok(translator)
    }

    /// Initializes the function body enclosing control block.
    fn init_func_body_block(&mut self) -> Result<(), Error> {
        let func_ty = self.module.get_type_of_func(self.func);
        let block_ty = BlockType::func_type(func_ty);
        let end_label = self.instrs.new_label();
        let consume_fuel = self.instrs.encode_consume_fuel()?;
        self.stack
            .push_func_block(block_ty, end_label, consume_fuel)?;
        Ok(())
    }

    /// Initializes the function's parameters.
    fn init_func_params(&mut self) -> Result<(), Error> {
        for ty in self.func_type().params() {
            self.register_locals(1, *ty)?;
        }
        Ok(())
    }

    /// Slots an `amount` of local variables of type `ty`.
    fn register_locals(&mut self, amount: u32, ty: ValType) -> Result<(), Error> {
        let Ok(amount) = usize::try_from(amount) else {
            panic!(
                "failed to register {amount} local variables of type {ty:?}: out of bounds `usize`"
            )
        };
        self.locals.register(amount, ty)?;
        self.stack.register_locals(amount)?;
        self.layout.register_locals(amount)?;
        Ok(())
    }

    /// Returns the frame size of the to-be-compiled function.
    ///
    /// Returns `None` if the frame size is out of bounds.
    fn frame_size(&self) -> Option<u16> {
        let frame_size = self.stack.max_height().checked_add(self.locals.len())?;
        u16::try_from(frame_size).ok()
    }

    /// Returns the [`FuncType`] of the function that is currently translated.
    fn func_type(&self) -> FuncType {
        self.func_type_with(FuncType::clone)
    }

    /// Applies `f` to the [`FuncType`] of the function that is currently translated.
    fn func_type_with<R>(&self, f: impl FnOnce(&FuncType) -> R) -> R {
        self.resolve_func_type_with(self.func, f)
    }

    /// Returns the [`FuncType`] of the function at `func_index`.
    fn resolve_func_type(&self, func_index: FuncIdx) -> FuncType {
        self.resolve_func_type_with(func_index, FuncType::clone)
    }

    /// Applies `f` to the [`FuncType`] of the function at `func_index`.
    fn resolve_func_type_with<R>(&self, func_index: FuncIdx, f: impl FnOnce(&FuncType) -> R) -> R {
        let dedup_func_type = self.module.get_type_of_func(func_index);
        self.engine().resolve_func_type(dedup_func_type, f)
    }

    /// Resolves the [`FuncType`] at the given Wasm module `type_index`.
    fn resolve_type(&self, type_index: u32) -> FuncType {
        let func_type_idx = FuncTypeIdx::from(type_index);
        let dedup_func_type = self.module.get_func_type(func_type_idx);
        self.engine()
            .resolve_func_type(dedup_func_type, Clone::clone)
    }

    /// Returns the [`Engine`] for which the function is compiled.
    fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Copy the top-most `len` operands to [`Operand::Temp`] values.
    ///
    /// # Note
    ///
    /// - The top-most `len` operands on the [`Stack`] will be [`Operand::Temp`] upon completion.
    /// - Does nothing if an [`Operand`] is already an [`Operand::Temp`].
    fn move_operands_to_temp(
        &mut self,
        len: usize,
        consume_fuel: Option<Pos<ir::BlockFuel>>,
    ) -> Result<SlotSpan, Error> {
        debug_assert!(len > 0);
        for n in 0..len {
            let operand = self.stack.operand_to_temp(n);
            self.copy_operand_to_temp(operand, consume_fuel)?;
        }
        let first_idx = self.stack.peek(len - 1).index();
        let first = self.layout.temp_to_slot(first_idx)?;
        Ok(SlotSpan::new(first))
    }

    /// Convert all branch params up to `depth` to [`Operand::Temp`].
    ///
    /// # Note
    ///
    /// - The top-most `depth` operands on the [`Stack`] will be [`Operand::Temp`] upon completion.
    /// - Does nothing if an [`Operand`] is already an [`Operand::Temp`].
    fn copy_branch_params(
        &mut self,
        target: &impl ControlFrameBase,
        consume_fuel_instr: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        let len_branch_params = target.len_branch_params(&self.engine);
        let Some(branch_results) = self.frame_results(target)? else {
            return Ok(());
        };
        self.encode_copies(branch_results, len_branch_params, consume_fuel_instr)?;
        Ok(())
    }

    /// Pushes the temporary results of the control `frame` onto the [`Stack`].
    ///
    /// # Note
    ///
    /// - Before pushing the results, the [`Stack`] is truncated to the `frame`'s height.
    /// - Not all control frames have temporary results, e.g. Wasm `loop`s, Wasm `if`s with
    ///   a compile-time known branch or Wasm `block`s that are never branched to, do not
    ///   require to call this function.
    fn push_frame_results(&mut self, frame: &impl ControlFrameBase) -> Result<(), Error> {
        let height = frame.height();
        self.stack.trunc(height);
        frame
            .ty()
            .func_type_with(&self.engine, |func_ty| -> Result<(), Error> {
                for result in func_ty.results() {
                    self.stack.push_temp(*result)?;
                }
                Ok(())
            })?;
        Ok(())
    }

    /// Encodes a copy instruction for the top-most `len_values` on the stack to `results`.
    ///
    /// # Note
    ///
    /// - This does _not_ pop values from the stack or manipulate the stack otherwise.
    /// - This might allocate new function local constant values if necessary.
    /// - This does _not_ encode a copy if the copy is a no-op.
    fn encode_copies(
        &mut self,
        results: SlotSpan,
        len_values: u16,
        consume_fuel_instr: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        match len_values {
            0 => Ok(()),
            1 => {
                let result = results.head();
                let value = self.stack.peek(0);
                self.encode_copy(result, value, consume_fuel_instr)?;
                Ok(())
            }
            _ => self.encode_copy_many(results, len_values, consume_fuel_instr),
        }
    }

    /// Convenience wrapper for [`Self::encode_copy_impl`].
    fn encode_copy(
        &mut self,
        result: Slot,
        value: Operand,
        consume_fuel_instr: Option<Pos<ir::BlockFuel>>,
    ) -> Result<Option<Pos<Op>>, Error> {
        Self::encode_copy_impl(
            result,
            value,
            consume_fuel_instr,
            &mut self.layout,
            &mut self.instrs,
        )
    }

    /// Encodes a single copy instruction.
    ///
    /// # Note
    ///
    /// This won't encode a copy if `result` and `value` yields a no-op copy.
    fn encode_copy_impl(
        result: Slot,
        value: Operand,
        consume_fuel_instr: Option<Pos<ir::BlockFuel>>,
        layout: &mut StackLayout,
        encoder: &mut OpEncoder,
    ) -> Result<Option<Pos<Op>>, Error> {
        let Some(copy_instr) = Self::make_copy_instr(result, value, layout)? else {
            // Case: no-op copy instruction
            return Ok(None);
        };
        let pos = encoder.encode(copy_instr, consume_fuel_instr, FuelCostsProvider::base)?;
        Ok(Some(pos))
    }

    /// Returns the copy instruction to copy the given `operand` to `result`.
    ///
    /// Returns `None` if the resulting copy instruction is a no-op.
    fn make_copy_instr(
        result: Slot,
        value: Operand,
        layout: &mut StackLayout,
    ) -> Result<Option<Op>, Error> {
        let instr = match value {
            Operand::Temp(value) => {
                let ty = value.ty();
                let value = layout.temp_to_slot(value)?;
                if result == value {
                    // Case: no-op copy
                    return Ok(None);
                }
                match ty {
                    ValType::V128 => {
                        let results = SlotSpan::new(result);
                        let values = SlotSpan::new(value);
                        let Some(op) = Self::make_copy_span(results, values, 2) else {
                            return Ok(None);
                        };
                        op
                    }
                    _ => Op::copy(result, value),
                }
            }
            Operand::Local(value) => {
                let ty = value.ty();
                let value = layout.local_to_slot(value)?;
                if result == value {
                    // Case: no-op copy
                    return Ok(None);
                }
                match ty {
                    ValType::V128 => {
                        let results = SlotSpan::new(result);
                        let values = SlotSpan::new(value);
                        let Some(op) = Self::make_copy_span(results, values, 2) else {
                            return Ok(None);
                        };
                        op
                    }
                    _ => Op::copy(result, value),
                }
            }
            Operand::Immediate(value) => Self::make_copy_imm_instr(result, value.val())?,
        };
        Ok(Some(instr))
    }

    /// Returns the copy instruction to copy the given immediate `value` to `result`.
    fn make_copy_imm_instr(result: Slot, value: TypedVal) -> Result<Op, Error> {
        let instr = match value.ty() {
            ValType::I32 => Op::copy32(result, i32::from(value).to_bits()),
            ValType::I64 => Op::copy64(result, i64::from(value).to_bits()),
            ValType::F32 => Op::copy32(result, f32::from(value).to_bits()),
            ValType::F64 => Op::copy64(result, f64::from(value).to_bits()),
            ValType::ExternRef | ValType::FuncRef => {
                Op::copy64(result, u64::from(UntypedVal::from(value)))
            }
            #[cfg(feature = "simd")]
            ValType::V128 => {
                let value = V128::from(value).as_u128();
                let value_lo = (value & 0xFFFF_FFFF_FFFF_FFFF) as u64;
                let value_hi = (value >> 64) as u64;
                Op::copy128(result, value_lo, value_hi)
            }
            #[cfg(not(feature = "simd"))]
            ValType::V128 => panic!("unexpected `v128` operand: {value:?}"),
        };
        Ok(instr)
    }

    /// Encode a copy instruction that copies a contiguous span of values.
    ///
    /// # Note
    ///
    /// This won't encode a copy if the resulting copy instruction is a no-op.
    fn encode_copy_span(
        &mut self,
        results: SlotSpan,
        values: SlotSpan,
        len: u16,
        consume_fuel_instr: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        let Some(op) = Self::make_copy_span(results, values, len) else {
            // Case: results and values are equal and therefore the copy is a no-op
            return Ok(());
        };
        self.instrs
            .encode(op, consume_fuel_instr, |costs: &FuelCostsProvider| {
                costs.fuel_for_copying_values(u64::from(len))
            })?;
        Ok(())
    }

    /// Returns an [`Op::copy_span_asc`] or [`Op::copy_span_des`] depending on inputs.
    ///
    /// Returns `None` if the `copy_span` operation is a no-op.
    fn make_copy_span(results: SlotSpan, values: SlotSpan, len: u16) -> Option<Op> {
        if results == values {
            // Case: results and values are equal and therefore the copy is a no-op
            return None;
        }
        let copy_span = match results.head() > values.head() {
            true => Op::copy_span_des,
            false => Op::copy_span_asc,
        };
        Some(copy_span(results, values, len))
    }

    /// Encode a copy instruction that copies many values.
    ///
    /// # Note
    ///
    /// - This won't encode a copy if the resulting copy instruction is a no-op.
    /// - Encodes either `copy`, `copy2`, `copy_span` or `copy_many` depending on the amount
    ///   of noop copies between `results` and `values`.
    fn encode_copy_many(
        &mut self,
        results: SlotSpan,
        len: u16,
        consume_fuel_instr: Option<Pos<ir::BlockFuel>>,
    ) -> Result<(), Error> {
        self.peek_operands_into_buffer(usize::from(len));
        let values = &self.operands[..];
        let (results, values) = Self::copy_many_strip_noop_start(results, values, &self.layout)?;
        let values = Self::copy_many_strip_noop_end(results, values, &self.layout)?;
        let len = values.len() as u16;
        debug_assert!(!Self::has_overlapping_copies(
            results,
            values,
            &self.layout
        )?);
        match values {
            [] => return Ok(()),
            [val0] => {
                let result = results.head();
                let value = *val0;
                self.encode_copy(result, value, consume_fuel_instr)?;
                return Ok(());
            }
            _values => {}
        }
        debug_assert!(!values.is_empty());
        if let Some(values) = Self::try_form_regspan_of(values, &self.layout)? {
            return self.encode_copy_span(results, values, len, consume_fuel_instr);
        }
        let values = Self::copy_operands_to_temp(
            values,
            consume_fuel_instr,
            &mut self.layout,
            &mut self.instrs,
        )?;
        self.encode_copy_span(results, values, len, consume_fuel_instr)
    }

    /// Copy `values` to temporary stack [`Slot`]s without changing the translation stack.
    fn copy_operands_to_temp(
        values: &[Operand],
        pos_fuel: Option<Pos<ir::BlockFuel>>,
        layout: &mut StackLayout,
        instrs: &mut OpEncoder,
    ) -> Result<SlotSpan, Error> {
        debug_assert!(!values.is_empty());
        for value in values {
            let result = layout.temp_to_slot(value.index())?;
            let value = *value;
            Self::encode_copy_impl(result, value, pos_fuel, layout, instrs)?;
        }
        let first = layout.temp_to_slot(values[0].index())?;
        let span = SlotSpan::new(first);
        Ok(span)
    }

    /// Tries to strip noop copies from the start of the `copy_many`.
    ///
    /// Returns the stripped `results` [`SlotSpan`] and `values` slice of [`Operand`]s.
    fn copy_many_strip_noop_start<'a>(
        results: SlotSpan,
        values: &'a [Operand],
        layout: &StackLayout,
    ) -> Result<(SlotSpan, &'a [Operand]), Error> {
        let mut result = results.head();
        let mut values = values;
        while let Some((value, rest)) = values.split_first() {
            let value = match value {
                Operand::Local(value) => layout.local_to_slot(value)?,
                Operand::Temp(value) => layout.temp_to_slot(value)?,
                Operand::Immediate(_) => {
                    // Immediate values will never yield no-op copies.
                    break;
                }
            };
            if result != value {
                // Can no longer strip no-op copies from the start.
                break;
            }
            result = result.next();
            values = rest;
        }
        Ok((SlotSpan::new(result), values))
    }

    /// Tries to strip noop copies from the end of the `copy_many`.
    ///
    /// Returns the stripped `values` slice of [`Operand`]s.
    fn copy_many_strip_noop_end<'a>(
        results: SlotSpan,
        values: &'a [Operand],
        layout: &StackLayout,
    ) -> Result<&'a [Operand], Error> {
        let Ok(len) = u16::try_from(values.len()) else {
            panic!("out of bounds `copy_many` values length: {}", values.len())
        };
        let mut result = results.head().next_n(len);
        let mut values = values;
        while let Some((value, rest)) = values.split_last() {
            let value = match value {
                Operand::Local(value) => layout.local_to_slot(value)?,
                Operand::Temp(value) => layout.temp_to_slot(value)?,
                Operand::Immediate(_) => {
                    // Immediate values will never yield no-op copies.
                    break;
                }
            };
            result = result.prev();
            if result != value {
                // Can no longer strip no-op copies from the end.
                break;
            }
            values = rest;
        }
        Ok(values)
    }

    /// Returns `true` if there are overlapping copies with `results` and `values`.
    ///
    /// # Examples
    ///
    /// - `[ 0 <- 1, 1 <- 1, 2 <- 4 ]` has no overlapping copies.
    /// - `[ 0 <- 1, 1 <- 0 ]` has overlapping copies since register `0`
    ///   is written to in the first copy but read from in the next.
    /// - `[ 3 <- 1, 4 <- 2, 5 <- 3 ]` has overlapping copies since register `3`
    ///   is written to in the first copy but read from in the third.
    fn has_overlapping_copies(
        results: SlotSpan,
        values: &[Operand],
        layout: &StackLayout,
    ) -> Result<bool, Error> {
        if values.is_empty() {
            // An empty set of copies can never have overlapping copies.
            return Ok(false);
        }
        let Ok(len) = u16::try_from(values.len()) else {
            panic!("operand span too large: len={}", values.len());
        };
        let result0 = results.head();
        for (result, value) in results.iter(len).zip(values) {
            // Note: We only have to check the register case since constant value
            //       copies can never overlap.
            let value = match value {
                Operand::Local(value) => layout.local_to_slot(value)?,
                Operand::Temp(value) => layout.temp_to_slot(value)?,
                Operand::Immediate(_) => {
                    // Immediates are allocated as function local constants
                    // which can not collide with the result registers.
                    continue;
                }
            };
            if result0 <= value && value < result {
                // Case: `value` is in the range of `result0..result` which
                //       means it has been overwritten by previous copies,
                //       thus we detected a collission.
                return Ok(true);
            }
        }
        // No copy collissions have been found.
        Ok(false)
    }

    /// Returns the results [`SlotSpan`] of the `frame` if any.
    fn frame_results(&self, frame: &impl ControlFrameBase) -> Result<Option<SlotSpan>, Error> {
        Self::frame_results_impl(frame, &self.engine, &self.layout)
    }

    /// Returns the results [`SlotSpan`] of the `frame` if any.
    fn frame_results_impl(
        frame: &impl ControlFrameBase,
        engine: &Engine,
        layout: &StackLayout,
    ) -> Result<Option<SlotSpan>, Error> {
        if frame.len_branch_params(engine) == 0 {
            return Ok(None);
        }
        let height = frame.height();
        let start = layout.temp_to_slot(StackPos::from(height))?;
        let span = SlotSpan::new(start);
        Ok(Some(span))
    }

    /// Returns `true` if the [`ControlFrame`] at `depth` requires copying for its branch parameters.
    ///
    /// # Note
    ///
    /// Some instructions can be encoded in a more efficient way if no branch parameter copies are required.
    fn requires_branch_param_copies(&self, depth: usize) -> bool {
        let frame = self.stack.peek_control(depth);
        let len_branch_params = usize::from(frame.len_branch_params(&self.engine));
        let frame_height = frame.height();
        let height_matches = frame_height == (self.stack.height() - len_branch_params);
        let only_temps = (0..len_branch_params)
            .map(|depth| self.stack.peek(depth))
            .all(|o| o.is_temp());
        let can_avoid_copies = height_matches && only_temps;
        !can_avoid_copies
    }

    /// Convert the [`Operand`] at `depth` into an [`Operand::Temp`] by copying if necessary.
    ///
    /// # Note
    ///
    /// Does nothing if the [`Operand`] is already an [`Operand::Temp`].
    fn copy_operand_to_temp(
        &mut self,
        operand: Operand,
        consume_fuel: Option<Pos<ir::BlockFuel>>,
    ) -> Result<Slot, Error> {
        let result = self.layout.temp_to_slot(operand.index())?;
        self.encode_copy(result, operand, consume_fuel)?;
        Ok(result)
    }

    /// Copies the `operand` to its temporary [`Slot`] if it is an immediate.
    ///
    /// Returns the temporary [`Slot`] of the `operand`.
    ///
    /// # Note
    ///
    /// - Returns the associated [`Slot`] if `operand` is an [`Operand::Temp`] or [`Operand::Local`].
    fn copy_if_immediate(&mut self, operand: Operand) -> Result<Slot, Error> {
        match operand {
            Operand::Local(operand) => self.layout.local_to_slot(operand),
            Operand::Temp(operand) => self.layout.temp_to_slot(operand),
            Operand::Immediate(operand) => {
                let value = operand.val();
                let result = self.layout.temp_to_slot(operand.operand_index())?;
                let copy_instr = Self::make_copy_imm_instr(result, value)?;
                let consume_fuel = self.stack.consume_fuel_instr();
                self.instrs
                    .encode(copy_instr, consume_fuel, FuelCostsProvider::base)?;
                Ok(result)
            }
        }
    }

    /// Preserves all local operands on the stack.
    ///
    /// # Note
    ///
    /// This works by encoding copy instructions to `temp` register space.
    fn preserve_all_locals(&mut self) -> Result<(), Error> {
        let consume_fuel_instr = self.stack.consume_fuel_instr();
        for local in self.stack.preserve_all_locals() {
            debug_assert!(matches!(local, Operand::Local(_)));
            let result = self.layout.temp_to_slot(local.index())?;
            let Some(copy_instr) = Self::make_copy_instr(result, local, &mut self.layout)? else {
                unreachable!("`result` and `local` refer to different stack spaces");
            };
            self.instrs
                .encode(copy_instr, consume_fuel_instr, FuelCostsProvider::base)?;
        }
        Ok(())
    }

    /// Pushes the `instr` to the function with the associated `fuel_costs`.
    fn push_instr(
        &mut self,
        instr: Op,
        fuel_costs: impl FnOnce(&FuelCostsProvider) -> u64,
    ) -> Result<Pos<Op>, Error> {
        debug_assert!(instr.result_ref().is_none());
        let consume_fuel = self.stack.consume_fuel_instr();
        let instr = self.instrs.encode(instr, consume_fuel, fuel_costs)?;
        Ok(instr)
    }

    /// Pushes the `instr` to the function with the associated `fuel_costs`.
    fn push_instr_with_result(
        &mut self,
        result_ty: ValType,
        make_instr: impl FnOnce(Slot) -> Op,
        fuel_costs: impl FnOnce(&FuelCostsProvider) -> u64,
    ) -> Result<(), Error> {
        let consume_fuel_instr = self.stack.consume_fuel_instr();
        let result = self.layout.temp_to_slot(self.stack.push_temp(result_ty)?)?;
        let op = make_instr(result);
        debug_assert!(op.result_ref().is_some());
        self.instrs.stage(op, consume_fuel_instr, fuel_costs)?;
        Ok(())
    }

    /// Pushes a binary instruction with a result and associated fuel costs.
    fn push_binary_instr_with_result(
        &mut self,
        result_ty: ValType,
        lhs: Operand,
        rhs: Operand,
        make_instr: impl FnOnce(Slot, Slot, Slot) -> Op,
        fuel_costs: impl FnOnce(&FuelCostsProvider) -> u64,
    ) -> Result<(), Error> {
        debug_assert_eq!(lhs.ty(), rhs.ty());
        let lhs = self.layout.operand_to_slot(lhs)?;
        let rhs = self.layout.operand_to_slot(rhs)?;
        self.push_instr_with_result(result_ty, |result| make_instr(result, lhs, rhs), fuel_costs)
    }

    /// Populate the `buffer` with the `table` targets including the `table` default target.
    ///
    /// Returns a shared slice to the `buffer` after it has been filled.
    ///
    /// # Note
    ///
    /// The `table` default target is pushed last to the `buffer`.
    fn copy_targets_from_br_table(
        table: &wasmparser::BrTable,
        buffer: &mut Vec<TypedVal>,
    ) -> Result<(), Error> {
        let default_target = table.default();
        buffer.clear();
        for target in table.targets() {
            buffer.push(TypedVal::from(target?));
        }
        buffer.push(TypedVal::from(default_target));
        Ok(())
    }

    /// Encodes a Wasm `br_table` that does not copy branching values.
    ///
    /// # Note
    ///
    /// Upon call the `immediates` buffer contains all `br_table` target values.
    fn encode_br_table_0(&mut self, table: wasmparser::BrTable, index: Slot) -> Result<(), Error> {
        // We add +1 because we include the default target here.
        let len_targets = table.len() + 1;
        debug_assert_eq!(self.immediates.len(), len_targets as usize);
        self.push_instr(
            Op::branch_table(len_targets, index),
            FuelCostsProvider::base,
        )?;
        // Encode the `br_table` targets:
        let fuel_pos = self.stack.consume_fuel_instr();
        let targets = &self.immediates[..];
        for target in targets {
            let Ok(depth) = usize::try_from(u32::from(*target)) else {
                panic!("out of bounds `br_table` target does not fit `usize`: {target:?}");
            };
            let mut frame = self.stack.peek_control_mut(depth).control_frame();
            self.instrs
                .encode_branch(frame.label(), identity, fuel_pos, 0)?;
            frame.branch_to();
        }
        Ok(())
    }

    /// Encodes a Wasm `br_table` that has to copy `len_values` branching values.
    ///
    /// # Note
    ///
    /// Upon call the `immediates` buffer contains all `br_table` target values.
    fn encode_br_table_n(
        &mut self,
        table: wasmparser::BrTable,
        index: Slot,
        len_values: u16,
    ) -> Result<(), Error> {
        debug_assert_eq!(self.immediates.len(), (table.len() + 1) as usize);
        let consume_fuel_instr = self.stack.consume_fuel_instr();
        let values = self.try_form_regspan_or_move(usize::from(len_values), consume_fuel_instr)?;
        self.push_instr(
            Op::branch_table_span(table.len() + 1, index, values, len_values),
            FuelCostsProvider::base,
        )?;
        // Encode the `br_table` targets:
        let fuel_pos = self.stack.consume_fuel_instr();
        let targets = &self.immediates[..];
        for target in targets {
            let Ok(depth) = usize::try_from(u32::from(*target)) else {
                panic!("out of bounds `br_table` target does not fit `usize`: {target:?}");
            };
            let mut frame = self.stack.peek_control_mut(depth).control_frame();
            let Some(results) = Self::frame_results_impl(&frame, &self.engine, &self.layout)?
            else {
                panic!("must have frame results since `br_table` requires to copy values");
            };
            self.instrs.encode_branch(
                frame.label(),
                |offset| ir::BranchTableTarget::new(results, offset),
                fuel_pos,
                0,
            )?;
            frame.branch_to();
        }
        Ok(())
    }

    /// Encodes a generic return instruction.
    fn encode_return(
        &mut self,
        consume_fuel: Option<Pos<ir::BlockFuel>>,
    ) -> Result<Pos<Op>, Error> {
        let len_results = self.func_type_with(FuncType::len_results);
        let instr = match len_results {
            0 => Op::Return {},
            1 => match self.stack.peek(0) {
                Operand::Local(operand) => {
                    let value = self.layout.local_to_slot(operand)?;
                    Op::return_slot(value)
                }
                Operand::Temp(operand) => {
                    let value = self.layout.temp_to_slot(operand)?;
                    Op::return_slot(value)
                }
                Operand::Immediate(operand) => {
                    let val = operand.val();
                    match operand.ty() {
                        ValType::I32 => Op::return32(i32::from(val).to_bits()),
                        ValType::I64 => Op::return64(i64::from(val).to_bits()),
                        ValType::F32 => Op::return32(f32::from(val).to_bits()),
                        ValType::F64 => Op::return64(f64::from(val).to_bits()),
                        ValType::FuncRef | ValType::ExternRef => {
                            Op::return64(u64::from(UntypedVal::from(val)))
                        }
                        ValType::V128 => {
                            let value = self.stack.peek(0);
                            let temp_slot = self.copy_operand_to_temp(value, consume_fuel)?;
                            Op::return_span(BoundedSlotSpan::new(SlotSpan::new(temp_slot), 2))
                        }
                    }
                }
            },
            _ => {
                self.move_operands_to_temp(usize::from(len_results), consume_fuel)?;
                let result0 = self.stack.peek(usize::from(len_results) - 1);
                let slot0 = self.layout.temp_to_slot(result0.index())?;
                Op::return_span(BoundedSlotSpan::new(SlotSpan::new(slot0), len_results))
            }
        };
        let instr = self
            .instrs
            .encode(instr, consume_fuel, FuelCostsProvider::base)?;
        Ok(instr)
    }

    /// Store the top-most [`Operand`]s on the [`Stack`] into the operands buffer.
    fn peek_operands_into_buffer(&mut self, len: usize) {
        self.operands.clear();
        self.operands.extend(self.stack.peek_n(len));
    }

    /// Tries to form a [`SlotSpan`] from the top-most `n` operands on the [`Stack`].
    ///
    /// Returns `None` if forming a [`SlotSpan`] was not possible.
    fn try_form_regspan(&self, len: usize) -> Result<Option<SlotSpan>, Error> {
        Self::try_form_regspan_of(self.stack.peek_n(len), &self.layout)
    }

    /// Tries to form a [`SlotSpan`] from the `values` slice of [`Operand`]s.
    ///
    /// Returns `None` if forming a [`SlotSpan`] was not possible.
    fn try_form_regspan_of<T>(
        values: impl IntoIterator<Item = T>,
        layout: &StackLayout,
    ) -> Result<Option<SlotSpan>, Error>
    where
        T: AsRef<Operand>,
    {
        let mut values = values.into_iter();
        let Some(head) = values.next() else {
            return Ok(None);
        };
        let mut head = match head.as_ref() {
            Operand::Local(start) => layout.local_to_slot(start)?,
            Operand::Temp(start) => layout.temp_to_slot(start)?,
            Operand::Immediate(_) => return Ok(None),
        };
        let start = head;
        for value in values {
            let cur = match value.as_ref() {
                Operand::Immediate(_) => return Ok(None),
                Operand::Local(value) => layout.local_to_slot(value)?,
                Operand::Temp(value) => layout.temp_to_slot(value)?,
            };
            if head != cur.prev() {
                return Ok(None);
            }
            head = cur;
        }
        Ok(Some(SlotSpan::new(start)))
    }

    /// Tries to form a [`SlotSpan`] from the top-most `len` operands on the [`Stack`] or copy to temporaries.
    ///
    /// Returns `None` if forming a [`SlotSpan`] was not possible.
    fn try_form_regspan_or_move(
        &mut self,
        len: usize,
        consume_fuel_instr: Option<Pos<ir::BlockFuel>>,
    ) -> Result<SlotSpan, Error> {
        if let Some(span) = self.try_form_regspan(len)? {
            return Ok(span);
        }
        self.move_operands_to_temp(len, consume_fuel_instr)?;
        let Some(span) = self.try_form_regspan(len)? else {
            unreachable!(
                "the top-most `len` operands are now temporaries thus `SlotSpan` forming should succeed"
            )
        };
        Ok(span)
    }

    /// Translates the end of a Wasm `block` control frame.
    fn translate_end_block(&mut self, frame: BlockControlFrame) -> Result<(), Error> {
        let consume_fuel_instr = frame.consume_fuel_instr();
        if frame.is_branched_to() {
            if self.reachable {
                self.copy_branch_params(&frame, consume_fuel_instr)?;
            }
            self.push_frame_results(&frame)?;
        }
        self.instrs.pin_label(frame.label())?;
        self.reachable |= frame.is_branched_to();
        if self.reachable && self.stack.is_control_empty() {
            self.encode_return(consume_fuel_instr)?;
        }
        Ok(())
    }

    /// Translates the end of a Wasm `loop` control frame.
    fn translate_end_loop(&mut self, _frame: LoopControlFrame) -> Result<(), Error> {
        debug_assert!(!self.stack.is_control_empty());
        // Nothing needs to be done since Wasm `loop` control frames always only have a single exit.
        //
        // Note: no need to reset `last_instr` since end of `loop` is not a control flow boundary.
        Ok(())
    }

    /// Translates the end of a Wasm `if` control frame.
    fn translate_end_if(&mut self, frame: IfControlFrame) -> Result<(), Error> {
        debug_assert!(!self.stack.is_control_empty());
        let is_end_of_then_reachable = self.reachable;
        let IfReachability::Both { else_label } = frame.reachability() else {
            let is_end_reachable = match frame.reachability() {
                IfReachability::OnlyThen => self.reachable,
                IfReachability::OnlyElse => true,
                IfReachability::Both { .. } => unreachable!(),
            };
            return self.translate_end_if_or_else_only(frame, is_end_reachable);
        };
        let len_results = frame.ty().len_results(self.engine());
        let has_results = len_results >= 1;
        if is_end_of_then_reachable && has_results {
            let consume_fuel_instr = frame.consume_fuel_instr();
            self.copy_branch_params(&frame, consume_fuel_instr)?;
            self.instrs.encode_branch(
                frame.label(),
                Op::branch,
                consume_fuel_instr,
                FuelCostsProvider::base,
            )?;
        }
        self.instrs.pin_label_if_unpinned(else_label)?;
        self.stack.push_else_operands(&frame)?;
        if has_results {
            // We haven't visited the `else` block and thus the `else`
            // providers are still on the auxiliary stack and need to
            // be popped. We use them to restore the stack to the state
            // when entering the `if` block so that we can properly copy
            // the `else` results to were they are expected.
            let consume_fuel_instr = self.instrs.encode_consume_fuel()?;
            self.copy_branch_params(&frame, consume_fuel_instr)?;
        }
        self.push_frame_results(&frame)?;
        self.instrs.pin_label(frame.label())?;
        self.reachable = true;
        Ok(())
    }

    /// Translates the end of a Wasm `else` control frame.
    fn translate_end_else(&mut self, frame: ElseControlFrame) -> Result<(), Error> {
        debug_assert!(!self.stack.is_control_empty());
        match frame.reachability() {
            ElseReachability::OnlyThen {
                is_end_of_then_reachable,
            } => {
                return self.translate_end_if_or_else_only(frame, is_end_of_then_reachable);
            }
            ElseReachability::OnlyElse => {
                return self.translate_end_if_or_else_only(frame, self.reachable);
            }
            _ => {}
        };
        let end_of_then_reachable = frame.is_end_of_then_reachable();
        let end_of_else_reachable = self.reachable;
        let reachable = match (end_of_then_reachable, end_of_else_reachable) {
            (false, false) => frame.is_branched_to(),
            _ => true,
        };
        if end_of_else_reachable {
            let consume_fuel_instr: Option<Pos<ir::BlockFuel>> = frame.consume_fuel_instr();
            self.copy_branch_params(&frame, consume_fuel_instr)?;
        }
        self.push_frame_results(&frame)?;
        self.instrs.pin_label(frame.label())?;
        self.reachable = reachable;
        Ok(())
    }

    /// Translates the end of a Wasm `else` control frame where only one branch is known to be reachable.
    fn translate_end_if_or_else_only(
        &mut self,
        frame: impl ControlFrameBase,
        end_is_reachable: bool,
    ) -> Result<(), Error> {
        if frame.is_branched_to() {
            if end_is_reachable {
                let consume_fuel_instr = frame.consume_fuel_instr();
                self.copy_branch_params(&frame, consume_fuel_instr)?;
            }
            self.push_frame_results(&frame)?;
        }
        self.instrs.pin_label(frame.label())?;
        self.reachable = end_is_reachable || frame.is_branched_to();
        Ok(())
    }

    /// Translates the end of an unreachable Wasm control frame.
    fn translate_end_unreachable(&mut self, _frame: ControlFrameKind) -> Result<(), Error> {
        debug_assert!(!self.stack.is_control_empty());
        // We reset `last_instr` out of caution in case there is a control flow boundary.
        self.instrs.try_encode_staged()?;
        Ok(())
    }

    /// Translate the Wasm `local.set` and `local.tee` operations.
    ///
    /// # Note
    ///
    /// This applies op-code fusion that replaces the result of the previous instruction
    /// instead of encoding a copy instruction for the `local.set` or `local.tee` if possible.
    fn translate_local_set(&mut self, local_index: u32, push_result: bool) -> Result<(), Error> {
        bail_unreachable!(self);
        let input = self.stack.pop();
        let input_ty = input.ty();
        if let Operand::Local(input) = input {
            if u32::from(input.local_index()) == local_index {
                // Case: `(local.set $n (local.get $n))` is a no-op so we can ignore it.
                //
                // Note: This does not require any preservation since it won't change
                //       the value of `local $n`.
                if push_result {
                    // Need to push back input before we exit.
                    self.stack.push_operand(input.into())?;
                }
                return Ok(());
            }
        }
        let local_idx = LocalIdx::from(local_index);
        let consume_fuel_instr = self.stack.consume_fuel_instr();
        for preserved in self.stack.preserve_locals(local_idx) {
            let result = self.layout.temp_to_slot(preserved)?;
            let value = self.layout.local_to_slot(local_idx)?;
            self.instrs.encode(
                Op::copy(result, value),
                consume_fuel_instr,
                FuelCostsProvider::base,
            )?;
        }
        if push_result {
            match input {
                Operand::Immediate(input) => {
                    self.stack.push_immediate(input.val())?;
                }
                _ => {
                    self.stack.push_local(local_idx, input_ty)?;
                }
            }
        }
        if self.try_replace_result(local_idx, input)? {
            // Case: it was possible to replace the result of the previous
            //       instructions so no copy instruction is required.
            return Ok(());
        }
        // At this point we need to encode a copy instruction.
        let result = self.layout.local_to_slot(local_idx)?;
        let outcome = self.encode_copy(result, input, consume_fuel_instr)?;
        debug_assert!(
            outcome.is_some(),
            "no-op copy cases have been filtered out already"
        );
        Ok(())
    }

    /// Tries to replace the result of the previous instruction with `new_result` if possible.
    ///
    /// Returns `Ok(true)` if replacement was successful and `Ok(false)` otherwise.
    fn try_replace_result(
        &mut self,
        new_result: LocalIdx,
        old_result: Operand,
    ) -> Result<bool, Error> {
        let Some(mut staged) = self.instrs.peek_staged() else {
            // Case: cannot replace result without staged operator.
            return Ok(false);
        };
        let new_result = self.layout.local_to_slot(new_result)?;
        debug_assert!(matches!(
            self.layout.stack_space(new_result),
            StackSpace::Local
        ));
        let old_result = match old_result {
            Operand::Temp(old_result) => self.layout.temp_to_slot(old_result)?,
            Operand::Local(_) | Operand::Immediate(_) => {
                // Case immediate: cannot replace immediate value result.
                // Case     local: cannot replace local with another local due to observable behavior.
                return Ok(false);
            }
        };
        let Some(staged_result) = staged.result_mut() else {
            // Case: staged has no result and thus cannot have its result changed.
            return Ok(false);
        };
        if *staged_result != old_result {
            // Case: staged result does not match `old_result` and thus is not available for mutation.
            return Ok(false);
        }
        *staged_result = new_result;
        let (fuel_pos, fuel_used) = self.instrs.drop_staged();
        self.instrs.encode(staged, fuel_pos, fuel_used)?;
        Ok(true)
    }

    /// Encodes an unconditional Wasm `branch` instruction.
    fn encode_br(&mut self, label: LabelRef) -> Result<Pos<Op>, Error> {
        let fuel_pos = self.stack.consume_fuel_instr();
        let (br_op, _) =
            self.instrs
                .encode_branch(label, Op::branch, fuel_pos, FuelCostsProvider::base)?;
        Ok(br_op)
    }

    /// Encodes a `i32.eqz`+`br_if` or `if` conditional branch instruction.
    fn encode_br_eqz(&mut self, condition: Operand, label: LabelRef) -> Result<(), Error> {
        self.encode_br_if(condition, label, true)
    }

    /// Encodes a `br_if` conditional branch instruction.
    fn encode_br_nez(&mut self, condition: Operand, label: LabelRef) -> Result<(), Error> {
        self.encode_br_if(condition, label, false)
    }

    /// Encodes a generic `br_if` fused conditional branch instruction.
    fn encode_br_if(
        &mut self,
        condition: Operand,
        label: LabelRef,
        branch_eqz: bool,
    ) -> Result<(), Error> {
        if self.try_fuse_branch_cmp(condition, label, branch_eqz)? {
            return Ok(());
        }
        let condition = match condition {
            Operand::Local(condition) => self.layout.local_to_slot(condition)?,
            Operand::Temp(condition) => self.layout.temp_to_slot(condition)?,
            Operand::Immediate(condition) => {
                let condition = i32::from(condition.val());
                let take_branch = match branch_eqz {
                    true => condition == 0,
                    false => condition != 0,
                };
                match take_branch {
                    true => {
                        self.encode_br(label)?;
                        self.reachable = false;
                        return Ok(());
                    }
                    false => return Ok(()),
                }
            }
        };
        let fuel_pos = self.stack.consume_fuel_instr();
        self.instrs.encode_branch(
            label,
            |offset| match branch_eqz {
                true => Op::branch_i32_eq_si(offset, condition, 0),
                false => Op::branch_i32_not_eq_si(offset, condition, 0),
            },
            fuel_pos,
            FuelCostsProvider::base,
        )?;
        Ok(())
    }

    /// Try to fuse a cmp+branch [`Op`] with optional negation.
    fn try_fuse_branch_cmp(
        &mut self,
        condition: Operand,
        label: LabelRef,
        negate: bool,
    ) -> Result<bool, Error> {
        let Some(staged_op) = self.instrs.peek_staged() else {
            // Case: cannot fuse without a known last instruction
            return Ok(false);
        };
        let Operand::Temp(condition) = condition else {
            // Case: cannot fuse non-temporary operands
            //  - locals have observable behavior.
            //  - immediates cannot be the result of a previous instruction.
            return Ok(false);
        };
        debug_assert!(matches!(condition.ty(), ValType::I32 | ValType::I64));
        let Some(cmp_result) = staged_op.result_ref().copied() else {
            // Note: `cmp` operators must have a result.
            return Ok(false);
        };
        if matches!(self.layout.stack_space(cmp_result), StackSpace::Local) {
            // Note: local variable results have observable behavior which must not change.
            return Ok(false);
        }
        let br_condition = self.layout.temp_to_slot(condition)?;
        if cmp_result != br_condition {
            // Note: cannot fuse cmp instruction with a result that differs
            //       from the branch condition operand.
            return Ok(false);
        }
        let cmp_op = match negate {
            false => staged_op,
            true => match staged_op.negate_cmp_instr() {
                Some(negated) => negated,
                None => {
                    // Note: cannot negate staged [`Op`], thus it is not a `cmp` operator and thus not fusable.
                    return Ok(false);
                }
            },
        };
        let Some(fused_cmp_branch) = cmp_op.try_into_cmp_branch_instr(BranchOffset::uninit())
        else {
            return Ok(false);
        };
        let (fuel_pos, _) = self.instrs.drop_staged();
        self.instrs.encode_branch(
            label,
            |offset| fused_cmp_branch.with_branch_offset(offset),
            fuel_pos,
            FuelCostsProvider::base,
        )?;
        Ok(true)
    }

    /// Generically translates a `call` or `return_call` Wasm operator.
    fn translate_call(
        &mut self,
        function_index: u32,
        call_internal: fn(params: BoundedSlotSpan, func: index::InternalFunc) -> Op,
        call_imported: fn(params: BoundedSlotSpan, func: index::Func) -> Op,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let consume_fuel = self.stack.consume_fuel_instr();
        let func_idx = FuncIdx::from(function_index);
        let callee_ty = self.resolve_func_type(func_idx);
        let params = self.adjust_stack_for_call(&callee_ty, consume_fuel)?;
        let instr = match self.module.get_engine_func(func_idx) {
            Some(engine_func) => {
                // Case: We are calling an internal function and can optimize
                //       this case by using the special instruction for it.
                call_internal(params, index::InternalFunc::from(engine_func))
            }
            None => {
                // Case: We are calling an imported function and must use the
                //       general calling operator for it.
                call_imported(params, index::Func::from(function_index))
            }
        };
        self.push_instr(instr, FuelCostsProvider::call)?;
        Ok(())
    }

    /// Generically translates a `call_indirect` or `return_call_indirect` Wasm operator.
    fn translate_call_indirect(
        &mut self,
        type_index: u32,
        table_index: u32,
        make_instr: fn(
            params: BoundedSlotSpan,
            index: Slot,
            func_type: index::FuncType,
            table: index::Table,
        ) -> Op,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let index = self.stack.pop();
        let consume_fuel = self.stack.consume_fuel_instr();
        let table = index::Table::from(table_index);
        let callee_ty = self.resolve_type(type_index);
        let index = self.copy_if_immediate(index)?;
        let params = self.adjust_stack_for_call(&callee_ty, consume_fuel)?;
        self.push_instr(
            make_instr(params, index, index::FuncType::from(type_index), table),
            FuelCostsProvider::call,
        )?;
        Ok(())
    }

    /// Adjusts the stack for a call to a function with type `ty`.
    ///
    /// Returns a bounded [`SlotSpan`] to the start of the call
    /// parameters and results.
    fn adjust_stack_for_call(
        &mut self,
        ty: &FuncType,
        fuel_pos: Option<Pos<ir::BlockFuel>>,
    ) -> Result<BoundedSlotSpan, Error> {
        let len_params = ty.len_params();
        for _ in 0..len_params {
            let operand = self.stack.pop();
            self.copy_operand_to_temp(operand, fuel_pos)?;
        }
        let height = self.stack.height();
        let start = self.layout.temp_to_slot(StackPos::from(height))?;
        let params = BoundedSlotSpan::new(SlotSpan::new(start), len_params);
        for result in ty.results() {
            self.stack.push_temp(*result)?;
        }
        Ok(params)
    }

    /// Translates a unary Wasm instruction to Wasmi bytecode.
    fn translate_unary<T, R>(
        &mut self,
        make_instr: fn(result: Slot, input: Slot) -> Op,
        consteval: fn(input: T) -> R,
    ) -> Result<(), Error>
    where
        T: From<TypedVal>,
        R: Into<TypedVal> + Typed,
    {
        bail_unreachable!(self);
        let input = self.stack.pop();
        if let Operand::Immediate(input) = input {
            self.stack.push_immediate(consteval(input.val().into()))?;
            return Ok(());
        }
        let input = self.layout.operand_to_slot(input)?;
        self.push_instr_with_result(
            <R as Typed>::TY,
            |result| make_instr(result, input),
            FuelCostsProvider::base,
        )
    }

    /// Translates a unary Wasm instruction to Wasmi bytecode.
    fn translate_unary_fallible<T, R>(
        &mut self,
        make_instr: fn(result: Slot, input: Slot) -> Op,
        consteval: fn(input: T) -> Result<R, TrapCode>,
    ) -> Result<(), Error>
    where
        T: From<TypedVal>,
        R: Into<TypedVal> + Typed,
    {
        bail_unreachable!(self);
        let input = self.stack.pop();
        if let Operand::Immediate(input) = input {
            let input = T::from(input.val());
            match consteval(input) {
                Ok(result) => {
                    self.stack.push_immediate(result)?;
                }
                Err(trap) => {
                    self.translate_trap(trap)?;
                }
            }
            return Ok(());
        }
        let input = self.layout.operand_to_slot(input)?;
        self.push_instr_with_result(
            <R as Typed>::TY,
            |result| make_instr(result, input),
            FuelCostsProvider::base,
        )
    }

    /// Translate a generic Wasm reinterpret-like operation.
    ///
    /// # Note
    ///
    /// This Wasm operation is a no-op. Ideally we only have to change the types on the stack.
    fn translate_reinterpret<T, R>(&mut self, consteval: fn(T) -> R) -> Result<(), Error>
    where
        T: From<TypedVal> + Typed,
        R: Into<TypedVal> + Typed,
    {
        bail_unreachable!(self);
        match self.stack.pop() {
            Operand::Local(input) => {
                debug_assert_eq!(input.ty(), <T as Typed>::TY);
                self.stack
                    .push_local(input.local_index(), <R as Typed>::TY)?;
            }
            Operand::Temp(input) => {
                debug_assert_eq!(input.ty(), <T as Typed>::TY);
                self.stack.push_temp(<R as Typed>::TY)?;
            }
            Operand::Immediate(input) => {
                let input: T = input.val().into();
                self.stack.push_immediate(consteval(input))?;
            }
        }
        Ok(())
    }

    /// Create a new generic [`Input`] from the given `operand`.
    fn make_input<R>(
        &mut self,
        operand: Operand,
        f: impl FnOnce(&mut Self, TypedVal) -> Result<Input<R>, Error>,
    ) -> Result<Input<R>, Error> {
        let reg = match operand {
            Operand::Local(operand) => self.layout.local_to_slot(operand)?,
            Operand::Temp(operand) => self.layout.temp_to_slot(operand)?,
            Operand::Immediate(operand) => return f(self, operand.val()),
        };
        Ok(Input::Slot(reg))
    }

    /// Converts the `provider` to a 64-bit index-type constant value.
    ///
    /// # Note
    ///
    /// This expects `operand` to be either `u32` or `u64` if `memory64` is enabled or disabled respectively.
    pub(super) fn make_index64(
        &mut self,
        operand: Operand,
        index_type: IndexType,
    ) -> Result<Input<u64>, Error> {
        let value = match operand {
            Operand::Immediate(value) => value.val(),
            Operand::Local(value) => {
                debug_assert_eq!(operand.ty(), index_type.ty());
                let reg = self.layout.local_to_slot(value)?;
                return Ok(Input::Slot(reg));
            }
            Operand::Temp(value) => {
                debug_assert_eq!(operand.ty(), index_type.ty());
                let reg = self.layout.temp_to_slot(value)?;
                return Ok(Input::Slot(reg));
            }
        };
        let value = match index_type {
            IndexType::I32 => u64::from(u32::from(value)),
            IndexType::I64 => u64::from(value),
        };
        Ok(Input::Immediate(value))
    }

    /// Copies `operand` to a temporary stack slot if it is an immediate that cannot be encoded using 32-bits.
    ///
    /// - Returns [`Input::Slot`] if `operand` is a local or a temporary operand.
    /// - Returns [`Input::Immediate`] if `operand` is an immediate that can be encoded as 32-bit value.
    /// - Returns [`Input::Slot`] otherwise and encodes a copy storing the immediate into its temporary stack slot.
    fn make_index32_or_copy(
        &mut self,
        operand: Operand,
        index_ty: IndexType,
    ) -> Result<Input<u32>, Error> {
        let index64 = match self.make_index64(operand, index_ty)? {
            Input::Slot(index) => return Ok(Input::Slot(index)),
            Input::Immediate(index) => index,
        };
        let index32 = match u32::try_from(index64) {
            Ok(index) => return Ok(Input::Immediate(index)),
            Err(_) => self.copy_if_immediate(operand)?,
        };
        Ok(Input::Slot(index32))
    }

    /// Evaluates `consteval(lhs, rhs)` and pushed either its result or tranlates a `trap`.
    fn translate_binary_consteval_fallible<T, R>(
        &mut self,
        lhs: ImmediateOperand,
        rhs: ImmediateOperand,
        consteval: impl FnOnce(T, T) -> Result<R, TrapCode>,
    ) -> Result<(), Error>
    where
        T: From<TypedVal>,
        R: Into<TypedVal>,
    {
        let lhs: T = lhs.val().into();
        let rhs: T = rhs.val().into();
        match consteval(lhs, rhs) {
            Ok(value) => {
                self.stack.push_immediate(value)?;
            }
            Err(trap) => {
                self.translate_trap(trap)?;
            }
        }
        Ok(())
    }

    /// Evaluates `consteval(lhs, rhs)` and pushed either its result or tranlates a `trap`.
    fn translate_binary_consteval<T, R>(
        &mut self,
        lhs: ImmediateOperand,
        rhs: ImmediateOperand,
        consteval: fn(T, T) -> R,
    ) -> Result<(), Error>
    where
        T: From<TypedVal>,
        R: Into<TypedVal>,
    {
        self.translate_binary_consteval_fallible::<T, R>(lhs, rhs, |lhs, rhs| {
            Ok(consteval(lhs, rhs))
        })
    }

    /// Convenience method to tell that there is no custom optimization.
    fn no_opt_ri<T>(&mut self, _lhs: Operand, _rhs: T) -> Result<bool, Error> {
        Ok(false)
    }

    /// Translates a commutative binary Wasm operator to Wasmi bytecode.
    fn translate_binary_commutative<T, R>(
        &mut self,
        make_sss: fn(result: Slot, lhs: Slot, rhs: Slot) -> Op,
        make_ssi: fn(result: Slot, lhs: Slot, rhs: T) -> Op,
        consteval: fn(T, T) -> R,
        opt_si: fn(this: &mut Self, lhs: Operand, rhs: T) -> Result<bool, Error>,
    ) -> Result<(), Error>
    where
        T: From<TypedVal> + Copy,
        R: Into<TypedVal> + Typed,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
            (Operand::Immediate(lhs), Operand::Immediate(rhs)) => {
                self.translate_binary_consteval::<T, R>(lhs, rhs, consteval)
            }
            (val, Operand::Immediate(imm)) | (Operand::Immediate(imm), val) => {
                let rhs = imm.val().into();
                if opt_si(self, val, rhs)? {
                    return Ok(());
                }
                let lhs = self.layout.operand_to_slot(val)?;
                self.push_instr_with_result(
                    <R as Typed>::TY,
                    |result| make_ssi(result, lhs, rhs),
                    FuelCostsProvider::base,
                )
            }
            (lhs, rhs) => self.push_binary_instr_with_result(
                <R as Typed>::TY,
                lhs,
                rhs,
                make_sss,
                FuelCostsProvider::base,
            ),
        }
    }

    /// Translates integer division and remainder Wasm operators to Wasmi bytecode.
    fn translate_divrem<T>(
        &mut self,
        make_instr_sss: fn(result: Slot, lhs: Slot, rhs: Slot) -> Op,
        make_instr_ssi: fn(result: Slot, lhs: Slot, rhs: <T as WasmInteger>::NonZero) -> Op,
        make_instr_sis: fn(result: Slot, lhs: T, rhs: Slot) -> Op,
        consteval: fn(T, T) -> Result<T, TrapCode>,
    ) -> Result<(), Error>
    where
        T: WasmInteger,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
            (Operand::Immediate(lhs), Operand::Immediate(rhs)) => {
                self.translate_binary_consteval_fallible::<T, T>(lhs, rhs, consteval)
            }
            (lhs, Operand::Immediate(rhs)) => {
                let lhs = self.layout.operand_to_slot(lhs)?;
                let rhs = T::from(rhs.val());
                let Some(non_zero_rhs) = <T as WasmInteger>::non_zero(rhs) else {
                    // Optimization: division by zero always traps
                    return self.translate_trap(TrapCode::IntegerDivisionByZero);
                };
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| make_instr_ssi(result, lhs, non_zero_rhs),
                    FuelCostsProvider::base,
                )
            }
            (Operand::Immediate(lhs), rhs) => {
                let lhs = T::from(lhs.val());
                let rhs = self.layout.operand_to_slot(rhs)?;
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| make_instr_sis(result, lhs, rhs),
                    FuelCostsProvider::base,
                )
            }
            (lhs, rhs) => self.push_binary_instr_with_result(
                <T as Typed>::TY,
                lhs,
                rhs,
                make_instr_sss,
                FuelCostsProvider::base,
            ),
        }
    }

    /// Translates binary non-commutative Wasm operators to Wasmi bytecode.
    fn translate_binary<T, R>(
        &mut self,
        make_instr_sss: fn(result: Slot, lhs: Slot, rhs: Slot) -> Op,
        make_instr_ssi: fn(result: Slot, lhs: Slot, rhs: T) -> Op,
        make_instr_sis: fn(result: Slot, lhs: T, rhs: Slot) -> Op,
        consteval: fn(T, T) -> R,
    ) -> Result<(), Error>
    where
        T: From<TypedVal> + Copy,
        R: Into<TypedVal> + Typed,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
            (Operand::Immediate(lhs), Operand::Immediate(rhs)) => {
                self.translate_binary_consteval::<T, R>(lhs, rhs, consteval)
            }
            (lhs, Operand::Immediate(rhs)) => {
                let lhs = self.layout.operand_to_slot(lhs)?;
                let rhs = T::from(rhs.val());
                self.push_instr_with_result(
                    <R as Typed>::TY,
                    |result| make_instr_ssi(result, lhs, rhs),
                    FuelCostsProvider::base,
                )
            }
            (Operand::Immediate(lhs), rhs) => {
                let lhs = T::from(lhs.val());
                let rhs = self.layout.operand_to_slot(rhs)?;
                self.push_instr_with_result(
                    <R as Typed>::TY,
                    |result| make_instr_sis(result, lhs, rhs),
                    FuelCostsProvider::base,
                )
            }
            (lhs, rhs) => self.push_binary_instr_with_result(
                <R as Typed>::TY,
                lhs,
                rhs,
                make_instr_sss,
                FuelCostsProvider::base,
            ),
        }
    }

    /// Translates Wasm shift and rotate operators to Wasmi bytecode.
    fn translate_shift<T>(
        &mut self,
        make_instr_sss: fn(result: Slot, lhs: Slot, rhs: Slot) -> Op,
        make_instr_ssi: fn(result: Slot, lhs: Slot, rhs: <T as IntoShiftAmount>::ShiftAmount) -> Op,
        make_instr_sis: fn(result: Slot, lhs: T, rhs: Slot) -> Op,
        consteval: fn(T, T) -> T,
    ) -> Result<(), Error>
    where
        T: WasmInteger + IntoShiftAmount<ShiftSource: From<TypedVal>>,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
            (Operand::Immediate(lhs), Operand::Immediate(rhs)) => {
                self.translate_binary_consteval::<T, T>(lhs, rhs, consteval)
            }
            (lhs, Operand::Immediate(rhs)) => {
                let shift_amount = <T::ShiftSource>::from(rhs.val());
                let Some(rhs) = T::into_shift_amount(shift_amount) else {
                    // Optimization: Shifting or rotating by zero bits is a no-op.
                    self.stack.push_operand(lhs)?;
                    return Ok(());
                };
                let lhs = self.layout.operand_to_slot(lhs)?;
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| make_instr_ssi(result, lhs, rhs),
                    FuelCostsProvider::base,
                )
            }
            (Operand::Immediate(lhs), rhs) => {
                let lhs = T::from(lhs.val());
                if lhs.is_zero() {
                    // Optimization: Shifting or rotating a zero value is a no-op.
                    self.stack.push_immediate(lhs)?;
                    return Ok(());
                }
                let rhs = self.layout.operand_to_slot(rhs)?;
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| make_instr_sis(result, lhs, rhs),
                    FuelCostsProvider::base,
                )
            }
            (lhs, rhs) => self.push_binary_instr_with_result(
                <T as Typed>::TY,
                lhs,
                rhs,
                make_instr_sss,
                FuelCostsProvider::base,
            ),
        }
    }

    /// Translate Wasmi `{f32,f64}.copysign` instructions.
    ///
    /// # Note
    ///
    /// - This applies some optimization that are valid for copysign instructions.
    /// - Applies constant evaluation if both operands are constant values.
    fn translate_fcopysign<T>(
        &mut self,
        make_instr_sss: fn(result: Slot, lhs: Slot, rhs: Slot) -> Op,
        make_instr_ssi: fn(result: Slot, lhs: Slot, rhs: Sign<T>) -> Op,
        make_instr_sis: fn(result: Slot, lhs: T, rhs: Slot) -> Op,
        consteval: fn(T, T) -> T,
    ) -> Result<(), Error>
    where
        T: WasmFloat,
    {
        bail_unreachable!(self);
        match self.stack.pop2() {
            (Operand::Immediate(lhs), Operand::Immediate(rhs)) => {
                self.translate_binary_consteval::<T, T>(lhs, rhs, consteval)
            }
            (lhs, Operand::Immediate(rhs)) => {
                let lhs = self.layout.operand_to_slot(lhs)?;
                let sign = T::from(rhs.val()).sign();
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| make_instr_ssi(result, lhs, sign),
                    FuelCostsProvider::base,
                )
            }
            (Operand::Immediate(lhs), rhs) => {
                let lhs = T::from(lhs.val());
                let rhs = self.layout.operand_to_slot(rhs)?;
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| make_instr_sis(result, lhs, rhs),
                    FuelCostsProvider::base,
                )
            }
            (lhs, rhs) => {
                if lhs.is_same(&rhs) {
                    // Optimization: `copysign x x` is always just `x`
                    self.stack.push_operand(lhs)?;
                    return Ok(());
                }
                self.push_binary_instr_with_result(
                    <T as Typed>::TY,
                    lhs,
                    rhs,
                    make_instr_sss,
                    FuelCostsProvider::base,
                )
            }
        }
    }

    /// Translates a generic trap instruction.
    fn translate_trap(&mut self, trap: TrapCode) -> Result<(), Error> {
        self.push_instr(Op::trap(trap), FuelCostsProvider::base)?;
        self.reachable = false;
        Ok(())
    }

    /// Translates a Wasm `select` or `select <ty>` instruction.
    ///
    /// # Note
    ///
    /// - This applies constant propagation in case `condition` is a constant value.
    /// - If both `lhs` and `rhs` are equal registers or constant values `lhs` is forwarded.
    /// - Fuses compare instructions with the associated select instructions if possible.
    fn translate_select(&mut self, type_hint: Option<ValType>) -> Result<(), Error> {
        bail_unreachable!(self);
        let (true_val, false_val, condition) = self.stack.pop3();
        if let Some(type_hint) = type_hint {
            debug_assert_eq!(true_val.ty(), type_hint);
            debug_assert_eq!(false_val.ty(), type_hint);
        }
        let ty = true_val.ty();
        if true_val.is_same(&false_val) {
            // Optimization: both `lhs` and `rhs` either are the same register or constant values and
            //               thus `select` will always yield this same value irrespective of the condition.
            self.stack.push_operand(true_val)?;
            return Ok(());
        }
        let condition = match condition {
            Operand::Immediate(condition) => {
                let condition = i32::from(condition.val()) != 0;
                let selected = match condition {
                    true => true_val,
                    false => false_val,
                };
                if let Operand::Temp(_) = selected {
                    // Case: the selected operand is a temporary which needs to be copied
                    //       if it was the `false_val` since it changed its index. This is
                    //       not the case for the `true_val` since `true_val` is the first
                    //       value popped from the stack.
                    let consume_fuel_instr = self.stack.consume_fuel_instr();
                    let result = self.layout.temp_to_slot(self.stack.push_temp(ty)?)?;
                    let Some(op) = Self::make_copy_instr(result, selected, &mut self.layout)?
                    else {
                        return Ok(());
                    };
                    debug_assert!(op.result_ref().is_some());
                    self.instrs
                        .stage(op, consume_fuel_instr, FuelCostsProvider::base)?;
                    return Ok(());
                }
                self.stack.push_operand(selected)?;
                return Ok(());
            }
            Operand::Local(condition) => self.layout.local_to_slot(condition)?,
            Operand::Temp(condition) => self.layout.temp_to_slot(condition)?,
        };
        let true_val = self.copy_if_immediate(true_val)?;
        let false_val = self.copy_if_immediate(false_val)?;
        #[cfg(feature = "simd")]
        if matches!(ty, ValType::V128) {
            // Case: for `v128` values the `select128` instruction must be used.
            // Unlike normal `select` instructions the `select128` cannot be fused.
            self.push_instr_with_result(
                ty,
                |result| Op::select128(result, condition, false_val, true_val),
                FuelCostsProvider::base,
            )?;
            return Ok(());
        }
        if !self.try_fuse_select(ty, condition, true_val, false_val)? {
            self.push_instr_with_result(
                ty,
                |result| Op::select_i32_eq_ssi(result, false_val, true_val, condition, 0_i32),
                FuelCostsProvider::base,
            )?;
        };
        Ok(())
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
        condition: Slot,
        true_val: Slot,
        false_val: Slot,
    ) -> Result<bool, Error> {
        let Some(staged) = self.instrs.peek_staged() else {
            // If there is no last instruction there is no comparison instruction to negate.
            return Ok(false);
        };
        let Some(staged_result) = staged.result_ref().copied() else {
            // All negatable instructions have a single result register.
            return Ok(false);
        };
        if matches!(self.layout.stack_space(staged_result), StackSpace::Local) {
            // The operator stores its result into a local variable which
            // is an observable side effect which we are not allowed to mutate.
            return Ok(false);
        }
        if staged_result != condition {
            // The result of the last instruction and the select's `condition`
            // are not equal thus indicating that we cannot fuse the instructions.
            return Ok(false);
        }
        let CmpSelectFusion::Applied(fused_select) =
            staged.try_into_cmp_select_instr(true_val, false_val, || {
                let select_result = self.stack.push_temp(ty)?;
                let select_result = self.layout.temp_to_slot(select_result)?;
                Ok(select_result)
            })?
        else {
            return Ok(false);
        };
        self.instrs.replace_staged(fused_select)?;
        Ok(true)
    }

    /// Tries to fuse a Wasm `i32.eqz` (or `i32.eq` with 0 `rhs` value) instruction.
    ///
    /// Returns
    ///
    /// - `Ok(true)` if the intruction fusion was successful.
    /// - `Ok(false)` if instruction fusion could not be applied.
    /// - `Err(_)` if an error occurred.
    pub fn fuse_eqz<T: WasmInteger>(&mut self, lhs: Operand, rhs: T) -> Result<bool, Error> {
        self.fuse_commutative_cmp_with(lhs, rhs, NegateCmpInstr::negate_cmp_instr)
    }

    /// Tries to fuse a Wasm `i32.ne` instruction with 0 `rhs` value.
    ///
    /// Returns
    ///
    /// - `Ok(true)` if the intruction fusion was successful.
    /// - `Ok(false)` if instruction fusion could not be applied.
    /// - `Err(_)` if an error occurred.
    pub fn fuse_nez<T: WasmInteger>(&mut self, lhs: Operand, rhs: T) -> Result<bool, Error> {
        self.fuse_commutative_cmp_with(lhs, rhs, LogicalizeCmpInstr::logicalize_cmp_instr)
    }

    /// Tries to fuse a `i{32,64}`.{eq,ne}` instruction with `rhs` of zero.
    ///
    /// Generically applies `f` onto the fused last instruction.
    ///
    /// Returns
    ///
    /// - `Ok(true)` if the intruction fusion was successful.
    /// - `Ok(false)` if instruction fusion could not be applied.
    /// - `Err(_)` if an error occurred.
    pub fn fuse_commutative_cmp_with<T: WasmInteger>(
        &mut self,
        lhs: Operand,
        rhs: T,
        try_fuse: fn(cmp: &Op) -> Option<Op>,
    ) -> Result<bool, Error> {
        if !rhs.is_zero() {
            // Case: cannot fuse with non-zero `rhs`
            return Ok(false);
        }
        let Some(staged) = self.instrs.peek_staged() else {
            // Case: cannot fuse without registered last instruction
            return Ok(false);
        };
        let Operand::Temp(lhs) = lhs else {
            // Case: cannot fuse non-temporary operands
            //  - locals have observable behavior.
            //  - immediates cannot be the result of a previous instruction.
            return Ok(false);
        };
        let lhs_reg = self.layout.temp_to_slot(lhs)?;
        let Some(result) = staged.result_ref().copied() else {
            // Case: cannot fuse non-cmp instructions
            return Ok(false);
        };
        if result != lhs_reg {
            // Case: the `cmp` instruction does not feed into the `eqz` and cannot be fused
            return Ok(false);
        }
        let Some(negated) = try_fuse(&staged) else {
            // Case: the `cmp` instruction cannot be negated
            return Ok(false);
        };
        // Need to push back `lhs` but with its type adjusted to be `i32`
        // since that's the return type of `i{32,64}.{eqz,eq,ne}`.
        let result_idx = self.stack.push_temp(ValType::I32)?;
        // Need to replace `cmp` instruction result register since it might
        // have been misaligned if `lhs` originally referred to the zero operand.
        let new_result = self.layout.temp_to_slot(result_idx)?;
        let Some(negated) = negated.update_result_slot(new_result) else {
            unreachable!("`negated` has been asserted as `cmp` instruction");
        };
        self.instrs.replace_staged(negated)?;
        Ok(true)
    }

    /// Translates a Wasm `load` instruction to Wasmi bytecode.
    ///
    /// # Note
    ///
    /// This chooses the right encoding for the given `load` instruction.
    /// If `ptr+offset` is a constant value the address is pre-calculated.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to Wasmi bytecode:
    ///
    /// - `{i32, i64, f32, f64}.load`
    /// - `i32.{load8_s, load8_u, load16_s, load16_u}`
    /// - `i64.{load8_s, load8_u, load16_s, load16_u load32_s, load32_u}`
    fn translate_load<T: op::LoadOperator>(&mut self, memarg: MemArg) -> Result<(), Error> {
        bail_unreachable!(self);
        let (memory, offset) = Self::decode_memarg(memarg)?;
        let ptr = self.stack.pop();
        let ptr = match ptr {
            Operand::Local(ptr) => self.layout.local_to_slot(ptr)?,
            Operand::Temp(ptr) => self.layout.temp_to_slot(ptr)?,
            Operand::Immediate(ptr) => {
                let Some(address) = self.effective_address(memory, ptr.val(), offset) else {
                    return self.translate_trap(TrapCode::MemoryOutOfBounds);
                };
                match T::load_si(address, memory) {
                    Some(load_si) => {
                        self.push_instr_with_result(
                            T::LOADED_TY,
                            load_si,
                            FuelCostsProvider::load,
                        )?;
                        return Ok(());
                    }
                    None => {
                        let consume_fuel = self.stack.consume_fuel_instr();
                        self.copy_operand_to_temp(ptr.into(), consume_fuel)?
                    }
                }
            }
        };
        if memory.is_default() {
            if let Ok(offset) = Offset16::try_from(offset) {
                self.push_instr_with_result(
                    T::LOADED_TY,
                    |result| T::load_mem0_offset16_ss(result, ptr, offset),
                    FuelCostsProvider::load,
                )?;
                return Ok(());
            }
        }
        self.push_instr_with_result(
            T::LOADED_TY,
            |result| T::load_ss(result, ptr, offset, memory),
            FuelCostsProvider::load,
        )?;
        Ok(())
    }

    /// Translates Wasm integer `store` and `storeN` instructions to Wasmi bytecode.
    ///
    /// # Note
    ///
    /// This chooses the most efficient encoding for the given `store` instruction.
    /// If `ptr+offset` is a constant value the pointer address is pre-calculated.
    ///
    /// # Usage
    ///
    /// Used for translating the following Wasm operators to Wasmi bytecode:
    ///
    /// - `{i32, i64}.{store, store8, store16, store32}`
    fn translate_store<T: op::StoreOperator>(&mut self, memarg: MemArg) -> Result<(), Error>
    where
        T::Value: Copy + From<TypedVal>,
        T::Immediate: Copy,
    {
        bail_unreachable!(self);
        let (ptr, value) = self.stack.pop2();
        self.encode_store::<T>(memarg, ptr, value)
    }

    /// Encodes a Wasm store operator to Wasmi bytecode.
    fn encode_store<T: op::StoreOperator>(
        &mut self,
        memarg: MemArg,
        ptr: Operand,
        value: Operand,
    ) -> Result<(), Error>
    where
        T::Value: Copy + From<TypedVal>,
        T::Immediate: Copy,
    {
        let (memory, offset) = Self::decode_memarg(memarg)?;
        let ptr = match ptr {
            Operand::Local(ptr) => self.layout.local_to_slot(ptr)?,
            Operand::Temp(ptr) => self.layout.temp_to_slot(ptr)?,
            Operand::Immediate(ptr) => {
                return self.encode_store_ix::<T>(ptr, offset, memory, value);
            }
        };
        if self.encode_store_mem0_offset16::<T>(ptr, offset, memory, value)? {
            return Ok(());
        }
        let store_op = match value {
            Operand::Local(value) => {
                let value = self.layout.local_to_slot(value)?;
                T::store_ss(ptr, offset, value, memory)
            }
            Operand::Temp(value) => {
                let value = self.layout.temp_to_slot(value)?;
                T::store_ss(ptr, offset, value, memory)
            }
            Operand::Immediate(value) => {
                let value = <T::Value>::from(value.val());
                let immediate = <T as op::StoreOperator>::into_immediate(value);
                T::store_si(ptr, offset, immediate, memory)
            }
        };
        self.push_instr(store_op, FuelCostsProvider::store)?;
        Ok(())
    }

    /// Encodes a Wasm store operator with immediate `ptr` to Wasmi bytecode.
    fn encode_store_ix<T: op::StoreOperator>(
        &mut self,
        ptr: ImmediateOperand,
        offset: u64,
        memory: index::Memory,
        value: Operand,
    ) -> Result<(), Error>
    where
        T::Value: Copy + From<TypedVal>,
        T::Immediate: Copy,
    {
        let Some(address) = self.effective_address(memory, ptr.val(), offset) else {
            return self.translate_trap(TrapCode::MemoryOutOfBounds);
        };
        let store_op = match value {
            Operand::Local(value) => {
                let value = self.layout.local_to_slot(value)?;
                T::store_is(address, value, memory)
            }
            Operand::Temp(value) => {
                let value = self.layout.temp_to_slot(value)?;
                T::store_is(address, value, memory)
            }
            Operand::Immediate(value) => {
                let value = <T::Value>::from(value.val());
                let immediate = <T as op::StoreOperator>::into_immediate(value);
                T::store_ii(address, immediate, memory)
            }
        };
        self.push_instr(store_op, FuelCostsProvider::store)?;
        Ok(())
    }

    /// Encodes a Wasm store operator with `(mem 0)` and 16-bit encodable `offset` to Wasmi bytecode.
    ///
    /// # Note
    ///
    /// - Returns `Ok(true)` if encoding was successfull.
    /// - Returns `Ok(false)` if encoding was unsuccessful.
    /// - Returns `Err(_)` if an error occurred.
    fn encode_store_mem0_offset16<T: op::StoreOperator>(
        &mut self,
        ptr: Slot,
        offset: u64,
        memory: index::Memory,
        value: Operand,
    ) -> Result<bool, Error>
    where
        T::Value: Copy + From<TypedVal>,
        T::Immediate: Copy,
    {
        if !memory.is_default() {
            return Ok(false);
        }
        let Ok(offset16) = Offset16::try_from(offset) else {
            return Ok(false);
        };
        let store_op = match value {
            Operand::Local(value) => {
                let value = self.layout.local_to_slot(value)?;
                T::store_mem0_offset16_ss(ptr, offset16, value)
            }
            Operand::Temp(value) => {
                let value = self.layout.temp_to_slot(value)?;
                T::store_mem0_offset16_ss(ptr, offset16, value)
            }
            Operand::Immediate(value) => {
                let value = <T::Value>::from(value.val());
                let immediate = <T as op::StoreOperator>::into_immediate(value);
                T::store_mem0_offset16_si(ptr, offset16, immediate)
            }
        };
        self.push_instr(store_op, FuelCostsProvider::store)?;
        Ok(true)
    }

    /// Returns the [`MemArg`] linear `memory` index and load/store `offset`.
    ///
    /// # Panics
    ///
    /// If the [`MemArg`] offset is not 32-bit.
    fn decode_memarg(memarg: MemArg) -> Result<(index::Memory, u64), Error> {
        let memory = index::Memory::try_from(memarg.memory)?;
        Ok((memory, memarg.offset))
    }

    /// Returns the effective address `ptr+offset` if it is valid.
    fn effective_address(&self, mem: index::Memory, ptr: TypedVal, offset: u64) -> Option<Address> {
        let memory_type = *self
            .module
            .get_type_of_memory(MemoryIdx::from(u32::from(u16::from(mem))));
        let ptr = match memory_type.is_64() {
            true => u64::from(ptr),
            false => u64::from(u32::from(ptr)),
        };
        let Some(address) = ptr.checked_add(offset) else {
            // Case: address overflows any legal memory index.
            return None;
        };
        if let Some(max) = memory_type.maximum() {
            // The memory's maximum size in bytes.
            let max_size = max << memory_type.page_size_log2();
            if address > max_size {
                // Case: address overflows the memory's maximum size.
                return None;
            }
        }
        if !memory_type.is_64() && address >= 1 << 32 {
            // Case: address overflows the 32-bit memory index.
            return None;
        }
        let Ok(address) = Address::try_from(address) else {
            // Case: address is too big for the system to handle properly.
            return None;
        };
        Some(address)
    }

    /// Translates a Wasm `i64.binop128` instruction from the `wide-arithmetic` proposal.
    fn translate_i64_binop128(
        &mut self,
        make_instr: fn(
            results: FixedSlotSpan<2>,
            lhs_lo: Slot,
            lhs_hi: Slot,
            rhs_lo: Slot,
            rhs_hi: Slot,
        ) -> Op,
        const_eval: fn(lhs_lo: i64, lhs_hi: i64, rhs_lo: i64, rhs_hi: i64) -> (i64, i64),
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (rhs_lo, rhs_hi) = self.stack.pop2();
        let (lhs_lo, lhs_hi) = self.stack.pop2();
        if let (
            Operand::Immediate(lhs_lo),
            Operand::Immediate(lhs_hi),
            Operand::Immediate(rhs_lo),
            Operand::Immediate(rhs_hi),
        ) = (lhs_lo, lhs_hi, rhs_lo, rhs_hi)
        {
            let (result_lo, result_hi) = const_eval(
                lhs_lo.val().into(),
                lhs_hi.val().into(),
                rhs_lo.val().into(),
                rhs_hi.val().into(),
            );
            self.stack.push_immediate(result_lo)?;
            self.stack.push_immediate(result_hi)?;
            return Ok(());
        }
        let rhs_lo = self.copy_if_immediate(rhs_lo)?;
        let rhs_hi = self.copy_if_immediate(rhs_hi)?;
        let lhs_lo = self.copy_if_immediate(lhs_lo)?;
        let lhs_hi = self.copy_if_immediate(lhs_hi)?;
        let result_lo = self.stack.push_temp(ValType::I64)?;
        let result_hi = self.stack.push_temp(ValType::I64)?;
        let result_lo = self.layout.temp_to_slot(result_lo)?;
        let result_hi = self.layout.temp_to_slot(result_hi)?;
        let Ok(results) = <FixedSlotSpan<2>>::new(SlotSpan::new(result_lo)) else {
            return Err(Error::from(TranslationError::AllocatedTooManySlots));
        };
        debug_assert_eq!(results.to_array(), [result_lo, result_hi]);
        self.push_instr(
            make_instr(results, lhs_lo, lhs_hi, rhs_lo, rhs_hi),
            FuelCostsProvider::base,
        )?;
        Ok(())
    }

    /// Translates a Wasm `i64.mul_wide_sx` instruction from the `wide-arithmetic` proposal.
    fn translate_i64_mul_wide_sx(
        &mut self,
        make_instr: fn(results: FixedSlotSpan<2>, lhs: Slot, rhs: Slot) -> Op,
        const_eval: fn(lhs: i64, rhs: i64) -> (i64, i64),
        signed: bool,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (lhs, rhs) = self.stack.pop2();
        let (lhs, rhs) = match (lhs, rhs) {
            (Operand::Immediate(lhs), Operand::Immediate(rhs)) => {
                let (result_lo, result_hi) = const_eval(lhs.val().into(), rhs.val().into());
                self.stack.push_immediate(result_lo)?;
                self.stack.push_immediate(result_hi)?;
                return Ok(());
            }
            (lhs, Operand::Immediate(rhs_imm)) => {
                let rhs_val = rhs_imm.val();
                if self.try_opt_i64_mul_wide_sx(lhs, rhs_val, signed)? {
                    return Ok(());
                }
                let lhs = self.layout.operand_to_slot(lhs)?;
                let rhs = self.copy_if_immediate(rhs)?;
                (lhs, rhs)
            }
            (Operand::Immediate(lhs_imm), rhs) => {
                let lhs_val = lhs_imm.val();
                if self.try_opt_i64_mul_wide_sx(rhs, lhs_val, signed)? {
                    return Ok(());
                }
                let lhs = self.copy_if_immediate(lhs)?;
                let rhs = self.layout.operand_to_slot(rhs)?;
                (lhs, rhs)
            }
            (lhs, rhs) => {
                let lhs = self.layout.operand_to_slot(lhs)?;
                let rhs = self.layout.operand_to_slot(rhs)?;
                (lhs, rhs)
            }
        };
        let result0 = self.stack.push_temp(ValType::I64)?;
        let _result1 = self.stack.push_temp(ValType::I64)?;
        let result0 = self.layout.temp_to_slot(result0)?;
        let Ok(results) = <FixedSlotSpan<2>>::new(SlotSpan::new(result0)) else {
            return Err(Error::from(TranslationError::AllocatedTooManySlots));
        };
        self.push_instr(make_instr(results, lhs, rhs), FuelCostsProvider::base)?;
        Ok(())
    }

    /// Try to optimize a `i64.mul_wide_sx` instruction with one [`Slot`] and one immediate input.
    ///
    /// - Returns `Ok(true)` if the optimiation was applied successfully.
    /// - Returns `Ok(false)` if no optimization was applied.
    fn try_opt_i64_mul_wide_sx(
        &mut self,
        lhs: Operand,
        rhs: TypedVal,
        signed: bool,
    ) -> Result<bool, Error> {
        let rhs = i64::from(rhs);
        if rhs == 0 {
            // Case: `mul(x, 0)` or `mul(0, x)` always evaluates to 0.
            self.stack.push_immediate(0_i64)?; // lo-bits
            self.stack.push_immediate(0_i64)?; // hi-bits
            return Ok(true);
        }
        if rhs == 1 && !signed {
            // Case: `mul(x, 1)` or `mul(1, x)` always evaluates to just `x`.
            // This is only valid if `x` is not a singed (negative) value.
            let result = self.stack.push_operand(lhs)?; // lo-bits
            if matches!(lhs, Operand::Temp(_)) {
                // Case: `lhs` is temporary and thus might need a copy to its new result.
                let consume_fuel_instr = self.stack.consume_fuel_instr();
                let result = self.layout.temp_to_slot(result)?;
                self.encode_copy(result, lhs, consume_fuel_instr)?;
            }
            self.stack.push_immediate(0_i64)?; // hi-bits
            return Ok(true);
        }
        Ok(false)
    }
}
