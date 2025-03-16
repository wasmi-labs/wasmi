#![allow(dead_code)] // TODO: remove silencing of warnings again

use crate::{wasm, ReadAs, UntypedVal, WriteAs};

/// The Wasm `simd` proposal's `v128` type.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct V128([u8; 16]);

impl From<UntypedVal> for V128 {
    fn from(value: UntypedVal) -> Self {
        let u128 = (u128::from(value.hi64) << 64) | (u128::from(value.lo64));
        Self(u128.to_le_bytes())
    }
}

impl From<V128> for UntypedVal {
    fn from(value: V128) -> Self {
        let u128 = u128::from_le_bytes(value.0);
        let lo64 = u128 as u64;
        let hi64 = (u128 >> 64) as u64;
        Self { lo64, hi64 }
    }
}

impl ReadAs<V128> for UntypedVal {
    fn read_as(&self) -> V128 {
        // Note: we can re-use the `From` impl since both types are of equal size.
        V128::from(*self)
    }
}

impl WriteAs<V128> for UntypedVal {
    fn write_as(&mut self, value: V128) {
        // Note: we can re-use the `From` impl since both types are of equal size.
        *self = UntypedVal::from(value);
    }
}

/// A single unconstrained byte (0-255).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ImmByte(u8);

pub struct OutOfBoundsLaneId;

pub trait IntoLaneIdx {
    type LaneIdx;
}

macro_rules! impl_imm_lane_id {
    (
        $(
            $( #[$attr:meta] )*
            struct $name:ident(x $n:literal);
        )*
    ) => {
        $(
            $( #[$attr] )*
            #[derive(Debug, Copy, Clone, PartialEq, Eq)]
            #[repr(transparent)]
            pub struct $name(u8);

            impl IntoLaneIdx for [(); $n] {
                type LaneIdx = $name;
            }

            impl $name {
                /// Helper bit mask for construction and getter.
                const MASK: u8 = (1_u8 << u8::ilog2($n)) - 1;

                /// Returns the lane id as `u8`.
                ///
                /// This will never return a `u8` value that is out of bounds for `self`.
                pub fn get(self) -> u8 {
                    self.0 & Self::MASK
                }
            }

            impl TryFrom<u8> for $name {
                type Error = OutOfBoundsLaneId;

                fn try_from(lane: u8) -> Result<Self, Self::Error> {
                    if lane > Self::MASK {
                        return Err(OutOfBoundsLaneId)
                    }
                    Ok(Self(lane))
                }
            }
        )*
    };
}
impl_imm_lane_id! {
    /// A byte with values in the range 0–1 identifying a lane.
    struct ImmLaneIdx2(x 2);
    /// A byte with values in the range 0–3 identifying a lane.
    struct ImmLaneIdx4(x 4);
    /// A byte with values in the range 0–7 identifying a lane.
    struct ImmLaneIdx8(x 8);
    /// A byte with values in the range 0–15 identifying a lane.
    struct ImmLaneIdx16(x 16);
    /// A byte with values in the range 0–31 identifying a lane.
    struct ImmLaneIdx32(x 32);
}

trait IntoLanes {
    type Lanes: Lanes<Item = Self, LaneIdx = Self::LaneIdx>;
    type LaneIdx;
}

trait Lanes {
    type Item;
    type LaneIdx;

    const LANES: usize;

    const ALL_ONES: Self::Item;
    const ALL_ZEROS: Self::Item;

    fn from_v128(value: V128) -> Self;
    fn into_v128(self) -> V128;

    fn splat(value: Self::Item) -> Self;
    fn extract_lane(self, lane: Self::LaneIdx) -> Self::Item;
    fn replace_lane(self, lane: Self::LaneIdx, item: Self::Item) -> Self;
    fn lanewise_unary(self, f: impl Fn(Self::Item) -> Self::Item) -> Self;
    fn lanewise_binary(self, other: Self, f: impl Fn(Self::Item, Self::Item) -> Self::Item)
        -> Self;
    fn lanewise_comparison(self, other: Self, f: impl Fn(Self::Item, Self::Item) -> bool) -> Self;
}

macro_rules! impl_lanes_for {
    ( $( struct $name:ident([$ty:ty; $n:literal]); )* ) => {
        $(
            #[derive(Copy, Clone)]
            #[repr(transparent)]
            struct $name([$ty; $n]);

            impl IntoLanes for $ty {
                type Lanes = $name;
                type LaneIdx = <[(); $n] as IntoLaneIdx>::LaneIdx;
            }

            impl Lanes for $name {
                type Item = $ty;
                type LaneIdx = <[(); $n] as IntoLaneIdx>::LaneIdx;

                const LANES: usize = $n;
                const ALL_ONES: Self::Item = <$ty>::from_le_bytes([0xFF_u8; 16 / $n]);
                const ALL_ZEROS: Self::Item = <$ty>::from_le_bytes([0x00_u8; 16 / $n]);

                fn from_v128(value: V128) -> Self {
                    // SAFETY: the types chosen to implement `Split` are always
                    //         of same size as `V128` and have no invalid bit-patterns.
                    Self(unsafe { ::core::mem::transmute::<V128, [$ty; $n]>(value) })
                }

                fn into_v128(self) -> V128 {
                    // SAFETY: the types chosen to implement `Combine` are always
                    //         of same size as `V128` and have no invalid bit-patterns.
                    unsafe { ::core::mem::transmute::<[$ty; $n], V128>(self.0) }
                }

                fn splat(value: Self::Item) -> Self {
                    Self([value; $n])
                }

                fn extract_lane(self, lane: Self::LaneIdx) -> Self::Item {
                    self.0[lane.get() as usize]
                }

                fn replace_lane(self, lane: Self::LaneIdx, item: Self::Item) -> Self {
                    let mut this = self;
                    this.0[lane.get() as usize] = item;
                    this
                }

                fn lanewise_unary(self, f: impl Fn(Self::Item) -> Self::Item) -> Self {
                    let mut this = self.0;
                    for i in 0..Self::LANES {
                        this[i] = f(this[i]);
                    }
                    Self(this)
                }

                fn lanewise_binary(self, other: Self, f: impl Fn(Self::Item, Self::Item) -> Self::Item) -> Self {
                    let mut lhs = self.0;
                    let rhs = other.0;
                    for i in 0..Self::LANES {
                        lhs[i] = f(lhs[i], rhs[i]);
                    }
                    Self(lhs)
                }

                fn lanewise_comparison(self, other: Self, f: impl Fn(Self::Item, Self::Item) -> bool) -> Self {
                    self.lanewise_binary(other, |lhs, rhs| match f(lhs, rhs) {
                        true => Self::ALL_ONES,
                        false => Self::ALL_ZEROS,
                    })
                }
            }
        )*
    };
}
impl_lanes_for! {
    struct I64x2([i64; 2]);
    struct I32x4([i32; 4]);
    struct I16x8([i16; 8]);
    struct I8x16([i8; 16]);
    struct F32x4([f32; 4]);
    struct F64x2([f64; 2]);
}

trait LanewiseWidening: Lanes {
    type Narrow: Lanes;

    fn lanewise_widening_unary(
        value: Self::Narrow,
        f: impl Fn(<Self::Narrow as Lanes>::Item, <Self::Narrow as Lanes>::Item) -> Self::Item,
    ) -> Self;

    fn lanewise_widening_binary(
        lhs: Self::Narrow,
        rhs: Self::Narrow,
        f: impl Fn([<Self::Narrow as Lanes>::Item; 2], [<Self::Narrow as Lanes>::Item; 2]) -> Self::Item,
    ) -> Self;
}

impl LanewiseWidening for I64x2 {
    type Narrow = I32x4;

    fn lanewise_widening_unary(
        value: Self::Narrow,
        f: impl Fn(<Self::Narrow as Lanes>::Item, <Self::Narrow as Lanes>::Item) -> Self::Item,
    ) -> Self {
        let a = value.0;
        #[rustfmt::skip]
        let result = [
            f(a[0], a[1]),
            f(a[2], a[3]),
        ];
        Self(result)
    }

    fn lanewise_widening_binary(
        lhs: Self::Narrow,
        rhs: Self::Narrow,
        f: impl Fn([<Self::Narrow as Lanes>::Item; 2], [<Self::Narrow as Lanes>::Item; 2]) -> Self::Item,
    ) -> Self {
        let a = lhs.0;
        let b = rhs.0;
        #[rustfmt::skip]
        let result = [
            f([a[0], a[1]], [b[0], b[1]]),
            f([a[2], a[3]], [b[2], b[3]]),
        ];
        Self(result)
    }
}

impl LanewiseWidening for I32x4 {
    type Narrow = I16x8;

    fn lanewise_widening_unary(
        value: Self::Narrow,
        f: impl Fn(<Self::Narrow as Lanes>::Item, <Self::Narrow as Lanes>::Item) -> Self::Item,
    ) -> Self {
        let a = value.0;
        #[rustfmt::skip]
        let result = [
            f(a[0], a[1]),
            f(a[2], a[3]),
            f(a[4], a[5]),
            f(a[6], a[7]),
        ];
        Self(result)
    }

    fn lanewise_widening_binary(
        lhs: Self::Narrow,
        rhs: Self::Narrow,
        f: impl Fn([<Self::Narrow as Lanes>::Item; 2], [<Self::Narrow as Lanes>::Item; 2]) -> Self::Item,
    ) -> Self {
        let a = lhs.0;
        let b = rhs.0;
        #[rustfmt::skip]
        let result = [
            f([a[0], a[1]], [b[0], b[1]]),
            f([a[2], a[3]], [b[2], b[3]]),
            f([a[4], a[5]], [b[4], b[5]]),
            f([a[6], a[7]], [b[6], b[7]]),
        ];
        Self(result)
    }
}

impl LanewiseWidening for I16x8 {
    type Narrow = I8x16;

    fn lanewise_widening_unary(
        value: Self::Narrow,
        f: impl Fn(<Self::Narrow as Lanes>::Item, <Self::Narrow as Lanes>::Item) -> Self::Item,
    ) -> Self {
        let a = value.0;
        #[rustfmt::skip]
        let result = [
            f(a[ 0], a[ 1]),
            f(a[ 2], a[ 3]),
            f(a[ 4], a[ 5]),
            f(a[ 6], a[ 7]),
            f(a[ 8], a[ 9]),
            f(a[10], a[11]),
            f(a[12], a[13]),
            f(a[14], a[15]),
        ];
        Self(result)
    }

    fn lanewise_widening_binary(
        lhs: Self::Narrow,
        rhs: Self::Narrow,
        f: impl Fn([<Self::Narrow as Lanes>::Item; 2], [<Self::Narrow as Lanes>::Item; 2]) -> Self::Item,
    ) -> Self {
        let a = lhs.0;
        let b = rhs.0;
        #[rustfmt::skip]
        let result = [
            f([a[ 0], a[ 1]], [b[ 0], b[ 1]]),
            f([a[ 2], a[ 3]], [b[ 2], b[ 3]]),
            f([a[ 4], a[ 5]], [b[ 4], b[ 5]]),
            f([a[ 6], a[ 7]], [b[ 6], b[ 7]]),
            f([a[ 8], a[ 9]], [b[ 8], b[ 9]]),
            f([a[10], a[11]], [b[10], b[11]]),
            f([a[12], a[13]], [b[12], b[13]]),
            f([a[14], a[15]], [b[14], b[15]]),
        ];
        Self(result)
    }
}

trait LanewiseNarrowing: Lanes {
    type Wide: Lanes;

    fn lanewise_narrowing_unary(
        value: Self::Wide,
        f: impl Fn(<Self::Wide as Lanes>::Item) -> [Self::Item; 2],
    ) -> Self;

    fn lanewise_narrowing_binary(
        lhs: Self::Wide,
        rhs: Self::Wide,
        f: impl Fn(<Self::Wide as Lanes>::Item, <Self::Wide as Lanes>::Item) -> [Self::Item; 2],
    ) -> Self;
}

impl LanewiseNarrowing for I32x4 {
    type Wide = I64x2;

    fn lanewise_narrowing_unary(
        value: Self::Wide,
        f: impl Fn(<Self::Wide as Lanes>::Item) -> [Self::Item; 2],
    ) -> Self {
        let w = value.0;
        let [a0, a1] = f(w[0]);
        let [b0, b1] = f(w[1]);
        Self([a0, a1, b0, b1])
    }

    fn lanewise_narrowing_binary(
        lhs: Self::Wide,
        rhs: Self::Wide,
        f: impl Fn(<Self::Wide as Lanes>::Item, <Self::Wide as Lanes>::Item) -> [Self::Item; 2],
    ) -> Self {
        let lhs = lhs.0;
        let rhs = rhs.0;
        let [a0, a1] = f(lhs[0], rhs[0]);
        let [b0, b1] = f(lhs[1], rhs[1]);
        Self([a0, a1, b0, b1])
    }
}

impl LanewiseNarrowing for I16x8 {
    type Wide = I32x4;

    fn lanewise_narrowing_unary(
        value: Self::Wide,
        f: impl Fn(<Self::Wide as Lanes>::Item) -> [Self::Item; 2],
    ) -> Self {
        let w = value.0;
        let [a0, a1] = f(w[0]);
        let [b0, b1] = f(w[1]);
        let [c0, c1] = f(w[2]);
        let [d0, d1] = f(w[3]);
        Self([a0, a1, b0, b1, c0, c1, d0, d1])
    }

    fn lanewise_narrowing_binary(
        lhs: Self::Wide,
        rhs: Self::Wide,
        f: impl Fn(<Self::Wide as Lanes>::Item, <Self::Wide as Lanes>::Item) -> [Self::Item; 2],
    ) -> Self {
        let lhs = lhs.0;
        let rhs = rhs.0;
        let [a0, a1] = f(lhs[0], rhs[0]);
        let [b0, b1] = f(lhs[1], rhs[1]);
        let [c0, c1] = f(lhs[2], rhs[2]);
        let [d0, d1] = f(lhs[3], rhs[3]);
        Self([a0, a1, b0, b1, c0, c1, d0, d1])
    }
}

impl LanewiseNarrowing for I8x16 {
    type Wide = I16x8;

    fn lanewise_narrowing_unary(
        value: Self::Wide,
        f: impl Fn(<Self::Wide as Lanes>::Item) -> [Self::Item; 2],
    ) -> Self {
        let w = value.0;
        let [a0, a1] = f(w[0]);
        let [b0, b1] = f(w[1]);
        let [c0, c1] = f(w[2]);
        let [d0, d1] = f(w[3]);
        let [e0, e1] = f(w[4]);
        let [f0, f1] = f(w[5]);
        let [g0, g1] = f(w[6]);
        let [h0, h1] = f(w[7]);
        Self([
            a0, a1, b0, b1, c0, c1, d0, d1, e0, e1, f0, f1, g0, g1, h0, h1,
        ])
    }

    fn lanewise_narrowing_binary(
        lhs: Self::Wide,
        rhs: Self::Wide,
        f: impl Fn(<Self::Wide as Lanes>::Item, <Self::Wide as Lanes>::Item) -> [Self::Item; 2],
    ) -> Self {
        let lhs = lhs.0;
        let rhs = rhs.0;
        let [a0, a1] = f(lhs[0], rhs[0]);
        let [b0, b1] = f(lhs[1], rhs[1]);
        let [c0, c1] = f(lhs[2], rhs[2]);
        let [d0, d1] = f(lhs[3], rhs[3]);
        let [e0, e1] = f(lhs[4], rhs[4]);
        let [f0, f1] = f(lhs[5], rhs[5]);
        let [g0, g1] = f(lhs[6], rhs[6]);
        let [h0, h1] = f(lhs[7], rhs[7]);
        Self([
            a0, a1, b0, b1, c0, c1, d0, d1, e0, e1, f0, f1, g0, g1, h0, h1,
        ])
    }
}

impl V128 {
    fn splat<T: IntoLanes>(value: T) -> Self {
        <<T as IntoLanes>::Lanes>::splat(value).into_v128()
    }

    fn extract_lane<T: IntoLanes>(self, lane: <T as IntoLanes>::LaneIdx) -> T {
        <<T as IntoLanes>::Lanes>::from_v128(self).extract_lane(lane)
    }

    fn replace_lane<T: IntoLanes>(self, lane: <T as IntoLanes>::LaneIdx, item: T) -> Self {
        <<T as IntoLanes>::Lanes>::from_v128(self)
            .replace_lane(lane, item)
            .into_v128()
    }

    fn lanewise_unary<T: IntoLanes>(v128: Self, f: impl Fn(T) -> T) -> Self {
        <<T as IntoLanes>::Lanes>::from_v128(v128)
            .lanewise_unary(f)
            .into_v128()
    }

    fn lanewise_binary<T: IntoLanes>(lhs: Self, rhs: Self, f: impl Fn(T, T) -> T) -> Self {
        let lhs = <<T as IntoLanes>::Lanes>::from_v128(lhs);
        let rhs = <<T as IntoLanes>::Lanes>::from_v128(rhs);
        lhs.lanewise_binary(rhs, f).into_v128()
    }

    fn lanewise_comparison<T: IntoLanes>(lhs: Self, rhs: Self, f: impl Fn(T, T) -> bool) -> Self {
        let lhs = <<T as IntoLanes>::Lanes>::from_v128(lhs);
        let rhs = <<T as IntoLanes>::Lanes>::from_v128(rhs);
        lhs.lanewise_comparison(rhs, f).into_v128()
    }

    fn lanewise_widening_unary<T: LanewiseWidening>(
        self,
        f: impl Fn(<T::Narrow as Lanes>::Item, <T::Narrow as Lanes>::Item) -> T::Item,
    ) -> Self {
        T::lanewise_widening_unary(<T::Narrow as Lanes>::from_v128(self), f).into_v128()
    }

    fn lanewise_widening_binary<T: LanewiseWidening>(
        self,
        rhs: Self,
        f: impl Fn([<T::Narrow as Lanes>::Item; 2], [<T::Narrow as Lanes>::Item; 2]) -> T::Item,
    ) -> Self {
        T::lanewise_widening_binary(
            <T::Narrow as Lanes>::from_v128(self),
            <T::Narrow as Lanes>::from_v128(rhs),
            f,
        )
        .into_v128()
    }

    fn lanewise_narrowing_unary<T: LanewiseNarrowing>(
        self,
        f: impl Fn(<T::Wide as Lanes>::Item) -> [T::Item; 2],
    ) -> Self {
        T::lanewise_narrowing_unary(<T::Wide as Lanes>::from_v128(self), f).into_v128()
    }

    fn lanewise_narrowing_binary<T: LanewiseNarrowing>(
        self,
        rhs: Self,
        f: impl Fn(<T::Wide as Lanes>::Item, <T::Wide as Lanes>::Item) -> [T::Item; 2],
    ) -> Self {
        T::lanewise_narrowing_binary(
            <T::Wide as Lanes>::from_v128(self),
            <T::Wide as Lanes>::from_v128(rhs),
            f,
        )
        .into_v128()
    }
}
