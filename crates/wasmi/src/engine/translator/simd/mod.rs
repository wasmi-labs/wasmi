mod visit;

use super::{utils::Wrap, FuncTranslator, Instr, TypedProvider};
use crate::{
    core::{simd, TrapCode, TypedVal, V128},
    engine::{translator::Provider, FuelCosts},
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

trait IntoLane {
    type LaneType: TryFrom<u8> + Into<u8>;
}

macro_rules! impl_into_lane_for {
    ( $( ($ty:ty => $lane_ty:ty) ),* $(,)? ) => {
        $(
            impl IntoLane for $ty {
                type LaneType = $lane_ty;
            }
        )*
    };
}
impl_into_lane_for! {
    (i8 => simd::ImmLaneIdx16),
    (u8 => simd::ImmLaneIdx16),
    (i16 => simd::ImmLaneIdx8),
    (u16 => simd::ImmLaneIdx8),
    (i32 => simd::ImmLaneIdx4),
    (u32 => simd::ImmLaneIdx4),
    (f32 => simd::ImmLaneIdx4),
    (i64 => simd::ImmLaneIdx2),
    (u64 => simd::ImmLaneIdx2),
    (f64 => simd::ImmLaneIdx2),
}

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
        let value = self.alloc.stack.pop();
        let value = match value {
            Provider::Register(value) => value,
            Provider::Const(value) => {
                let value = T::from(value).wrap();
                let result = const_eval(value);
                self.alloc.stack.push_const(result);
                return Ok(());
            }
        };
        let result = self.alloc.stack.push_dynamic()?;
        self.push_fueled_instr(make_instr(result, value), FuelCosts::base)?;
        Ok(())
    }

    /// Generically translate any of the Wasm `simd` extract lane instructions.
    fn translate_extract_lane<T: IntoLane, R>(
        &mut self,
        lane: u8,
        make_instr: fn(result: Reg, input: Reg, lane: T::LaneType) -> Instruction,
        const_eval: fn(input: V128, lane: T::LaneType) -> R,
    ) -> Result<(), Error>
    where
        R: Into<TypedVal>,
    {
        bail_unreachable!(self);
        let Ok(lane) = <T::LaneType>::try_from(lane) else {
            panic!("encountered out of bounds lane index: {lane}")
        };
        let input = self.alloc.stack.pop();
        let input = match input {
            Provider::Register(input) => input,
            Provider::Const(input) => {
                let result = const_eval(input.into(), lane);
                self.alloc.stack.push_const(result);
                return Ok(());
            }
        };
        let result = self.alloc.stack.push_dynamic()?;
        self.push_fueled_instr(make_instr(result, input, lane), FuelCosts::base)?;
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
        let input = self.alloc.stack.pop();
        let input = match input {
            Provider::Register(input) => input,
            Provider::Const(input) => {
                // Case: the input is an immediate so we can const-eval the result.
                let result = const_eval(input.into());
                self.alloc.stack.push_const(result);
                return Ok(());
            }
        };
        let result = self.alloc.stack.push_dynamic()?;
        self.push_fueled_instr(make_instr(result, input), FuelCosts::base)?;
        Ok(())
    }

    /// Generically translate a Wasm binary instruction.
    fn translate_simd_binary(
        &mut self,
        make_instr: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        const_eval: fn(lhs: V128, rhs: V128) -> V128,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let (lhs, rhs) = self.alloc.stack.pop2();
        if let (Provider::Const(lhs), Provider::Const(rhs)) = (lhs, rhs) {
            // Case: both inputs are immediates so we can const-eval the result.
            let result = const_eval(lhs.into(), rhs.into());
            self.alloc.stack.push_const(result);
            return Ok(());
        }
        let result = self.alloc.stack.push_dynamic()?;
        let lhs = self.alloc.stack.provider2reg(&lhs)?;
        let rhs = self.alloc.stack.provider2reg(&rhs)?;
        self.push_fueled_instr(make_instr(result, lhs, rhs), FuelCosts::base)?;
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
        let (lhs, rhs) = self.alloc.stack.pop2();
        if let (Provider::Const(lhs), Provider::Const(rhs)) = (lhs, rhs) {
            // Case: both inputs are immediates so we can const-eval the result.
            let result = const_eval(lhs.into(), rhs.into());
            self.alloc.stack.push_const(result);
            return Ok(());
        }
        let lhs = self.alloc.stack.provider2reg(&lhs)?;
        let result = self.alloc.stack.push_dynamic()?;
        let instr = match rhs {
            Provider::Register(rhs) => make_instr(result, lhs, rhs),
            Provider::Const(rhs) => {
                let Some(rhs) = T::into_shift_amount(rhs.into()) else {
                    // Case: the shift operation is a no-op
                    self.alloc.stack.push_register(lhs)?;
                    return Ok(());
                };
                make_instr_imm(result, lhs, rhs)
            }
        };
        self.push_fueled_instr(instr, FuelCosts::base)?;
        Ok(())
    }

    /// Generically translate a Wasm SIMD replace lane instruction.
    #[allow(clippy::type_complexity)]
    fn translate_replace_lane<T>(
        &mut self,
        lane: u8,
        const_eval: fn(input: V128, lane: T::LaneType, value: T) -> V128,
        make_instr: fn(result: Reg, input: Reg, lane: T::LaneType) -> Instruction,
        make_instr_imm: fn(
            this: &mut Self,
            result: Reg,
            input: Reg,
            lane: T::LaneType,
            value: T,
        ) -> Result<(Instruction, Option<Instruction>), Error>,
    ) -> Result<(), Error>
    where
        T: IntoLane + From<TypedVal>,
    {
        bail_unreachable!(self);
        let Ok(lane) = <T::LaneType>::try_from(lane) else {
            panic!("encountered out of bounds lane index: {lane}");
        };
        let (input, value) = self.alloc.stack.pop2();
        if let (Provider::Const(x), Provider::Const(item)) = (input, value) {
            let result = const_eval(x.into(), lane, item.into());
            self.alloc.stack.push_const(result);
            return Ok(());
        }
        let input = self.alloc.stack.provider2reg(&input)?;
        let result = self.alloc.stack.push_dynamic()?;
        let (instr, param) = match value {
            Provider::Register(value) => (
                make_instr(result, input, lane),
                Some(Instruction::register(value)),
            ),
            Provider::Const(value) => make_instr_imm(self, result, input, lane, value.into())?,
        };
        self.push_fueled_instr(instr, FuelCosts::base)?;
        if let Some(param) = param {
            self.alloc.instr_encoder.append_instr(param)?;
        }
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    fn translate_v128_store_lane<T: IntoLane>(
        &mut self,
        memarg: MemArg,
        lane: u8,
        make_instr: fn(ptr: Reg, offset_lo: Offset64Lo) -> Instruction,
        make_instr_offset8: fn(
            ptr: Reg,
            value: Reg,
            offset: Offset8,
            lane: T::LaneType,
        ) -> Instruction,
        make_instr_at: fn(value: Reg, address: Address32) -> Instruction,
        translate_imm: fn(
            &mut Self,
            memarg: MemArg,
            ptr: TypedProvider,
            lane: T::LaneType,
            value: V128,
        ) -> Result<(), Error>,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let Ok(lane) = <T::LaneType>::try_from(lane) else {
            panic!("encountered out of bounds lane index: {lane}");
        };
        let (ptr, v128) = self.alloc.stack.pop2();
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
                let zero_ptr = self.alloc.stack.alloc_const(0_u64)?;
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
        let memidx = Instruction::memory_index(memory);
        self.push_fueled_instr(instr, FuelCosts::store)?;
        self.alloc.instr_encoder.append_instr(param)?;
        if !memory.is_default() {
            self.alloc.instr_encoder.append_instr(memidx)?;
        }
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

    fn translate_v128_store_lane_at<T: IntoLane>(
        &mut self,
        memory: index::Memory,
        address: Address32,
        value: Reg,
        lane: T::LaneType,
        make_instr_at: fn(value: Reg, address: Address32) -> Instruction,
    ) -> Result<(), Error> {
        self.push_fueled_instr(make_instr_at(value, address), FuelCosts::store)?;
        self.alloc
            .instr_encoder
            .append_instr(Instruction::lane_and_memory_index(lane, memory))?;
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
            FuelCosts::store,
        )?;
        Ok(Some(instr))
    }
}
