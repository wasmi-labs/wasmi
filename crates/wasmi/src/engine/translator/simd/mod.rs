mod visit;

use super::{utils::Wrap, FuncTranslator};
use crate::{
    core::{simd, TypedVal, V128},
    engine::{translator::provider::Provider, FuelCosts},
    ir::{Instruction, IntoShiftAmount, Reg},
    Error,
};

trait IntoLane {
    type LaneType;
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
        T::LaneType: TryFrom<u8>,
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
    fn translate_simd_unary(
        &mut self,
        make_instr: fn(result: Reg, input: Reg) -> Instruction,
        const_eval: fn(input: V128) -> V128,
    ) -> Result<(), Error> {
        bail_unreachable!(self);
        let input = self.alloc.stack.pop();
        if let Provider::Const(input) = input {
            // Case: both inputs are immediates so we can const-eval the result.
            let result = const_eval(input.into());
            self.alloc.stack.push_const(result);
            return Ok(());
        }
        let result = self.alloc.stack.push_dynamic()?;
        let input = self.alloc.stack.provider2reg(&input)?;
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

    /// Generically translate a Wasm shift instruction.
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
        T: From<TypedVal> + IntoShiftAmount,
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
                let Some(rhs) = T::from(rhs).into_shift_amount() else {
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
}
