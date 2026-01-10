use super::FuncTranslator;

mod op;
mod visit;

use crate::{
    Error,
    V128,
    ValType,
    core::{FuelCostsProvider, Typed, TypedVal, simd::IntoLaneIdx},
    engine::translator::{
        func::{Operand, utils::Input},
        utils::{IntoShiftAmount, ToBits, Wrap},
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
        make_instr_ss: fn(result: Slot, value: Slot) -> Op,
        make_instr_si: fn(result: Slot, value: <Wrapped as ToBits>::Out) -> Op,
    ) -> Result<(), Error>
    where
        T: From<TypedVal> + Wrap<Wrapped>,
        Wrapped: ToBits,
    {
        bail_unreachable!(self);
        let value = self.stack.pop();
        let value = self.make_input(value, |_this, value| {
            Ok(Input::Immediate(T::from(value).wrap().to_bits()))
        })?;
        self.push_instr_with_result(
            ValType::V128,
            |result| match value {
                Input::Slot(value) => make_instr_ss(result, value),
                Input::Immediate(value) => make_instr_si(result, value),
            },
            FuelCostsProvider::simd,
        )?;
        Ok(())
    }

    /// Generically translate any of the Wasm `simd` extract lane instructions.
    fn translate_extract_lane<T: IntoLaneIdx, R>(
        &mut self,
        lane: u8,
        make_instr: fn(result: Slot, input: Slot, lane: T::LaneIdx) -> Op,
        const_eval: fn(input: V128, lane: T::LaneIdx) -> R,
    ) -> Result<(), Error>
    where
        R: Into<TypedVal> + Typed,
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
        self.push_instr_with_result(
            <R as Typed>::TY,
            |result| make_instr(result, input, lane),
            FuelCostsProvider::simd,
        )?;
        Ok(())
    }

    /// Generically translate a Wasm SIMD replace lane instruction.
    fn translate_replace_lane<T: op::SimdReplaceLane>(&mut self, lane: u8) -> Result<(), Error>
    where
        T::Item: IntoLaneIdx + From<TypedVal> + Copy,
        T::Immediate: Copy,
    {
        bail_unreachable!(self);
        let Ok(lane) = <<T::Item as IntoLaneIdx>::LaneIdx>::try_from(lane) else {
            panic!("encountered out of bounds lane index: {lane}");
        };
        let (input, value) = self.stack.pop2();
        if let (Operand::Immediate(x), Operand::Immediate(item)) = (input, value) {
            let result = T::const_eval(x.val().into(), lane, item.val().into());
            self.stack.push_immediate(result)?;
            return Ok(());
        }
        let input = self.copy_if_immediate(input)?;
        let value = self.make_input::<T::Immediate>(value, |_this, value| {
            Ok(Input::Immediate(T::into_immediate(T::Item::from(value))))
        })?;
        self.push_instr_with_result(
            <T::Item as Typed>::TY,
            |result| match value {
                Input::Slot(value) => T::replace_lane_sss(result, input, lane, value),
                Input::Immediate(value) => T::replace_lane_ssi(result, input, lane, value),
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
        T: Into<TypedVal> + Typed,
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
        self.push_instr_with_result(
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
        let lhs = self.copy_if_immediate(lhs)?;
        let rhs = self.copy_if_immediate(rhs)?;
        self.push_instr_with_result(
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
        let lhs = self.copy_if_immediate(a)?;
        let rhs = self.copy_if_immediate(b)?;
        let selector = self.copy_if_immediate(c)?;
        self.push_instr_with_result(
            ValType::V128,
            |result| make_instr(result, lhs, rhs, selector),
            FuelCostsProvider::simd,
        )?;
        Ok(())
    }

    /// Generically translate a Wasm SIMD shift instruction.
    fn translate_simd_shift<T>(
        &mut self,
        make_instr_sss: fn(result: Slot, lhs: Slot, rhs: Slot) -> Op,
        make_instr_ssi: fn(result: Slot, lhs: Slot, rhs: <T as IntoShiftAmount>::ShiftAmount) -> Op,
        const_eval: fn(lhs: V128, rhs: u32) -> V128,
    ) -> Result<(), Error>
    where
        T: IntoShiftAmount<ShiftSource: From<TypedVal>>,
    {
        bail_unreachable!(self);
        let (lhs, rhs) = self.stack.pop2();
        if let (Operand::Immediate(lhs), Operand::Immediate(rhs)) = (lhs, rhs) {
            // Case: both inputs are immediates so we can const-eval the result.
            let result = const_eval(lhs.val().into(), rhs.val().into());
            self.stack.push_immediate(result)?;
            return Ok(());
        }
        if let Operand::Immediate(rhs) = rhs {
            let shift_amount = <T::ShiftSource>::from(rhs.val());
            let Some(rhs) = T::into_shift_amount(shift_amount) else {
                // Case: the shift operation is a no-op
                self.stack.push_operand(lhs)?;
                return Ok(());
            };
            let lhs = self.copy_if_immediate(lhs)?;
            self.push_instr_with_result(
                ValType::V128,
                |result| make_instr_ssi(result, lhs, rhs),
                FuelCostsProvider::simd,
            )?;
            return Ok(());
        }
        let lhs = self.copy_if_immediate(lhs)?;
        let rhs = self.copy_if_immediate(rhs)?;
        self.push_instr_with_result(
            ValType::V128,
            |result| make_instr_sss(result, lhs, rhs),
            FuelCostsProvider::simd,
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
        let ptr = self.copy_if_immediate(ptr)?;
        let v128 = self.copy_if_immediate(v128)?;
        if memory.is_default() {
            if let Ok(offset) = Offset16::try_from(offset) {
                self.push_instr_with_result(
                    <T as Typed>::TY,
                    |result| load_lane_mem0_offset16(result, ptr, offset, v128, lane),
                    FuelCostsProvider::load,
                )?;
                return Ok(());
            }
        }
        self.push_instr_with_result(
            <T as Typed>::TY,
            |result| load_lane(result, ptr, offset, memory, v128, lane),
            FuelCostsProvider::load,
        )?;
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    fn translate_v128_store_lane<T: IntoLaneIdx>(
        &mut self,
        memarg: MemArg,
        lane: u8,
        make_instr: fn(ptr: Slot, offset: u64, value: Slot, memory: Memory, lane: T::LaneIdx) -> Op,
        make_instr_mem0_offset16: fn(
            ptr: Slot,
            offset: Offset16,
            value: Slot,
            lane: T::LaneIdx,
        ) -> Op,
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
            Operand::Immediate(v128) => {
                // Case: with `v128` being an immediate value we can extract its
                //       lane value and translate as a more efficient non-SIMD operation.
                return translate_imm(self, memarg, ptr, lane, V128::from(v128.val()));
            }
            Operand::Local(v128) => self.layout.local_to_slot(v128)?,
            Operand::Temp(v128) => self.layout.temp_to_slot(v128)?,
        };
        let (memory, offset) = Self::decode_memarg(memarg)?;
        let ptr = self.copy_if_immediate(ptr)?;
        if memory.is_default() {
            if let Ok(offset16) = Offset16::try_from(offset) {
                self.push_instr(
                    make_instr_mem0_offset16(ptr, offset16, v128, lane),
                    FuelCostsProvider::store,
                )?;
                return Ok(());
            }
        }
        self.push_instr(
            make_instr(ptr, offset, v128, memory, lane),
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
        ptr: Slot,
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
            Op::store128_mem0_offset16_ss(ptr, offset16, value),
            FuelCostsProvider::store,
        )?;
        Ok(true)
    }
}
