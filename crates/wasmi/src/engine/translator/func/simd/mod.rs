use super::FuncTranslator;

mod op;
mod visit;

use crate::{
    Error,
    TrapCode,
    V128,
    ValType,
    core::{
        FuelCostsProvider,
        IntoShiftAmount,
        RawVal,
        ShiftAmount,
        Typed,
        TypedRawVal,
        simd::IntoLaneIdx,
    },
    engine::translator::{
        func::{
            Operand,
            stack::{Location, ResolvedOperand},
        },
        utils::{ToBits, Wrap},
    },
    ir::{
        Offset16,
        Op,
        Slot,
        index::{self, Memory},
    },
};
use wasmparser::MemArg;

impl FuncTranslator {
    /// Generically translate any of the Wasm `simd` splat instructions.
    fn translate_simd_splat<T, Wrapped>(
        &mut self,
        op_sr: fn(result: Slot) -> Op,
        op_ss: fn(result: Slot, value: Slot) -> Op,
        op_si: fn(result: Slot, value: <Wrapped as ToBits>::Out) -> Op,
    ) -> Result<(), Error>
    where
        T: From<RawVal> + Wrap<Wrapped>,
        Wrapped: ToBits,
    {
        bail_unreachable!(self);
        let value = self.stack.pop();
        let value = self.resolve_operand_as::<RawVal>(value)?;
        self.push_instr_with_result_slot(
            ValType::V128,
            |result| match value {
                ResolvedOperand::Reg => op_sr(result),
                ResolvedOperand::Slot(value) => op_ss(result, value),
                ResolvedOperand::Immediate(value) => {
                    let value = T::from(value).wrap().to_bits();
                    op_si(result, value)
                }
            },
            FuelCostsProvider::simd,
        )?;
        Ok(())
    }

    /// Generically translate any of the Wasm `simd` extract lane instructions.
    fn translate_extract_lane<T: IntoLaneIdx, R>(
        &mut self,
        lane: u8,
        make_instr: fn(input: Slot, lane: T::LaneIdx) -> Op,
        const_eval: fn(input: V128, lane: T::LaneIdx) -> R,
    ) -> Result<(), Error>
    where
        R: Into<TypedRawVal> + Typed,
    {
        bail_unreachable!(self);
        let Ok(lane) = <T::LaneIdx>::try_from(lane) else {
            panic!("encountered out of bounds lane index: {lane}")
        };
        let input = self.stack.pop();
        if let Operand::Immediate(input) = input {
            let result = const_eval(input.val().into(), lane);
            self.stack.push_immediate(result)?;
            return Ok(());
        };
        let input = self.layout.operand_to_slot(input)?;
        self.push_instr_with_result_reg(
            <R as Typed>::TY,
            make_instr(input, lane),
            FuelCostsProvider::simd,
        )?;
        Ok(())
    }

    /// Generically translate a Wasm SIMD replace lane instruction.
    fn translate_replace_lane<T: op::SimdReplaceLane>(&mut self, lane: u8) -> Result<(), Error>
    where
        T::Item: IntoLaneIdx + From<RawVal> + Copy,
        T::Immediate: Copy,
    {
        bail_unreachable!(self);
        let Ok(lane) = <<T::Item as IntoLaneIdx>::LaneIdx>::try_from(lane) else {
            panic!("encountered out of bounds lane index: {lane}");
        };
        let (input, value) = self.stack.pop2();
        if let (Operand::Immediate(x), Operand::Immediate(item)) = (input, value) {
            let result = T::const_eval(x.val().into(), lane, item.val().raw().into());
            self.stack.push_immediate(result)?;
            return Ok(());
        }
        let input = self.copy_operand_to_slot(input)?;
        let value = self
            .resolve_operand_as::<RawVal>(value)?
            .map(|value| T::into_immediate(T::Item::from(value)));
        self.push_instr_with_result_slot(
            ValType::V128,
            |result| match value {
                ResolvedOperand::Reg => T::op_ssr(result, input, lane),
                ResolvedOperand::Slot(value) => T::op_sss(result, input, lane, value),
                ResolvedOperand::Immediate(value) => T::op_ssi(result, input, lane, value),
            },
            FuelCostsProvider::simd,
        )?;
        Ok(())
    }

    /// Generically translate a Wasm unary instruction.
    fn translate_simd_unary<T>(
        &mut self,
        make_instr: fn(result: Slot, input: Slot) -> Op,
        const_eval: fn(input: V128) -> T,
    ) -> Result<(), Error>
    where
        T: Into<TypedRawVal> + Typed,
    {
        bail_unreachable!(self);
        let input = self.stack.pop();
        if let Operand::Immediate(input) = input {
            // Case: the input is an immediate so we can const-eval the result.
            let result = const_eval(input.val().into());
            self.stack.push_immediate(result)?;
            return Ok(());
        };
        let input = self.layout.operand_to_slot(input)?;
        self.push_instr_with_result_slot(
            <T as Typed>::TY,
            |result| make_instr(result, input),
            FuelCostsProvider::simd,
        )?;
        Ok(())
    }

    /// Generically translate a Wasm binary instruction.
    fn translate_simd_binary(
        &mut self,
        make_instr: fn(result: Slot, lhs: Slot, rhs: Slot) -> Op,
        const_eval: fn(lhs: V128, rhs: V128) -> V128,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (lhs, rhs) = self.stack.pop2();
        if let (Operand::Immediate(lhs), Operand::Immediate(rhs)) = (lhs, rhs) {
            // Case: both inputs are immediates so we can const-eval the result.
            let result = const_eval(lhs.val().into(), rhs.val().into());
            self.stack.push_immediate(result)?;
            return Ok(());
        }
        let lhs = self.copy_operand_to_slot(lhs)?;
        let rhs = self.copy_operand_to_slot(rhs)?;
        self.push_instr_with_result_slot(
            ValType::V128,
            |result| make_instr(result, lhs, rhs),
            FuelCostsProvider::simd,
        )?;
        Ok(())
    }

    /// Generically translate a Wasm ternary instruction.
    fn translate_simd_ternary(
        &mut self,
        make_instr: fn(result: Slot, a: Slot, b: Slot, c: Slot) -> Op,
        const_eval: fn(lhas: V128, b: V128, c: V128) -> V128,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (a, b, c) = self.stack.pop3();
        if let (Operand::Immediate(lhs), Operand::Immediate(rhs), Operand::Immediate(c)) = (a, b, c)
        {
            // Case: all inputs are immediates so we can const-eval the result.
            let result = const_eval(lhs.val().into(), rhs.val().into(), c.val().into());
            self.stack.push_immediate(result)?;
            return Ok(());
        }
        let lhs = self.copy_operand_to_slot(a)?;
        let rhs = self.copy_operand_to_slot(b)?;
        let selector = self.copy_operand_to_slot(c)?;
        self.push_instr_with_result_slot(
            ValType::V128,
            |result| make_instr(result, lhs, rhs, selector),
            FuelCostsProvider::simd,
        )?;
        Ok(())
    }

    /// Generically translate a Wasm SIMD shift instruction.
    fn translate_simd_shift<T>(
        &mut self,
        op_ssr: fn(result: Slot, lhs: Slot) -> Op,
        op_sss: fn(result: Slot, lhs: Slot, rhs: Slot) -> Op,
        op_ssi: fn(result: Slot, lhs: Slot, rhs: ShiftAmount) -> Op,
        const_eval: fn(lhs: V128, rhs: u32) -> V128,
    ) -> Result<(), Error>
    where
        T: IntoShiftAmount<ShiftSource: From<RawVal>>,
    {
        bail_unreachable!(self);
        let (lhs, rhs) = self.stack.pop2();
        if let (Operand::Immediate(lhs), Operand::Immediate(rhs)) = (lhs, rhs) {
            // Case: both inputs are immediates so we can const-eval the result.
            let result = const_eval(lhs.val().into(), rhs.val().into());
            self.stack.push_immediate(result)?;
            return Ok(());
        }
        let Some(rhs) = self
            .resolve_operand_as::<T::ShiftSource>(lhs)?
            .map(T::into_shift_amount)
            .transpose()
        else {
            // Case: the shift operation is a no-op
            self.stack.push_operand(lhs)?;
            return Ok(());
        };
        let lhs = self.copy_operand_to_slot(lhs)?;
        self.push_instr_with_result_slot(
            ValType::V128,
            |result| match rhs {
                ResolvedOperand::Reg => op_ssr(result, lhs),
                ResolvedOperand::Slot(rhs) => op_sss(result, lhs, rhs),
                ResolvedOperand::Immediate(rhs) => op_ssi(result, lhs, rhs),
            },
            FuelCostsProvider::simd,
        )?;
        Ok(())
    }

    fn translate_simd_load<T: op::SimdLoadOp>(&mut self, memarg: MemArg) -> Result<(), Error> {
        bail_unreachable!(self);
        let (memory, offset) = Self::decode_memarg(memarg)?;
        let ptr = self.stack.pop();
        self.copy_immediate_to_slot(ptr)?;
        let ptr = self.resolve_operand_as_index(ptr, memory)?;
        'opt: {
            // Try to encode an optimized load operator if possible, otherwise fallback.
            if !memory.is_default() {
                break 'opt;
            }
            let offset = match Offset16::try_from(offset) {
                Ok(offset) => offset,
                Err(_) => break 'opt,
            };
            self.push_instr_with_result_slot(
                ValType::V128,
                |result| match ptr {
                    ResolvedOperand::Reg => T::op_sr_mem0_offset16(result, offset),
                    ResolvedOperand::Slot(ptr) => T::op_ss_mem0_offset16(result, ptr, offset),
                    ResolvedOperand::Immediate(_) => unreachable!(),
                },
                FuelCostsProvider::load,
            )?;
            return Ok(());
        }
        // We need to encode a non-optimized fallback load operator.
        let Some(ptr) = ptr.filter_map(|ptr| self.effective_address(memory, ptr, offset)) else {
            return self.translate_trap(TrapCode::MemoryOutOfBounds);
        };
        self.push_instr_with_result_slot(
            ValType::V128,
            |result| match ptr {
                ResolvedOperand::Reg => T::op_sr(result, offset, memory),
                ResolvedOperand::Slot(ptr) => T::op_ss(result, ptr, offset, memory),
                ResolvedOperand::Immediate(_) => unreachable!(),
            },
            FuelCostsProvider::load,
        )?;
        Ok(())
    }

    fn translate_v128_load_lane<T: IntoLaneIdx + Typed>(
        &mut self,
        memarg: MemArg,
        lane: u8,
        load_lane: fn(
            result: Slot,
            ptr: Slot,
            offset: u64,
            memory: index::Memory,
            v128: Slot,
            lane: T::LaneIdx,
        ) -> Op,
        load_lane_mem0_offset16: fn(
            result: Slot,
            ptr: Slot,
            offset: Offset16,
            v128: Slot,
            lane: T::LaneIdx,
        ) -> Op,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (memory, offset) = Self::decode_memarg(memarg)?;
        let Ok(lane) = <T::LaneIdx>::try_from(lane) else {
            panic!("encountered out of bounds lane: {lane}");
        };
        let (ptr, v128) = self.stack.pop2();
        let ptr = self.copy_operand_to_slot(ptr)?;
        let v128 = self.copy_operand_to_slot(v128)?;
        if memory.is_default() {
            if let Ok(offset) = Offset16::try_from(offset) {
                self.push_instr_with_result_slot(
                    ValType::V128,
                    |result| load_lane_mem0_offset16(result, ptr, offset, v128, lane),
                    FuelCostsProvider::load,
                )?;
                return Ok(());
            }
        }
        self.push_instr_with_result_slot(
            ValType::V128,
            |result| load_lane(result, ptr, offset, memory, v128, lane),
            FuelCostsProvider::load,
        )?;
        Ok(())
    }

    #[expect(clippy::type_complexity, clippy::too_many_arguments)]
    fn translate_v128_store_lane<T: IntoLaneIdx>(
        &mut self,
        memarg: MemArg,
        lane: u8,
        op_rs: fn(offset: u64, value: Slot, memory: Memory, lane: T::LaneIdx) -> Op,
        op_ss: fn(ptr: Slot, offset: u64, value: Slot, memory: Memory, lane: T::LaneIdx) -> Op,
        op_rs_mem0_offset16: fn(offset: Offset16, value: Slot, lane: T::LaneIdx) -> Op,
        op_ss_mem0_offset16: fn(ptr: Slot, offset: Offset16, value: Slot, lane: T::LaneIdx) -> Op,
        translate_imm: fn(
            &mut Self,
            memarg: MemArg,
            ptr: Operand,
            lane: T::LaneIdx,
            value: V128,
        ) -> Result<(), Error>,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let Ok(lane) = <T::LaneIdx>::try_from(lane) else {
            panic!("encountered out of bounds lane index: {lane}");
        };
        let (ptr, v128) = self.stack.pop2();
        let v128 = match v128 {
            Operand::Reg(_v128) => {
                // Note: `v128` typed values may not occupy register operands for now.
                unreachable!()
            }
            Operand::Immediate(v128) => {
                // Case: with `v128` being an immediate value we can extract its
                //       lane value and translate as a more efficient non-SIMD operation.
                return translate_imm(self, memarg, ptr, lane, V128::from(v128.val()));
            }
            Operand::Local(v128) => self.layout.local_to_slot(v128)?,
            Operand::Temp(v128) => v128.temp_slots().head(),
        };
        let (memory, offset) = Self::decode_memarg(memarg)?;
        let ptr = self.copy_immediate_to_slot(ptr)?;
        if memory.is_default() {
            if let Ok(offset16) = Offset16::try_from(offset) {
                self.push_instr(
                    match ptr {
                        Location::Reg => op_rs_mem0_offset16(offset16, v128, lane),
                        Location::Slot(ptr) => op_ss_mem0_offset16(ptr, offset16, v128, lane),
                    },
                    FuelCostsProvider::store,
                )?;
                return Ok(());
            }
        }
        self.push_instr(
            match ptr {
                Location::Reg => op_rs(offset, v128, memory, lane),
                Location::Slot(ptr) => op_ss(ptr, offset, v128, memory, lane),
            },
            FuelCostsProvider::store,
        )?;
        Ok(())
    }

    /// Encodes a Wasmi `store128` operator with `(mem 0)` and 16-bit encodable `offset` to Wasmi bytecode.
    ///
    /// # Note
    ///
    /// - Returns `Ok(true)` if encoding was successfull.
    /// - Returns `Ok(false)` if encoding was unsuccessful.
    /// - Returns `Err(_)` if an error occurred.
    fn translate_store128_mem0_offset16(
        &mut self,
        ptr: Location,
        offset: u64,
        memory: index::Memory,
        value: Slot,
    ) -> Result<bool, Error> {
        if !memory.is_default() {
            return Ok(false);
        }
        let Ok(offset16) = Offset16::try_from(offset) else {
            return Ok(false);
        };
        self.push_instr(
            match ptr {
                Location::Slot(ptr) => Op::v128_store_mem0_offset16_ss(ptr, offset16, value),
                Location::Reg => Op::v128_store_mem0_offset16_rs(offset16, value),
            },
            FuelCostsProvider::store,
        )?;
        Ok(true)
    }
}
