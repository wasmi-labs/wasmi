use super::FuncTranslator;

mod op;
mod visit;

use crate::{
    core::{simd::IntoLaneIdx, FuelCostsProvider, Typed, TypedVal},
    engine::translator::{
        func2::{utils::Input, Operand},
        utils::{Instr, Wrap},
    },
    ir::{
        index::{self, Memory},
        Address32,
        Instruction,
        IntoShiftAmount,
        Offset64,
        Offset64Lo,
        Offset8,
        Reg,
    },
    Error,
    TrapCode,
    ValType,
    V128,
};
use wasmparser::MemArg;

impl FuncTranslator {
    /// Generically translate any of the Wasm `simd` splat instructions.
    fn translate_simd_splat<T, Wrapped>(
        &mut self,
        make_instr: fn(result: Reg, value: Reg) -> Instruction,
        const_eval: fn(Wrapped) -> V128,
    ) -> Result<(), Error>
    where
        T: From<TypedVal> + Wrap<Wrapped>,
    {
        bail_unreachable!(self);
        let value = self.stack.pop();
        if let Operand::Immediate(value) = value {
            let value = T::from(value.val()).wrap();
            let result = const_eval(value);
            self.stack.push_immediate(result)?;
            return Ok(());
        };
        let value = self.layout.operand_to_reg(value)?;
        self.push_instr_with_result(
            ValType::V128,
            |result| make_instr(result, value),
            FuelCostsProvider::simd,
        )?;
        Ok(())
    }

    /// Generically translate any of the Wasm `simd` extract lane instructions.
    fn translate_extract_lane<T: IntoLaneIdx, R>(
        &mut self,
        lane: u8,
        make_instr: fn(result: Reg, input: Reg, lane: T::LaneIdx) -> Instruction,
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
        let input = self.layout.operand_to_reg(input)?;
        self.push_instr_with_result(
            <R as Typed>::TY,
            |result| make_instr(result, input, lane),
            FuelCostsProvider::simd,
        )?;
        Ok(())
    }

    /// Generically translate a Wasm SIMD replace lane instruction.
    #[allow(clippy::type_complexity)]
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
        let input = self.layout.operand_to_reg(input)?;
        let value =
            self.make_input::<T::Immediate>(value, |this, value| {
                match T::value_to_imm(T::Item::from(value)) {
                    Some(imm) => Ok(Input::Immediate(imm)),
                    None => {
                        let imm = this.layout.const_to_reg(value)?;
                        Ok(Input::Reg(imm))
                    }
                }
            })?;
        let param = match value {
            Input::Reg(value) => Some(Instruction::register(value)),
            Input::Immediate(value) => T::replace_lane_imm_param(value),
        };
        self.push_instr_with_result(
            <T::Item as Typed>::TY,
            |result| match value {
                Input::Reg(_) => T::replace_lane(result, input, lane),
                Input::Immediate(value) => T::replace_lane_imm(result, input, lane, value),
            },
            FuelCostsProvider::simd,
        )?;
        if let Some(param) = param {
            self.push_param(param)?;
        }
        Ok(())
    }

    /// Generically translate a Wasm unary instruction.
    fn translate_simd_unary<T>(
        &mut self,
        make_instr: fn(result: Reg, input: Reg) -> Instruction,
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
        let input = self.layout.operand_to_reg(input)?;
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
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
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
        let lhs = self.layout.operand_to_reg(lhs)?;
        let rhs = self.layout.operand_to_reg(rhs)?;
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
        make_instr: fn(result: Reg, a: Reg, b: Reg) -> Instruction,
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
        let lhs = self.layout.operand_to_reg(a)?;
        let rhs = self.layout.operand_to_reg(b)?;
        let selector = self.layout.operand_to_reg(c)?;
        self.push_instr_with_result(
            ValType::V128,
            |result| make_instr(result, lhs, rhs),
            FuelCostsProvider::simd,
        )?;
        self.push_param(Instruction::register(selector))?;
        Ok(())
    }

    /// Generically translate a Wasm SIMD shift instruction.
    fn translate_simd_shift<T>(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        make_instr_imm: fn(
            result: Reg,
            lhs: Reg,
            rhs: <T as IntoShiftAmount>::Output,
        ) -> Instruction,
        const_eval: fn(lhs: V128, rhs: u32) -> V128,
    ) -> Result<(), Error>
    where
        T: IntoShiftAmount<Input: From<TypedVal>>,
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
            let Some(rhs) = T::into_shift_amount(rhs.val().into()) else {
                // Case: the shift operation is a no-op
                self.stack.push_operand(lhs)?;
                return Ok(());
            };
            let lhs = self.layout.operand_to_reg(lhs)?;
            self.push_instr_with_result(
                ValType::V128,
                |result| make_instr_imm(result, lhs, rhs),
                FuelCostsProvider::simd,
            )?;
            return Ok(());
        }
        let lhs = self.layout.operand_to_reg(lhs)?;
        let rhs = self.layout.operand_to_reg(rhs)?;
        self.push_instr_with_result(
            ValType::V128,
            |result| make_instr(result, lhs, rhs),
            FuelCostsProvider::simd,
        )?;
        Ok(())
    }

    fn translate_v128_load_lane<T: IntoLaneIdx + Typed>(
        &mut self,
        memarg: MemArg,
        lane: u8,
        make_instr: fn(result: Reg, offset_lo: Offset64Lo) -> Instruction,
        make_instr_at: fn(result: Reg, address: Address32) -> Instruction,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (memory, offset) = Self::decode_memarg(memarg);
        let Ok(lane) = <T::LaneIdx>::try_from(lane) else {
            panic!("encountered out of bounds lane: {lane}");
        };
        let (ptr, x) = self.stack.pop2();
        let x = self.layout.operand_to_reg(x)?;
        let (ptr, offset) = match ptr {
            Operand::Immediate(ptr) => {
                let Some(address) = self.effective_address(memory, ptr.val(), offset) else {
                    return self.translate_trap(TrapCode::MemoryOutOfBounds);
                };
                if let Ok(address) = Address32::try_from(address) {
                    return self.translate_v128_load_lane_at::<T, _>(
                        memory,
                        x,
                        lane,
                        address,
                        make_instr_at,
                    );
                }
                let zero_ptr = self.layout.const_to_reg(0_u64)?;
                (zero_ptr, u64::from(address))
            }
            ptr => {
                let ptr = self.layout.operand_to_reg(ptr)?;
                (ptr, offset)
            }
        };
        let (offset_hi, offset_lo) = Offset64::split(offset);
        self.push_instr_with_result(
            <T as Typed>::TY,
            |result| make_instr(result, offset_lo),
            FuelCostsProvider::load,
        )?;
        self.push_param(Instruction::register_and_offset_hi(ptr, offset_hi))?;
        self.push_param(Instruction::register_and_lane(x, lane))?;
        if !memory.is_default() {
            self.push_param(Instruction::memory_index(memory))?;
        }
        Ok(())
    }

    fn translate_v128_load_lane_at<T: Typed, LaneType>(
        &mut self,
        memory: Memory,
        x: Reg,
        lane: LaneType,
        address: Address32,
        make_instr_at: fn(result: Reg, address: Address32) -> Instruction,
    ) -> Result<(), Error>
    where
        LaneType: Into<u8>,
    {
        self.push_instr_with_result(
            <T as Typed>::TY,
            |result| make_instr_at(result, address),
            FuelCostsProvider::load,
        )?;
        self.push_param(Instruction::register_and_lane(x, lane))?;
        if !memory.is_default() {
            self.push_param(Instruction::memory_index(memory))?;
        }
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    fn translate_v128_store_lane<T: IntoLaneIdx>(
        &mut self,
        memarg: MemArg,
        lane: u8,
        make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
        make_instr_offset8: fn(
            ptr: Reg,
            value: Reg,
            offset: Offset8,
            lane: T::LaneIdx,
        ) -> Instruction,
        make_instr_at: fn(value: Reg, address: Address32) -> Instruction,
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
            v128 => self.layout.operand_to_reg(v128)?,
        };
        let (memory, offset) = Self::decode_memarg(memarg);
        let (ptr, offset) = match ptr {
            Operand::Immediate(ptr) => {
                let Some(address) = self.effective_address(memory, ptr.val(), offset) else {
                    return self.translate_trap(TrapCode::MemoryOutOfBounds);
                };
                if let Ok(address) = Address32::try_from(address) {
                    return self.translate_v128_store_lane_at::<T>(
                        memory,
                        address,
                        v128,
                        lane,
                        make_instr_at,
                    );
                }
                // Case: we cannot use specialized encoding and thus have to fall back
                //       to the general case where `ptr` is zero and `offset` stores the
                //       `ptr+offset` address value.
                let zero_ptr = self.layout.const_to_reg(0_u64)?;
                (zero_ptr, u64::from(address))
            }
            ptr => {
                let ptr = self.layout.operand_to_reg(ptr)?;
                (ptr, offset)
            }
        };
        if let Ok(Some(_)) =
            self.translate_v128_store_lane_mem0(memory, ptr, offset, v128, lane, make_instr_offset8)
        {
            return Ok(());
        }
        let (offset_hi, offset_lo) = Offset64::split(offset);
        let instr = make_instr(ptr, offset_lo);
        let param = Instruction::register_and_offset_hi(v128, offset_hi);
        let param2 = Instruction::lane_and_memory_index(lane, memory);
        self.push_instr(instr, FuelCostsProvider::store)?;
        self.push_param(param)?;
        self.push_param(param2)?;
        Ok(())
    }

    fn translate_v128_store_lane_at<T: IntoLaneIdx>(
        &mut self,
        memory: index::Memory,
        address: Address32,
        value: Reg,
        lane: T::LaneIdx,
        make_instr_at: fn(value: Reg, address: Address32) -> Instruction,
    ) -> Result<(), Error> {
        self.push_instr(make_instr_at(value, address), FuelCostsProvider::store)?;
        self.push_param(Instruction::lane_and_memory_index(lane, memory))?;
        Ok(())
    }

    fn translate_v128_store_lane_mem0<LaneType>(
        &mut self,
        memory: Memory,
        ptr: Reg,
        offset: u64,
        value: Reg,
        lane: LaneType,
        make_instr_offset8: fn(Reg, Reg, Offset8, LaneType) -> Instruction,
    ) -> Result<Option<Instr>, Error> {
        if !memory.is_default() {
            return Ok(None);
        }
        let Ok(offset8) = Offset8::try_from(offset) else {
            return Ok(None);
        };
        let instr = self.push_instr(
            make_instr_offset8(ptr, value, offset8, lane),
            FuelCostsProvider::store,
        )?;
        Ok(Some(instr))
    }
}
