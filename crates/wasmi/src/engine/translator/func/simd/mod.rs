mod visit;

use super::{FuncTranslator, Instr, TypedProvider};
use crate::{
    core::{simd::IntoLaneIdx, FuelCostsProvider, TrapCode, TypedVal, V128},
    engine::translator::{func::Provider, utils::Wrap},
    ir::{
        index,
        index::Memory,
        Address32,
        AnyConst16,
        Instruction,
        IntoShiftAmount,
        Offset16,
        Offset64,
        Offset64Lo,
        Offset8,
        Reg,
    },
    Error,
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
        let value = match value {
            Provider::Register(value) => value,
            Provider::Const(value) => {
                let value = T::from(value).wrap();
                let result = const_eval(value);
                self.stack.push_const(result);
                return Ok(());
            }
        };
        let result = self.stack.push_dynamic()?;
        self.push_fueled_instr(make_instr(result, value), FuelCostsProvider::simd)?;
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
        R: Into<TypedVal>,
    {
        bail_unreachable!(self);
        let Ok(lane) = <T::LaneIdx>::try_from(lane) else {
            panic!("encountered out of bounds lane index: {lane}")
        };
        let input = self.stack.pop();
        let input = match input {
            Provider::Register(input) => input,
            Provider::Const(input) => {
                let result = const_eval(input.into(), lane);
                self.stack.push_const(result);
                return Ok(());
            }
        };
        let result = self.stack.push_dynamic()?;
        self.push_fueled_instr(make_instr(result, input, lane), FuelCostsProvider::simd)?;
        Ok(())
    }

    /// Generically translate a Wasm unary instruction.
    fn translate_simd_unary<T>(
        &mut self,
        make_instr: fn(result: Reg, input: Reg) -> Instruction,
        const_eval: fn(input: V128) -> T,
    ) -> Result<(), Error>
    where
        T: Into<TypedVal>,
    {
        bail_unreachable!(self);
        let input = self.stack.pop();
        let input = match input {
            Provider::Register(input) => input,
            Provider::Const(input) => {
                // Case: the input is an immediate so we can const-eval the result.
                let result = const_eval(input.into());
                self.stack.push_const(result);
                return Ok(());
            }
        };
        let result = self.stack.push_dynamic()?;
        self.push_fueled_instr(make_instr(result, input), FuelCostsProvider::simd)?;
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
        if let (Provider::Const(lhs), Provider::Const(rhs)) = (lhs, rhs) {
            // Case: both inputs are immediates so we can const-eval the result.
            let result = const_eval(lhs.into(), rhs.into());
            self.stack.push_const(result);
            return Ok(());
        }
        let result = self.stack.push_dynamic()?;
        let lhs = self.stack.provider2reg(&lhs)?;
        let rhs = self.stack.provider2reg(&rhs)?;
        self.push_fueled_instr(make_instr(result, lhs, rhs), FuelCostsProvider::simd)?;
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
        if let (Provider::Const(lhs), Provider::Const(rhs), Provider::Const(c)) = (a, b, c) {
            // Case: all inputs are immediates so we can const-eval the result.
            let result = const_eval(lhs.into(), rhs.into(), c.into());
            self.stack.push_const(result);
            return Ok(());
        }
        let result = self.stack.push_dynamic()?;
        let lhs = self.stack.provider2reg(&a)?;
        let rhs = self.stack.provider2reg(&b)?;
        let selector = self.stack.provider2reg(&c)?;
        self.push_fueled_instr(make_instr(result, lhs, rhs), FuelCostsProvider::simd)?;
        self.append_instr(Instruction::register(selector))?;
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
        if let (Provider::Const(lhs), Provider::Const(rhs)) = (lhs, rhs) {
            // Case: both inputs are immediates so we can const-eval the result.
            let result = const_eval(lhs.into(), rhs.into());
            self.stack.push_const(result);
            return Ok(());
        }
        let lhs = self.stack.provider2reg(&lhs)?;
        let rhs = match rhs {
            Provider::Register(rhs) => rhs,
            Provider::Const(rhs) => {
                let Some(rhs) = T::into_shift_amount(rhs.into()) else {
                    // Case: the shift operation is a no-op
                    self.stack.push_register(lhs)?;
                    return Ok(());
                };
                let result = self.stack.push_dynamic()?;
                self.push_fueled_instr(make_instr_imm(result, lhs, rhs), FuelCostsProvider::simd)?;
                return Ok(());
            }
        };
        let result = self.stack.push_dynamic()?;
        self.push_fueled_instr(make_instr(result, lhs, rhs), FuelCostsProvider::simd)?;
        Ok(())
    }

    /// Generically translate a Wasm SIMD replace lane instruction.
    #[allow(clippy::type_complexity)]
    fn translate_replace_lane<T>(
        &mut self,
        lane: u8,
        const_eval: fn(input: V128, lane: T::LaneIdx, value: T) -> V128,
        make_instr: fn(result: Reg, input: Reg, lane: T::LaneIdx) -> Instruction,
        make_instr_imm: fn(
            this: &mut Self,
            result: Reg,
            input: Reg,
            lane: T::LaneIdx,
            value: T,
        ) -> Result<(Instruction, Option<Instruction>), Error>,
    ) -> Result<(), Error>
    where
        T: IntoLaneIdx + From<TypedVal>,
    {
        bail_unreachable!(self);
        let Ok(lane) = <T::LaneIdx>::try_from(lane) else {
            panic!("encountered out of bounds lane index: {lane}");
        };
        let (input, value) = self.stack.pop2();
        if let (Provider::Const(x), Provider::Const(item)) = (input, value) {
            let result = const_eval(x.into(), lane, item.into());
            self.stack.push_const(result);
            return Ok(());
        }
        let input = self.stack.provider2reg(&input)?;
        let result = self.stack.push_dynamic()?;
        let (instr, param) = match value {
            Provider::Register(value) => (
                make_instr(result, input, lane),
                Some(Instruction::register(value)),
            ),
            Provider::Const(value) => make_instr_imm(self, result, input, lane, value.into())?,
        };
        self.push_fueled_instr(instr, FuelCostsProvider::simd)?;
        if let Some(param) = param {
            self.append_instr(param)?;
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
            ptr: TypedProvider,
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
            Provider::Register(v128) => v128,
            Provider::Const(v128) => {
                // Case: with `v128` being an immediate value we can extract its
                //       lane value and translate as a more efficient non-SIMD operation.
                return translate_imm(self, memarg, ptr, lane, V128::from(v128));
            }
        };
        let (memory, offset) = Self::decode_memarg(memarg);
        let (ptr, offset) = match ptr {
            Provider::Register(ptr) => (ptr, offset),
            Provider::Const(ptr) => {
                let Some(address) = self.effective_address(memory, ptr, offset) else {
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
                let zero_ptr = self.stack.alloc_const(0_u64)?;
                (zero_ptr, u64::from(address))
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
        self.push_fueled_instr(instr, FuelCostsProvider::store)?;
        self.append_instr(param)?;
        self.append_instr(param2)?;
        Ok(())
    }

    fn translate_v128_store_lane_imm<Src, Wrapped, Field>(
        &mut self,
        memarg: MemArg,
        ptr: TypedProvider,
        src: Src,
        make_instr_imm: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
        make_instr_offset16_imm: fn(ptr: Reg, offset: Offset16, value: Field) -> Instruction,
        make_instr_at_imm: fn(value: Field, address: Address32) -> Instruction,
    ) -> Result<(), Error>
    where
        Src: Copy + Wrap<Wrapped> + From<TypedVal> + Into<TypedVal>,
        Field: TryFrom<Wrapped> + Into<AnyConst16>,
    {
        self.translate_istore_wrap_impl::<Src, Wrapped, Field>(
            memarg,
            ptr,
            Provider::Const(src.into()),
            |_, _| unreachable!(),
            make_instr_imm,
            |_, _, _| unreachable!(),
            make_instr_offset16_imm,
            |_, _| unreachable!(),
            make_instr_at_imm,
        )
    }

    fn translate_v128_store_lane_at<T: IntoLaneIdx>(
        &mut self,
        memory: index::Memory,
        address: Address32,
        value: Reg,
        lane: T::LaneIdx,
        make_instr_at: fn(value: Reg, address: Address32) -> Instruction,
    ) -> Result<(), Error> {
        self.push_fueled_instr(make_instr_at(value, address), FuelCostsProvider::store)?;
        self.append_instr(Instruction::lane_and_memory_index(lane, memory))?;
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
        let instr = self.push_fueled_instr(
            make_instr_offset8(ptr, value, offset8, lane),
            FuelCostsProvider::store,
        )?;
        Ok(Some(instr))
    }

    fn translate_v128_load_lane<T: IntoLaneIdx>(
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
        let x = self.stack.provider2reg(&x)?;
        let (ptr, offset) = match ptr {
            Provider::Register(ptr) => (ptr, offset),
            Provider::Const(ptr) => {
                let Some(address) = self.effective_address(memory, ptr, offset) else {
                    return self.translate_trap(TrapCode::MemoryOutOfBounds);
                };
                if let Ok(address) = Address32::try_from(address) {
                    return self.translate_v128_load_lane_at(
                        memory,
                        x,
                        lane,
                        address,
                        make_instr_at,
                    );
                }
                let zero_ptr = self.stack.alloc_const(0_u64)?;
                (zero_ptr, u64::from(address))
            }
        };
        let (offset_hi, offset_lo) = Offset64::split(offset);
        let result = self.stack.push_dynamic()?;
        self.push_fueled_instr(make_instr(result, offset_lo), FuelCostsProvider::load)?;
        self.append_instr(Instruction::register_and_offset_hi(ptr, offset_hi))?;
        self.append_instr(Instruction::register_and_lane(x, lane))?;
        if !memory.is_default() {
            self.append_instr(Instruction::memory_index(memory))?;
        }
        Ok(())
    }

    fn translate_v128_load_lane_at<LaneType>(
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
        let result = self.stack.push_dynamic()?;
        let instr = make_instr_at(result, address);
        let param = Instruction::register_and_lane(x, lane);
        self.push_fueled_instr(instr, FuelCostsProvider::load)?;
        self.append_instr(param)?;
        if !memory.is_default() {
            self.append_instr(Instruction::memory_index(memory))?;
        }
        Ok(())
    }
}
