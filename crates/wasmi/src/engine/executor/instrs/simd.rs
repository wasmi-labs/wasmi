use super::Executor;
use crate::{
    core::{
        simd::{
            self,
            ImmLaneIdx16,
            ImmLaneIdx2,
            ImmLaneIdx32,
            ImmLaneIdx4,
            ImmLaneIdx8,
            IntoLaneIdx,
        },
        UntypedVal,
        WriteAs,
    },
    engine::{executor::InstructionPtr, utils::unreachable_unchecked},
    ir::{index, Address32, AnyConst32, Offset64, Offset64Lo, Offset8, Op, Reg, ShiftAmount},
    store::StoreInner,
    Error,
    TrapCode,
    V128,
};

#[cfg(doc)]
use crate::ir::Offset64Hi;

impl Executor<'_> {
    /// Fetches a [`Reg`] from an [`Op::Register`] instruction parameter.
    fn fetch_register(&self) -> Reg {
        let mut addr: InstructionPtr = self.ip;
        addr.add(1);
        match *addr.get() {
            Op::Register { reg } => reg,
            unexpected => {
                // Safety: Wasmi translation guarantees that [`Op::Register`] exists.
                unsafe {
                    unreachable_unchecked!("expected `Op::Register` but found {unexpected:?}")
                }
            }
        }
    }

    /// Fetches the [`Reg`] and [`Offset64Hi`] parameters for a load or store [`Op`].
    unsafe fn fetch_reg_and_lane<LaneType>(&self, delta: usize) -> (Reg, LaneType)
    where
        LaneType: TryFrom<u8>,
    {
        let mut addr: InstructionPtr = self.ip;
        addr.add(delta);
        match addr.get().filter_register_and_lane::<LaneType>() {
            Ok(value) => value,
            Err(instr) => unsafe {
                unreachable_unchecked!("expected an `Op::RegisterAndImm32` but found: {instr:?}")
            },
        }
    }

    /// Returns the register `value` and `lane` parameters for a `load` [`Op`].
    pub fn fetch_value_and_lane<LaneType>(&self, delta: usize) -> (Reg, LaneType)
    where
        LaneType: TryFrom<u8>,
    {
        // Safety: Wasmi translation guarantees that `Op::RegisterAndImm32` exists.
        unsafe { self.fetch_reg_and_lane::<LaneType>(delta) }
    }

    /// Fetches a [`Reg`] from an [`Op::Const32`] instruction parameter.
    fn fetch_const32_as<T>(&self) -> T
    where
        T: From<AnyConst32>,
    {
        let mut addr: InstructionPtr = self.ip;
        addr.add(1);
        match *addr.get() {
            Op::Const32 { value } => value.into(),
            unexpected => {
                // Safety: Wasmi translation guarantees that [`Op::Const32`] exists.
                unsafe { unreachable_unchecked!("expected `Op::Const32` but found {unexpected:?}") }
            }
        }
    }

    /// Fetches a [`Reg`] from an [`Op::I64Const32`] instruction parameter.
    fn fetch_i64const32(&self) -> i64 {
        let mut addr: InstructionPtr = self.ip;
        addr.add(1);
        match *addr.get() {
            Op::I64Const32 { value } => value.into(),
            unexpected => {
                // Safety: Wasmi translation guarantees that [`Op::I64Const32`] exists.
                unsafe {
                    unreachable_unchecked!("expected `Op::I64Const32` but found {unexpected:?}")
                }
            }
        }
    }

    /// Fetches a [`Reg`] from an [`Op::F64Const32`] instruction parameter.
    fn fetch_f64const32(&self) -> f64 {
        let mut addr: InstructionPtr = self.ip;
        addr.add(1);
        match *addr.get() {
            Op::F64Const32 { value } => value.into(),
            unexpected => {
                // Safety: Wasmi translation guarantees that [`Op::F64Const32`] exists.
                unsafe {
                    unreachable_unchecked!("expected `Op::F64Const32` but found {unexpected:?}")
                }
            }
        }
    }

    /// Executes an [`Op::I8x16Shuffle`] instruction.
    pub fn execute_i8x16_shuffle(&mut self, result: Reg, lhs: Reg, rhs: Reg) {
        let selector = self.fetch_register();
        let lhs = self.get_register_as::<V128>(lhs);
        let rhs = self.get_register_as::<V128>(rhs);
        let selector = self
            .get_register_as::<V128>(selector)
            .as_u128()
            .to_ne_bytes()
            .map(|lane| {
                match ImmLaneIdx32::try_from(lane) {
                    Ok(lane) => lane,
                    Err(_) => {
                        // Safety: Wasmi translation guarantees that the indices are within bounds.
                        unsafe { unreachable_unchecked!("unexpected out of bounds index: {lane}") }
                    }
                }
            });
        self.set_register_as::<V128>(result, simd::i8x16_shuffle(lhs, rhs, selector));
        self.next_instr_at(2);
    }
}

macro_rules! impl_ternary_simd_executors {
    ( $( (Op::$var_name:ident, $fn_name:ident, $op:expr $(,)?) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, a: Reg, b: Reg) {
                let c = self.fetch_register();
                let a = self.get_register_as::<V128>(a);
                let b = self.get_register_as::<V128>(b);
                let c = self.get_register_as::<V128>(c);
                self.set_register_as::<V128>(result, $op(a, b, c));
                self.next_instr_at(2);
            }
        )*
    };
}

impl Executor<'_> {
    impl_ternary_simd_executors! {
        (Op::V128Bitselect, execute_v128_bitselect, simd::v128_bitselect),
        (
            Op::I32x4RelaxedDotI8x16I7x16AddS,
            execute_i32x4_relaxed_dot_i8x16_i7x16_add_s,
            simd::i32x4_relaxed_dot_i8x16_i7x16_add_s,
        ),
        (Op::F32x4RelaxedMadd, execute_f32x4_relaxed_madd, simd::f32x4_relaxed_madd),
        (Op::F32x4RelaxedNmadd, execute_f32x4_relaxed_nmadd, simd::f32x4_relaxed_nmadd),
        (Op::F64x2RelaxedMadd, execute_f64x2_relaxed_madd, simd::f64x2_relaxed_madd),
        (Op::F64x2RelaxedNmadd, execute_f64x2_relaxed_nmadd, simd::f64x2_relaxed_nmadd),
    }
}

macro_rules! impl_replace_lane_ops {
    (
        $(
            ($ty:ty, Op::$instr_name:ident, $exec_name:ident, $execute:expr)
        ),* $(,)?
    ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($instr_name), "`].")]
            pub fn $exec_name(&mut self, result: Reg, input: Reg, lane: <$ty as IntoLaneIdx>::LaneIdx) {
                let value = self.fetch_register();
                let input = self.get_register_as::<V128>(input);
                let value = self.get_register_as::<$ty>(value);
                self.set_register_as::<V128>(result, $execute(input, lane, value));
                self.next_instr_at(2);
            }
        )*
    };
}

impl Executor<'_> {
    impl_replace_lane_ops! {
        (i8, Op::I8x16ReplaceLane, execute_i8x16_replace_lane, simd::i8x16_replace_lane),
        (i16, Op::I16x8ReplaceLane, execute_i16x8_replace_lane, simd::i16x8_replace_lane),
        (i32, Op::I32x4ReplaceLane, execute_i32x4_replace_lane, simd::i32x4_replace_lane),
        (i64, Op::I64x2ReplaceLane, execute_i64x2_replace_lane, simd::i64x2_replace_lane),
        (f32, Op::F32x4ReplaceLane, execute_f32x4_replace_lane, simd::f32x4_replace_lane),
        (f64, Op::F64x2ReplaceLane, execute_f64x2_replace_lane, simd::f64x2_replace_lane),
    }

    /// Executes an [`Op::I8x16ReplaceLaneImm`] instruction.
    pub fn execute_i8x16_replace_lane_imm(
        &mut self,
        result: Reg,
        input: Reg,
        lane: ImmLaneIdx16,
        value: i8,
    ) {
        self.execute_replace_lane_impl(result, input, lane, value, 1, simd::i8x16_replace_lane)
    }

    /// Executes an [`Op::I16x8ReplaceLaneImm`] instruction.
    pub fn execute_i16x8_replace_lane_imm(&mut self, result: Reg, input: Reg, lane: ImmLaneIdx8) {
        let value = self.fetch_const32_as::<i32>() as i16;
        self.execute_replace_lane_impl(result, input, lane, value, 2, simd::i16x8_replace_lane)
    }

    /// Executes an [`Op::I32x4ReplaceLaneImm`] instruction.
    pub fn execute_i32x4_replace_lane_imm(&mut self, result: Reg, input: Reg, lane: ImmLaneIdx4) {
        let value = self.fetch_const32_as::<i32>();
        self.execute_replace_lane_impl(result, input, lane, value, 2, simd::i32x4_replace_lane)
    }

    /// Executes an [`Op::I64x2ReplaceLaneImm32`] instruction.
    pub fn execute_i64x2_replace_lane_imm32(&mut self, result: Reg, input: Reg, lane: ImmLaneIdx2) {
        let value = self.fetch_i64const32();
        self.execute_replace_lane_impl(result, input, lane, value, 2, simd::i64x2_replace_lane)
    }

    /// Executes an [`Op::F32x4ReplaceLaneImm`] instruction.
    pub fn execute_f32x4_replace_lane_imm(&mut self, result: Reg, input: Reg, lane: ImmLaneIdx4) {
        let value = self.fetch_const32_as::<f32>();
        self.execute_replace_lane_impl(result, input, lane, value, 2, simd::f32x4_replace_lane)
    }

    /// Executes an [`Op::F64x2ReplaceLaneImm32`] instruction.
    pub fn execute_f64x2_replace_lane_imm32(&mut self, result: Reg, input: Reg, lane: ImmLaneIdx2) {
        let value = self.fetch_f64const32();
        self.execute_replace_lane_impl(result, input, lane, value, 2, simd::f64x2_replace_lane)
    }

    /// Generically execute a SIMD replace lane instruction.
    fn execute_replace_lane_impl<T, LaneType>(
        &mut self,
        result: Reg,
        input: Reg,
        lane: LaneType,
        value: T,
        delta: usize,
        eval: fn(V128, LaneType, T) -> V128,
    ) {
        let input = self.get_register_as::<V128>(input);
        self.set_register_as::<V128>(result, eval(input, lane, value));
        self.next_instr_at(delta);
    }

    impl_unary_executors! {
        (Op::V128AnyTrue, execute_v128_any_true, simd::v128_any_true),
        (Op::I8x16AllTrue, execute_i8x16_all_true, simd::i8x16_all_true),
        (Op::I8x16Bitmask, execute_i8x16_bitmask, simd::i8x16_bitmask),
        (Op::I16x8AllTrue, execute_i16x8_all_true, simd::i16x8_all_true),
        (Op::I16x8Bitmask, execute_i16x8_bitmask, simd::i16x8_bitmask),
        (Op::I32x4AllTrue, execute_i32x4_all_true, simd::i32x4_all_true),
        (Op::I32x4Bitmask, execute_i32x4_bitmask, simd::i32x4_bitmask),
        (Op::I64x2AllTrue, execute_i64x2_all_true, simd::i64x2_all_true),
        (Op::I64x2Bitmask, execute_i64x2_bitmask, simd::i64x2_bitmask),

        (Op::I8x16Neg, execute_i8x16_neg, simd::i8x16_neg),
        (Op::I16x8Neg, execute_i16x8_neg, simd::i16x8_neg),
        (Op::I16x8Neg, execute_i32x4_neg, simd::i32x4_neg),
        (Op::I16x8Neg, execute_i64x2_neg, simd::i64x2_neg),
        (Op::I16x8Neg, execute_f32x4_neg, simd::f32x4_neg),
        (Op::I16x8Neg, execute_f64x2_neg, simd::f64x2_neg),

        (Op::I8x16Abs, execute_i8x16_abs, simd::i8x16_abs),
        (Op::I16x8Abs, execute_i16x8_abs, simd::i16x8_abs),
        (Op::I16x8Abs, execute_i32x4_abs, simd::i32x4_abs),
        (Op::I16x8Abs, execute_i64x2_abs, simd::i64x2_abs),
        (Op::I16x8Abs, execute_f32x4_abs, simd::f32x4_abs),
        (Op::I16x8Abs, execute_f64x2_abs, simd::f64x2_abs),

        (Op::I8x16Splat, execute_i8x16_splat, simd::i8x16_splat),
        (Op::I16x8Splat, execute_i16x8_splat, simd::i16x8_splat),
        (Op::I32x4Splat, execute_i32x4_splat, simd::i32x4_splat),
        (Op::I64x2Splat, execute_i64x2_splat, simd::i64x2_splat),
        (Op::F32x4Splat, execute_f32x4_splat, simd::f32x4_splat),
        (Op::F64x2Splat, execute_f64x2_splat, simd::f64x2_splat),

        (Op::I16x8ExtaddPairwiseI8x16S, execute_i16x8_extadd_pairwise_i8x16_s, simd::i16x8_extadd_pairwise_i8x16_s),
        (Op::I16x8ExtaddPairwiseI8x16U, execute_i16x8_extadd_pairwise_i8x16_u, simd::i16x8_extadd_pairwise_i8x16_u),
        (Op::I32x4ExtaddPairwiseI16x8S, execute_i32x4_extadd_pairwise_i16x8_s, simd::i32x4_extadd_pairwise_i16x8_s),
        (Op::I32x4ExtaddPairwiseI16x8U, execute_i32x4_extadd_pairwise_i16x8_u, simd::i32x4_extadd_pairwise_i16x8_u),

        (Op::F32x4Ceil, execute_f32x4_ceil, simd::f32x4_ceil),
        (Op::F32x4Floor, execute_f32x4_floor, simd::f32x4_floor),
        (Op::F32x4Trunc, execute_f32x4_trunc, simd::f32x4_trunc),
        (Op::F32x4Nearest, execute_f32x4_nearest, simd::f32x4_nearest),
        (Op::F32x4Sqrt, execute_f32x4_sqrt, simd::f32x4_sqrt),
        (Op::F64x2Ceil, execute_f64x2_ceil, simd::f64x2_ceil),
        (Op::F64x2Floor, execute_f64x2_floor, simd::f64x2_floor),
        (Op::F64x2Trunc, execute_f64x2_trunc, simd::f64x2_trunc),
        (Op::F64x2Nearest, execute_f64x2_nearest, simd::f64x2_nearest),
        (Op::F64x2Sqrt, execute_f64x2_sqrt, simd::f64x2_sqrt),

        (Op::V128Not, execute_v128_not, simd::v128_not),
        (Op::I8x16Popcnt, execute_i8x16_popcnt, simd::i8x16_popcnt),

        (Op::i16x8_extend_low_i8x16_s, execute_i16x8_extend_low_i8x16_s, simd::i16x8_extend_low_i8x16_s),
        (Op::i16x8_extend_high_i8x16_s, execute_i16x8_extend_high_i8x16_s, simd::i16x8_extend_high_i8x16_s),
        (Op::i16x8_extend_low_i8x16_u, execute_i16x8_extend_low_i8x16_u, simd::i16x8_extend_low_i8x16_u),
        (Op::i16x8_extend_high_i8x16_u, execute_i16x8_extend_high_i8x16_u, simd::i16x8_extend_high_i8x16_u),
        (Op::i32x4_extend_low_i16x8_s, execute_i32x4_extend_low_i16x8_s, simd::i32x4_extend_low_i16x8_s),
        (Op::i32x4_extend_high_i16x8_s, execute_i32x4_extend_high_i16x8_s, simd::i32x4_extend_high_i16x8_s),
        (Op::i32x4_extend_low_i16x8_u, execute_i32x4_extend_low_i16x8_u, simd::i32x4_extend_low_i16x8_u),
        (Op::i32x4_extend_high_i16x8_u, execute_i32x4_extend_high_i16x8_u, simd::i32x4_extend_high_i16x8_u),
        (Op::i64x2_extend_low_i32x4_s, execute_i64x2_extend_low_i32x4_s, simd::i64x2_extend_low_i32x4_s),
        (Op::i64x2_extend_high_i32x4_s, execute_i64x2_extend_high_i32x4_s, simd::i64x2_extend_high_i32x4_s),
        (Op::i64x2_extend_low_i32x4_u, execute_i64x2_extend_low_i32x4_u, simd::i64x2_extend_low_i32x4_u),
        (Op::i64x2_extend_high_i32x4_u, execute_i64x2_extend_high_i32x4_u, simd::i64x2_extend_high_i32x4_u),

        (Op::I32x4TruncSatF32x4S, execute_i32x4_trunc_sat_f32x4_s, simd::i32x4_trunc_sat_f32x4_s),
        (Op::I32x4TruncSatF32x4U, execute_i32x4_trunc_sat_f32x4_u, simd::i32x4_trunc_sat_f32x4_u),
        (Op::F32x4ConvertI32x4S, execute_f32x4_convert_i32x4_s, simd::f32x4_convert_i32x4_s),
        (Op::F32x4ConvertI32x4U, execute_f32x4_convert_i32x4_u, simd::f32x4_convert_i32x4_u),
        (Op::I32x4TruncSatF64x2SZero, execute_i32x4_trunc_sat_f64x2_s_zero, simd::i32x4_trunc_sat_f64x2_s_zero),
        (Op::I32x4TruncSatF64x2UZero, execute_i32x4_trunc_sat_f64x2_u_zero, simd::i32x4_trunc_sat_f64x2_u_zero),
        (Op::F64x2ConvertLowI32x4S, execute_f64x2_convert_low_i32x4_s, simd::f64x2_convert_low_i32x4_s),
        (Op::F64x2ConvertLowI32x4U, execute_f64x2_convert_low_i32x4_u, simd::f64x2_convert_low_i32x4_u),
        (Op::F32x4DemoteF64x2Zero, execute_f32x4_demote_f64x2_zero, simd::f32x4_demote_f64x2_zero),
        (Op::F64x2PromoteLowF32x4, execute_f64x2_promote_low_f32x4, simd::f64x2_promote_low_f32x4),
    }

    impl_binary_executors! {
        (Op::I8x16Swizzle, execute_i8x16_swizzle, simd::i8x16_swizzle),

        (Op::I16x8Q15MulrSatS, execute_i16x8_q15mulr_sat_s, simd::i16x8_q15mulr_sat_s),
        (Op::I32x4DotI16x8S, execute_i32x4_dot_i16x8_s, simd::i32x4_dot_i16x8_s),
        (Op::I16x8RelaxedDotI8x16I7x16S, execute_i16x8_relaxed_dot_i8x16_i7x16_s, simd::i16x8_relaxed_dot_i8x16_i7x16_s),

        (Op::I16x8ExtmulLowI8x16S, execute_i16x8_extmul_low_i8x16_s, simd::i16x8_extmul_low_i8x16_s),
        (Op::I16x8ExtmulHighI8x16S, execute_i16x8_extmul_high_i8x16_s, simd::i16x8_extmul_high_i8x16_s),
        (Op::I16x8ExtmulLowI8x16U, execute_i16x8_extmul_low_i8x16_u, simd::i16x8_extmul_low_i8x16_u),
        (Op::I16x8ExtmulHighI8x16U, execute_i16x8_extmul_high_i8x16_u, simd::i16x8_extmul_high_i8x16_u),
        (Op::I32x4ExtmulLowI16x8S, execute_i32x4_extmul_low_i16x8_s, simd::i32x4_extmul_low_i16x8_s),
        (Op::I32x4ExtmulHighI16x8S, execute_i32x4_extmul_high_i16x8_s, simd::i32x4_extmul_high_i16x8_s),
        (Op::I32x4ExtmulLowI16x8U, execute_i32x4_extmul_low_i16x8_u, simd::i32x4_extmul_low_i16x8_u),
        (Op::I32x4ExtmulHighI16x8U, execute_i32x4_extmul_high_i16x8_u, simd::i32x4_extmul_high_i16x8_u),
        (Op::I64x2ExtmulLowI32x4S, execute_i64x2_extmul_low_i32x4_s, simd::i64x2_extmul_low_i32x4_s),
        (Op::I64x2ExtmulHighI32x4S, execute_i64x2_extmul_high_i32x4_s, simd::i64x2_extmul_high_i32x4_s),
        (Op::I64x2ExtmulLowI32x4U, execute_i64x2_extmul_low_i32x4_u, simd::i64x2_extmul_low_i32x4_u),
        (Op::I64x2ExtmulHighI32x4U, execute_i64x2_extmul_high_i32x4_u, simd::i64x2_extmul_high_i32x4_u),

        (Op::I32x4Add, execute_i32x4_add, simd::i32x4_add),
        (Op::I32x4Sub, execute_i32x4_sub, simd::i32x4_sub),
        (Op::I32x4Mul, execute_i32x4_mul, simd::i32x4_mul),

        (Op::I64x2Add, execute_i64x2_add, simd::i64x2_add),
        (Op::I64x2Sub, execute_i64x2_sub, simd::i64x2_sub),
        (Op::I64x2Mul, execute_i64x2_mul, simd::i64x2_mul),

        (Op::I8x16Eq, execute_i8x16_eq, simd::i8x16_eq),
        (Op::I8x16Ne, execute_i8x16_ne, simd::i8x16_ne),
        (Op::I8x16LtS, execute_i8x16_lt_s, simd::i8x16_lt_s),
        (Op::I8x16LtU, execute_i8x16_lt_u, simd::i8x16_lt_u),
        (Op::I8x16LeS, execute_i8x16_le_s, simd::i8x16_le_s),
        (Op::I8x16LeU, execute_i8x16_le_u, simd::i8x16_le_u),
        (Op::I16x8Eq, execute_i16x8_eq, simd::i16x8_eq),
        (Op::I16x8Ne, execute_i16x8_ne, simd::i16x8_ne),
        (Op::I16x8LtS, execute_i16x8_lt_s, simd::i16x8_lt_s),
        (Op::I16x8LtU, execute_i16x8_lt_u, simd::i16x8_lt_u),
        (Op::I16x8LeS, execute_i16x8_le_s, simd::i16x8_le_s),
        (Op::I16x8LeU, execute_i16x8_le_u, simd::i16x8_le_u),
        (Op::I32x4Eq, execute_i32x4_eq, simd::i32x4_eq),
        (Op::I32x4Ne, execute_i32x4_ne, simd::i32x4_ne),
        (Op::I32x4LtS, execute_i32x4_lt_s, simd::i32x4_lt_s),
        (Op::I32x4LtU, execute_i32x4_lt_u, simd::i32x4_lt_u),
        (Op::I32x4LeS, execute_i32x4_le_s, simd::i32x4_le_s),
        (Op::I32x4LeU, execute_i32x4_le_u, simd::i32x4_le_u),
        (Op::I64x2Eq, execute_i64x2_eq, simd::i64x2_eq),
        (Op::I64x2Ne, execute_i64x2_ne, simd::i64x2_ne),
        (Op::I64x2LtS, execute_i64x2_lt_s, simd::i64x2_lt_s),
        (Op::I64x2LeS, execute_i64x2_le_s, simd::i64x2_le_s),
        (Op::F32x4Eq, execute_f32x4_eq, simd::f32x4_eq),
        (Op::F32x4Ne, execute_f32x4_ne, simd::f32x4_ne),
        (Op::F32x4Lt, execute_f32x4_lt, simd::f32x4_lt),
        (Op::F32x4Le, execute_f32x4_le, simd::f32x4_le),
        (Op::F64x2Eq, execute_f64x2_eq, simd::f64x2_eq),
        (Op::F64x2Ne, execute_f64x2_ne, simd::f64x2_ne),
        (Op::F64x2Lt, execute_f64x2_lt, simd::f64x2_lt),
        (Op::F64x2Le, execute_f64x2_le, simd::f64x2_le),

        (Op::I8x16MinS, execute_i8x16_min_s, simd::i8x16_min_s),
        (Op::I8x16MinU, execute_i8x16_min_u, simd::i8x16_min_u),
        (Op::I8x16MaxS, execute_i8x16_max_s, simd::i8x16_max_s),
        (Op::I8x16MaxU, execute_i8x16_max_u, simd::i8x16_max_u),
        (Op::I8x16AvgrU, execute_i8x16_avgr_u, simd::i8x16_avgr_u),
        (Op::I16x8MinS, execute_i16x8_min_s, simd::i16x8_min_s),
        (Op::I16x8MinU, execute_i16x8_min_u, simd::i16x8_min_u),
        (Op::I16x8MaxS, execute_i16x8_max_s, simd::i16x8_max_s),
        (Op::I16x8MaxU, execute_i16x8_max_u, simd::i16x8_max_u),
        (Op::I16x8AvgrU, execute_i16x8_avgr_u, simd::i16x8_avgr_u),
        (Op::I32x4MinS, execute_i32x4_min_s, simd::i32x4_min_s),
        (Op::I32x4MinU, execute_i32x4_min_u, simd::i32x4_min_u),
        (Op::I32x4MaxS, execute_i32x4_max_s, simd::i32x4_max_s),
        (Op::I32x4MaxU, execute_i32x4_max_u, simd::i32x4_max_u),

        (Op::I8x16Shl, execute_i8x16_shl, simd::i8x16_shl),
        (Op::I8x16ShrS, execute_i8x16_shr_s, simd::i8x16_shr_s),
        (Op::I8x16ShrU, execute_i8x16_shr_u, simd::i8x16_shr_u),
        (Op::I16x8Shl, execute_i16x8_shl, simd::i16x8_shl),
        (Op::I16x8ShrS, execute_i16x8_shr_s, simd::i16x8_shr_s),
        (Op::I16x8ShrU, execute_i16x8_shr_u, simd::i16x8_shr_u),
        (Op::I32x4Shl, execute_i32x4_shl, simd::i32x4_shl),
        (Op::I32x4ShrS, execute_i32x4_shr_s, simd::i32x4_shr_s),
        (Op::I32x4ShrU, execute_i32x4_shr_u, simd::i32x4_shr_u),
        (Op::I64x2Shl, execute_i64x2_shl, simd::i64x2_shl),
        (Op::I64x2ShrS, execute_i64x2_shr_s, simd::i64x2_shr_s),
        (Op::I64x2ShrU, execute_i64x2_shr_u, simd::i64x2_shr_u),

        (Op::I8x16Add, execute_i8x16_add, simd::i8x16_add),
        (Op::I8x16AddSatS, execute_i8x16_add_sat_s, simd::i8x16_add_sat_s),
        (Op::I8x16AddSatU, execute_i8x16_add_sat_u, simd::i8x16_add_sat_u),
        (Op::I8x16Sub, execute_i8x16_sub, simd::i8x16_sub),
        (Op::I8x16SubSatS, execute_i8x16_sub_sat_s, simd::i8x16_sub_sat_s),
        (Op::I8x16SubSatU, execute_i8x16_sub_sat_u, simd::i8x16_sub_sat_u),

        (Op::I16x8Add, execute_i16x8_add, simd::i16x8_add),
        (Op::I16x8AddSatS, execute_i16x8_add_sat_s, simd::i16x8_add_sat_s),
        (Op::I16x8AddSatU, execute_i16x8_add_sat_u, simd::i16x8_add_sat_u),
        (Op::I16x8Sub, execute_i16x8_sub, simd::i16x8_sub),
        (Op::I16x8SubSatS, execute_i16x8_sub_sat_s, simd::i16x8_sub_sat_s),
        (Op::I16x8SubSatU, execute_i16x8_sub_sat_u, simd::i16x8_sub_sat_u),
        (Op::I16x8Sub, execute_i16x8_mul, simd::i16x8_mul),

        (Op::V128And, execute_v128_and, simd::v128_and),
        (Op::V128Andnot, execute_v128_andnot, simd::v128_andnot),
        (Op::V128Or, execute_v128_or, simd::v128_or),
        (Op::V128Xor, execute_v128_xor, simd::v128_xor),

        (Op::F32x4Add, execute_f32x4_add, simd::f32x4_add),
        (Op::F32x4Sub, execute_f32x4_sub, simd::f32x4_sub),
        (Op::F32x4Mul, execute_f32x4_mul, simd::f32x4_mul),
        (Op::F32x4Div, execute_f32x4_div, simd::f32x4_div),
        (Op::F32x4Min, execute_f32x4_min, simd::f32x4_min),
        (Op::F32x4Max, execute_f32x4_max, simd::f32x4_max),
        (Op::F32x4Pmin, execute_f32x4_pmin, simd::f32x4_pmin),
        (Op::F32x4Pmax, execute_f32x4_pmax, simd::f32x4_pmax),

        (Op::F64x2Add, execute_f64x2_add, simd::f64x2_add),
        (Op::F64x2Sub, execute_f64x2_sub, simd::f64x2_sub),
        (Op::F64x2Mul, execute_f64x2_mul, simd::f64x2_mul),
        (Op::F64x2Div, execute_f64x2_div, simd::f64x2_div),
        (Op::F64x2Min, execute_f64x2_min, simd::f64x2_min),
        (Op::F64x2Max, execute_f64x2_max, simd::f64x2_max),
        (Op::F64x2Pmin, execute_f64x2_pmin, simd::f64x2_pmin),
        (Op::F64x2Pmax, execute_f64x2_pmax, simd::f64x2_pmax),

        (Op::I8x16NarrowI16x8S, execute_i8x16_narrow_i16x8_s, simd::i8x16_narrow_i16x8_s),
        (Op::I8x16NarrowI16x8U, execute_i8x16_narrow_i16x8_u, simd::i8x16_narrow_i16x8_u),
        (Op::I16x8NarrowI32x4S, execute_i16x8_narrow_i32x4_s, simd::i16x8_narrow_i32x4_s),
        (Op::I16x8NarrowI32x4U, execute_i16x8_narrow_i32x4_u, simd::i16x8_narrow_i32x4_u),
    }
}

impl Executor<'_> {
    /// Executes a generic SIMD extract-lane [`Op`].
    #[inline(always)]
    fn execute_extract_lane<T, Lane>(
        &mut self,
        result: Reg,
        input: Reg,
        lane: Lane,
        op: fn(V128, Lane) -> T,
    ) where
        UntypedVal: WriteAs<T>,
    {
        let input = self.get_register_as::<V128>(input);
        self.set_register_as::<T>(result, op(input, lane));
        self.next_instr();
    }
}

macro_rules! impl_extract_lane_executors {
    (
        $( ($ty:ty, Op::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)?
    ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, input: Reg, lane: <$ty as IntoLaneIdx>::LaneIdx) {
                self.execute_extract_lane(result, input, lane, $op)
            }
        )*
    };
}
impl Executor<'_> {
    impl_extract_lane_executors! {
        (i8, Op::I8x16ExtractLaneS, i8x16_extract_lane_s, simd::i8x16_extract_lane_s),
        (u8, Op::I8x16ExtractLaneU, i8x16_extract_lane_u, simd::i8x16_extract_lane_u),
        (i16, Op::I16x8ExtractLaneS, i16x8_extract_lane_s, simd::i16x8_extract_lane_s),
        (u16, Op::I16x8ExtractLaneU, i16x8_extract_lane_u, simd::i16x8_extract_lane_u),
        (i32, Op::I32x4ExtractLane, i32x4_extract_lane, simd::i32x4_extract_lane),
        (u32, Op::F32x4ExtractLane, f32x4_extract_lane, simd::f32x4_extract_lane),
        (i64, Op::I64x2ExtractLane, i64x2_extract_lane, simd::i64x2_extract_lane),
        (u64, Op::F64x2ExtractLane, f64x2_extract_lane, simd::f64x2_extract_lane),
    }
}

impl Executor<'_> {
    /// Generically execute a SIMD shift operation with immediate shift amount.
    #[inline(always)]
    fn execute_simd_shift_by(
        &mut self,
        result: Reg,
        lhs: Reg,
        rhs: ShiftAmount<u32>,
        op: fn(V128, u32) -> V128,
    ) {
        let lhs = self.get_register_as::<V128>(lhs);
        let rhs = rhs.into();
        self.set_register_as::<V128>(result, op(lhs, rhs));
        self.next_instr();
    }
}

macro_rules! impl_simd_shift_executors {
    ( $( (Op::$var_name:ident, $fn_name:ident, $op:expr) ),* $(,)? ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($var_name), "`].")]
            pub fn $fn_name(&mut self, result: Reg, lhs: Reg, rhs: ShiftAmount<u32>) {
                self.execute_simd_shift_by(result, lhs, rhs, $op)
            }
        )*
    };
}
impl Executor<'_> {
    impl_simd_shift_executors! {
        (Op::I8x16ShlBy, execute_i8x16_shl_by, simd::i8x16_shl),
        (Op::I8x16ShrSBy, execute_i8x16_shr_s_by, simd::i8x16_shr_s),
        (Op::I8x16ShrUBy, execute_i8x16_shr_u_by, simd::i8x16_shr_u),
        (Op::I16x8ShlBy, execute_i16x8_shl_by, simd::i16x8_shl),
        (Op::I16x8ShrSBy, execute_i16x8_shr_s_by, simd::i16x8_shr_s),
        (Op::I16x8ShrUBy, execute_i16x8_shr_u_by, simd::i16x8_shr_u),
        (Op::I32x4ShlBy, execute_i32x4_shl_by, simd::i32x4_shl),
        (Op::I32x4ShrSBy, execute_i32x4_shr_s_by, simd::i32x4_shr_s),
        (Op::I32x4ShrUBy, execute_i32x4_shr_u_by, simd::i32x4_shr_u),
        (Op::I64x2ShlBy, execute_i64x2_shl_by, simd::i64x2_shl),
        (Op::I64x2ShrSBy, execute_i64x2_shr_s_by, simd::i64x2_shr_s),
        (Op::I64x2ShrUBy, execute_i64x2_shr_u_by, simd::i64x2_shr_u),
    }
}

impl Executor<'_> {
    /// Returns the optional `memory` parameter for a `load_at` [`Op`].
    ///
    /// # Note
    ///
    /// - Returns the default [`index::Memory`] if the parameter is missing.
    /// - Bumps `self.ip` if a [`Op::MemoryIndex`] parameter was found.
    #[inline(always)]
    fn fetch_lane_and_memory<LaneType>(&mut self, delta: usize) -> (LaneType, index::Memory)
    where
        LaneType: TryFrom<u8>,
    {
        let mut addr: InstructionPtr = self.ip;
        addr.add(delta);
        match addr.get().filter_lane_and_memory() {
            Ok(value) => value,
            Err(instr) => unsafe {
                unreachable_unchecked!("expected an `Op::Imm16AndImm32` but found: {instr:?}")
            },
        }
    }
}

type V128LoadLane<LaneType> =
    fn(memory: &[u8], ptr: u64, offset: u64, x: V128, lane: LaneType) -> Result<V128, TrapCode>;

type V128LoadLaneAt<LaneType> =
    fn(memory: &[u8], address: usize, x: V128, lane: LaneType) -> Result<V128, TrapCode>;

macro_rules! impl_execute_v128_load_lane {
    (
        $( ($ty:ty, Op::$op:ident, $exec:ident, $eval:expr) ),* $(,)?
    ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($op), "`] instruction.")]
            pub fn $exec(
                &mut self,
                store: &StoreInner,
                result: Reg,
                offset_lo: Offset64Lo,
            ) -> Result<(), Error> {
                self.execute_v128_load_lane_impl::<<$ty as IntoLaneIdx>::LaneIdx>(store, result, offset_lo, $eval)
            }
        )*
    };
}

macro_rules! impl_execute_v128_load_lane_at {
    (
        $( ($ty:ty, Op::$op:ident, $exec:ident, $eval:expr) ),* $(,)?
    ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($op), "`] instruction.")]
            pub fn $exec(
                &mut self,
                store: &StoreInner,
                result: Reg,
                address: Address32,
            ) -> Result<(), Error> {
                self.execute_v128_load_lane_at_impl::<<$ty as IntoLaneIdx>::LaneIdx>(store, result, address, $eval)
            }
        )*
    };
}

impl Executor<'_> {
    fn execute_v128_load_lane_impl<LaneType>(
        &mut self,
        store: &StoreInner,
        result: Reg,
        offset_lo: Offset64Lo,
        load: V128LoadLane<LaneType>,
    ) -> Result<(), Error>
    where
        LaneType: TryFrom<u8> + Into<u8> + Copy,
    {
        let (ptr, offset_hi) = self.fetch_value_and_offset_hi();
        let (v128, lane) = self.fetch_value_and_lane::<LaneType>(2);
        let memory = self.fetch_optional_memory(3);
        let offset = Offset64::combine(offset_hi, offset_lo);
        let ptr = self.get_register_as::<u64>(ptr);
        let v128 = self.get_register_as::<V128>(v128);
        let memory = self.fetch_memory_bytes(memory, store);
        let loaded = load(memory, ptr, u64::from(offset), v128, lane)?;
        self.set_register_as::<V128>(result, loaded);
        self.try_next_instr_at(3)
    }

    impl_execute_v128_load_lane! {
        (u8, Op::V128Load8Lane, execute_v128_load8_lane, simd::v128_load8_lane),
        (u16, Op::V128Load16Lane, execute_v128_load16_lane, simd::v128_load16_lane),
        (u32, Op::V128Load32Lane, execute_v128_load32_lane, simd::v128_load32_lane),
        (u64, Op::V128Load64Lane, execute_v128_load64_lane, simd::v128_load64_lane),
    }

    fn execute_v128_load_lane_at_impl<LaneType>(
        &mut self,
        store: &StoreInner,
        result: Reg,
        address: Address32,
        load_at: V128LoadLaneAt<LaneType>,
    ) -> Result<(), Error>
    where
        LaneType: TryFrom<u8> + Into<u8> + Copy,
    {
        let (v128, lane) = self.fetch_value_and_lane::<LaneType>(1);
        let memory = self.fetch_optional_memory(2);
        let v128 = self.get_register_as::<V128>(v128);
        let memory = self.fetch_memory_bytes(memory, store);
        let loaded = load_at(memory, usize::from(address), v128, lane)?;
        self.set_register_as::<V128>(result, loaded);
        self.try_next_instr_at(2)
    }

    impl_execute_v128_load_lane_at! {
        (u8, Op::V128Load8LaneAt, execute_v128_load8_lane_at, simd::v128_load8_lane_at),
        (u16, Op::V128Load16LaneAt, execute_v128_load16_lane_at, simd::v128_load16_lane_at),
        (u32, Op::V128Load32LaneAt, execute_v128_load32_lane_at, simd::v128_load32_lane_at),
        (u64, Op::V128Load64LaneAt, execute_v128_load64_lane_at, simd::v128_load64_lane_at),
    }
}

macro_rules! impl_execute_v128_store_lane {
    (
        $( ($ty:ty, Op::$op:ident, $exec:ident, $eval:expr) ),* $(,)?
    ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($op), "`] instruction.")]
            pub fn $exec(
                &mut self,
                store: &mut StoreInner,
                ptr: Reg,
                offset_lo: Offset64Lo,
            ) -> Result<(), Error> {
                self.execute_v128_store_lane::<$ty>(store, ptr, offset_lo, $eval)
            }
        )*
    };
}

macro_rules! impl_execute_v128_store_lane_offset16 {
    (
        $( ($ty:ty, Op::$op:ident, $exec:ident, $eval:expr) ),* $(,)?
    ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($op), "`] instruction.")]
            pub fn $exec(
                &mut self,
                ptr: Reg,
                value: Reg,
                offset: Offset8,
                lane: <$ty as IntoLaneIdx>::LaneIdx,
            ) -> Result<(), Error> {
                self.execute_v128_store_lane_offset8::<$ty>(ptr, value, offset, lane, $eval)
            }
        )*
    };
}

macro_rules! impl_execute_v128_store_lane_at {
    (
        $( ($ty:ty, Op::$op:ident, $exec:ident, $eval:expr) ),* $(,)?
    ) => {
        $(
            #[doc = concat!("Executes an [`Op::", stringify!($op), "`] instruction.")]
            pub fn $exec(
                &mut self,
                store: &mut StoreInner,
                value: Reg,
                address: Address32,
            ) -> Result<(), Error> {
                self.execute_v128_store_lane_at::<$ty>(store, value, address, $eval)
            }
        )*
    };
}

type V128StoreLane<LaneType> = fn(
    memory: &mut [u8],
    ptr: u64,
    offset: u64,
    value: V128,
    lane: LaneType,
) -> Result<(), TrapCode>;

type V128StoreLaneAt<LaneType> =
    fn(memory: &mut [u8], address: usize, value: V128, lane: LaneType) -> Result<(), TrapCode>;

impl Executor<'_> {
    fn execute_v128_store_lane<T: IntoLaneIdx>(
        &mut self,
        store: &mut StoreInner,
        ptr: Reg,
        offset_lo: Offset64Lo,
        eval: V128StoreLane<T::LaneIdx>,
    ) -> Result<(), Error> {
        let (value, offset_hi) = self.fetch_value_and_offset_hi();
        let (lane, memory) = self.fetch_lane_and_memory(2);
        let offset = Offset64::combine(offset_hi, offset_lo);
        let ptr = self.get_register_as::<u64>(ptr);
        let v128 = self.get_register_as::<V128>(value);
        let memory = self.fetch_memory_bytes_mut(memory, store);
        eval(memory, ptr, u64::from(offset), v128, lane)?;
        self.try_next_instr_at(3)
    }

    impl_execute_v128_store_lane! {
        (u8, Op::V128Store8Lane, execute_v128_store8_lane, simd::v128_store8_lane),
        (u16, Op::V128Store16Lane, execute_v128_store16_lane, simd::v128_store16_lane),
        (u32, Op::V128Store32Lane, execute_v128_store32_lane, simd::v128_store32_lane),
        (u64, Op::V128Store64Lane, execute_v128_store64_lane, simd::v128_store64_lane),
    }

    fn execute_v128_store_lane_offset8<T: IntoLaneIdx>(
        &mut self,
        ptr: Reg,
        value: Reg,
        offset: Offset8,
        lane: T::LaneIdx,
        eval: V128StoreLane<T::LaneIdx>,
    ) -> Result<(), Error> {
        let ptr = self.get_register_as::<u64>(ptr);
        let offset = u64::from(Offset64::from(offset));
        let v128 = self.get_register_as::<V128>(value);
        let memory = self.fetch_default_memory_bytes_mut();
        eval(memory, ptr, offset, v128, lane)?;
        self.try_next_instr()
    }

    impl_execute_v128_store_lane_offset16! {
        (u8, Op::V128Store8LaneOffset8, execute_v128_store8_lane_offset8, simd::v128_store8_lane),
        (u16, Op::V128Store16LaneOffset8, execute_v128_store16_lane_offset8, simd::v128_store16_lane),
        (u32, Op::V128Store32LaneOffset8, execute_v128_store32_lane_offset8, simd::v128_store32_lane),
        (u64, Op::V128Store64LaneOffset8, execute_v128_store64_lane_offset8, simd::v128_store64_lane),
    }

    fn execute_v128_store_lane_at<T: IntoLaneIdx>(
        &mut self,
        store: &mut StoreInner,
        value: Reg,
        address: Address32,
        eval: V128StoreLaneAt<T::LaneIdx>,
    ) -> Result<(), Error>
    where
        T::LaneIdx: TryFrom<u8> + Into<u8>,
    {
        let (lane, memory) = self.fetch_lane_and_memory::<T::LaneIdx>(1);
        let v128 = self.get_register_as::<V128>(value);
        let memory = self.fetch_memory_bytes_mut(memory, store);
        eval(memory, usize::from(address), v128, lane)?;
        self.try_next_instr_at(2)
    }

    impl_execute_v128_store_lane_at! {
        (u8, Op::V128Store8LaneAt, execute_v128_store8_lane_at, simd::v128_store8_lane_at),
        (u16, Op::V128Store16LaneAt, execute_v128_store16_lane_at, simd::v128_store16_lane_at),
        (u32, Op::V128Store32LaneAt, execute_v128_store32_lane_at, simd::v128_store32_lane_at),
        (u64, Op::V128Store64LaneAt, execute_v128_store64_lane_at, simd::v128_store64_lane_at),
    }
}
