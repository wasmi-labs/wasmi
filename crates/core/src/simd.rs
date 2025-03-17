#![expect(dead_code)] // TODO: remove silencing of warnings again

use crate::{wasm, ReadAs, UntypedVal, WriteAs};
use core::ops::{BitAnd, BitOr, BitXor, Neg, Not};

/// The Wasm `simd` proposal's `v128` type.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct V128([u8; 16]);

impl From<[u8; 16]> for V128 {
    fn from(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

impl From<i128> for V128 {
    fn from(value: i128) -> Self {
        Self(value.to_le_bytes())
    }
}

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

/// An error that may occur when constructing an out of bounds lane index.
pub struct OutOfBoundsLaneId;

/// Helper trait to help the type inference to do its jobs with fewer type annotations.
trait IntoLaneIdx {
    /// The associated lane index type.
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

/// Helper trait to help the type inference to do its jobs with fewer type annotations.
///
/// # Note
///
/// - This trait and its applications are hidden from outside this module.
/// - For example `i32` is associated to the `i32x4` lane type.
trait IntoLanes {
    /// The `Lanes` type associated to the implementing type.
    type Lanes: Lanes<Item = Self, LaneIdx = Self::LaneIdx>;
    /// The `LaneIdx` type associated to the implementing type.
    type LaneIdx;
}

/// Implemented by `Lanes` types.
///
/// Possible `Lanes` types include:
///
/// - `I64x2`
/// - `I32x4`
/// - `I16x8`
/// - `I8x16`
/// - `F64x2`
/// - `F32x4`
trait Lanes {
    /// The type used in the lanes. E.g. `i32` for `i32x4`.
    type Item;
    /// The associated lane index type. E.g. `ImmLaneIdx4` for `i32x4`.
    type LaneIdx;

    /// The number of lanes for `Self`.
    const LANES: usize;

    /// A lane item where all bits are `1`.
    const ALL_ONES: Self::Item;

    /// A lane item where all bits are `0`.
    const ALL_ZEROS: Self::Item;

    /// Converts the [`V128`] to `Self`.
    fn from_v128(value: V128) -> Self;

    /// Converts `self` to a [`V128`] value.
    fn into_v128(self) -> V128;

    /// Creates `Self` by splatting `value`.
    fn splat(value: Self::Item) -> Self;

    /// Extract the item at `lane` from `self`.
    fn extract_lane(self, lane: Self::LaneIdx) -> Self::Item;

    /// Replace the item at `lane` with `item` and return `self` afterwards.
    fn replace_lane(self, lane: Self::LaneIdx, item: Self::Item) -> Self;

    /// Apply `f` for all lane items in `self`.
    fn lanewise_unary(self, f: impl Fn(Self::Item) -> Self::Item) -> Self;

    /// Apply `f` for all pairs of lane items in `self` and `other`.
    fn lanewise_binary(self, other: Self, f: impl Fn(Self::Item, Self::Item) -> Self::Item)
        -> Self;

    /// Apply `f` comparison for all pairs of lane items in `self` and `other`.
    ///
    /// Storing [`Self::ALL_ONES`] if `f` evaluates to `true` or [`Self::ALL_ZEROS`] otherwise per item.
    fn lanewise_comparison(self, other: Self, f: impl Fn(Self::Item, Self::Item) -> bool) -> Self;
}

macro_rules! impl_lanes_for {
    (
        $(
            $( #[$attr:meta] )*
            struct $name:ident([$ty:ty; $n:literal]);
        )*
    ) => {
        $(
            $( #[$attr] )*
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
                    //
                    // Note: it is important to state that this could be implemented
                    //       in safe Rust entirely. However, during development it turned
                    //       out that _not_ using unsafe transmutation confused the
                    //       optimizer enough that optimizations became very flaky.
                    //       This was tested across a variety of compiler versions.
                    Self(unsafe { ::core::mem::transmute::<V128, [$ty; $n]>(value) })
                }

                fn into_v128(self) -> V128 {
                    // SAFETY: the types chosen to implement `Combine` are always
                    //         of same size as `V128` and have no invalid bit-patterns.
                    //
                    // Note: see note from `from_v128` method above.
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
    /// The Wasm `i64x2` vector type consisting of 2 `i64` values.
    struct I64x2([i64; 2]);
    /// The Wasm `u64x2` vector type consisting of 2 `u64` values.
    struct U64x2([u64; 2]);
    /// The Wasm `i32x4` vector type consisting of 4 `i32` values.
    struct I32x4([i32; 4]);
    /// The Wasm `u32x4` vector type consisting of 4 `u32` values.
    struct U32x4([u32; 4]);
    /// The Wasm `i16x8` vector type consisting of 8 `i16` values.
    struct I16x8([i16; 8]);
    /// The Wasm `u16x8` vector type consisting of 8 `u16` values.
    struct U16x8([u16; 8]);
    /// The Wasm `i8x16` vector type consisting of 16 `i8` values.
    struct I8x16([i8; 16]);
    /// The Wasm `u8x16` vector type consisting of 16 `u8` values.
    struct U8x16([u8; 16]);
    /// The Wasm `f32x4` vector type consisting of 4 `f32` values.
    struct F32x4([f32; 4]);
    /// The Wasm `f64x2` vector type consisting of 2 `f64` values.
    struct F64x2([f64; 2]);
}

/// Helper trait to help the type inference to do its jobs with fewer type annotations.
trait IntoLanewiseWidening: IntoLanes {
    /// The wide lanes type.
    type Wide: LanewiseWidening<Item = Self::WideItem, Narrow = <Self as IntoLanes>::Lanes>;
    /// The wide lanes' item type.
    type WideItem;
}

/// Trait allowing [`Lanes`] types to be widened.
///
/// # Example
///
/// - This allows a single `i32x4` value to be widened to a `i64x2`.
/// - This allows a pair of `i32x4` values to be widened to a `i64x2`.
trait LanewiseWidening: Lanes {
    /// The narrow [`Lanes`] type to be widened.
    type Narrow: Lanes;

    /// Widen `value` to `Self` by applying `f` for all pairs of lane items.
    fn lanewise_widening_unary(
        value: Self::Narrow,
        f: impl Fn(<Self::Narrow as Lanes>::Item, <Self::Narrow as Lanes>::Item) -> Self::Item,
    ) -> Self;

    /// Widen `lhs` and `rhs` to `Self` by applying `f` for all pairs of lane items lanewise.
    fn lanewise_widening_binary(
        lhs: Self::Narrow,
        rhs: Self::Narrow,
        f: impl Fn([<Self::Narrow as Lanes>::Item; 2], [<Self::Narrow as Lanes>::Item; 2]) -> Self::Item,
    ) -> Self;
}

macro_rules! impl_lanewise_widening_for_i64x2 {
    (
        $( impl LanewiseWidening for $ty:ident<Narrow = $narrow_ty:ty>; )*
    ) => {
        $(
            impl IntoLanewiseWidening for <$narrow_ty as Lanes>::Item {
                type Wide = $ty;
                type WideItem = <$ty as Lanes>::Item;
            }

            impl LanewiseWidening for $ty {
                type Narrow = $narrow_ty;

                fn lanewise_widening_unary(
                    value: Self::Narrow,
                    f: impl Fn(
                        <Self::Narrow as Lanes>::Item,
                        <Self::Narrow as Lanes>::Item,
                    ) -> Self::Item,
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
                    f: impl Fn(
                        [<Self::Narrow as Lanes>::Item; 2],
                        [<Self::Narrow as Lanes>::Item; 2],
                    ) -> Self::Item,
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
        )*
    };
}
impl_lanewise_widening_for_i64x2! {
    impl LanewiseWidening for I64x2<Narrow = I32x4>;
    impl LanewiseWidening for U64x2<Narrow = U32x4>;
}

macro_rules! impl_lanewise_widening_for_i32x4 {
    (
        $( impl LanewiseWidening for $ty:ident<Narrow = $narrow_ty:ty>; )*
    ) => {
        $(
            impl IntoLanewiseWidening for <$narrow_ty as Lanes>::Item {
                type Wide = $ty;
                type WideItem = <$ty as Lanes>::Item;
            }

            impl LanewiseWidening for $ty {
                type Narrow = $narrow_ty;

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
                    f: impl Fn(
                        [<Self::Narrow as Lanes>::Item; 2],
                        [<Self::Narrow as Lanes>::Item; 2],
                    ) -> Self::Item,
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
        )*
    };
}
impl_lanewise_widening_for_i32x4! {
    impl LanewiseWidening for I32x4<Narrow = I16x8>;
    impl LanewiseWidening for U32x4<Narrow = U16x8>;
}

macro_rules! impl_lanewise_widening_for_i16x8 {
    (
        $( impl LanewiseWidening for $ty:ident<Narrow = $narrow_ty:ty>; )*
    ) => {
        $(
            impl IntoLanewiseWidening for <$narrow_ty as Lanes>::Item {
                type Wide = $ty;
                type WideItem = <$ty as Lanes>::Item;
            }

            impl LanewiseWidening for $ty {
                type Narrow = $narrow_ty;

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
                    f: impl Fn(
                        [<Self::Narrow as Lanes>::Item; 2],
                        [<Self::Narrow as Lanes>::Item; 2],
                    ) -> Self::Item,
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
        )*
    };
}
impl_lanewise_widening_for_i16x8! {
    impl LanewiseWidening for I16x8<Narrow = I8x16>;
    impl LanewiseWidening for U16x8<Narrow = U8x16>;
}

/// Trait allowing [`Lanes`] types to be narrowed.
///
/// # Example
///
/// - This allows a single `i64x2` value to be narrowed into a `i32x4`.
/// - This allows a pair of `i64x2` values to be narrowed into a `i32x4`.
trait LanewiseNarrowing: Lanes {
    /// The wide [`Lanes`] type to be narrowed.
    type Wide: Lanes;

    /// Widen `value` to `Self` by applying `f` for all pairs of lane items.
    fn lanewise_narrowing_unary(
        value: Self::Wide,
        f: impl Fn(<Self::Wide as Lanes>::Item) -> [Self::Item; 2],
    ) -> Self;

    /// Widen `lhs` and `rhs` to `Self` by applying `f` for all pairs of lane items lanewise.
    fn lanewise_narrowing_binary(
        lhs: Self::Wide,
        rhs: Self::Wide,
        f: impl Fn(<Self::Wide as Lanes>::Item, <Self::Wide as Lanes>::Item) -> [Self::Item; 2],
    ) -> Self;
}

macro_rules! impl_lanewise_narrowing_for_i32x4 {
    (
        $( impl LanewiseNarrowing for $ty:ident<Wide = $wide_ty:ty>; )*
    ) => {
        $(
            impl LanewiseNarrowing for $ty {
                type Wide = $wide_ty;

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
                    f: impl Fn(
                        <Self::Wide as Lanes>::Item,
                        <Self::Wide as Lanes>::Item,
                    ) -> [Self::Item; 2],
                ) -> Self {
                    let lhs = lhs.0;
                    let rhs = rhs.0;
                    let [a0, a1] = f(lhs[0], rhs[0]);
                    let [b0, b1] = f(lhs[1], rhs[1]);
                    Self([a0, a1, b0, b1])
                }
            }
        )*
    };
}
impl_lanewise_narrowing_for_i32x4! {
    impl LanewiseNarrowing for I32x4<Wide = I64x2>;
    impl LanewiseNarrowing for U32x4<Wide = U64x2>;
}

macro_rules! impl_lanewise_narrowing_for_i16x8 {
    (
        $( impl LanewiseNarrowing for $ty:ident<Wide = $wide_ty:ty>; )*
    ) => {
        $(
            impl LanewiseNarrowing for $ty {
                type Wide = $wide_ty;

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
        )*
    };
}
impl_lanewise_narrowing_for_i16x8! {
    impl LanewiseNarrowing for I16x8<Wide = I32x4>;
    impl LanewiseNarrowing for U16x8<Wide = U32x4>;
}

macro_rules! impl_lanewise_narrowing_for_i16x8 {
    (
        $( impl LanewiseNarrowing for $ty:ident<Wide = $wide_ty:ty>; )*
    ) => {
        $(
            impl LanewiseNarrowing for $ty {
                type Wide = $wide_ty;

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
        )*
    };
}
impl_lanewise_narrowing_for_i16x8! {
    impl LanewiseNarrowing for I8x16<Wide = I16x8>;
    impl LanewiseNarrowing for U8x16<Wide = U16x8>;
}

impl V128 {
    /// Convenience method to help implement splatting methods.
    fn splat<T: IntoLanes>(value: T) -> Self {
        <<T as IntoLanes>::Lanes>::splat(value).into_v128()
    }

    /// Convenience method to help implement lane extraction methods.
    fn extract_lane<T: IntoLanes>(self, lane: <T as IntoLanes>::LaneIdx) -> T {
        <<T as IntoLanes>::Lanes>::from_v128(self).extract_lane(lane)
    }

    /// Convenience method to help implement lane replacement methods.
    fn replace_lane<T: IntoLanes>(self, lane: <T as IntoLanes>::LaneIdx, item: T) -> Self {
        <<T as IntoLanes>::Lanes>::from_v128(self)
            .replace_lane(lane, item)
            .into_v128()
    }

    /// Convenience method to help implement lanewise unary methods.
    fn lanewise_unary<T: IntoLanes>(self, f: impl Fn(T) -> T) -> Self {
        <<T as IntoLanes>::Lanes>::from_v128(self)
            .lanewise_unary(f)
            .into_v128()
    }

    /// Convenience method to help implement lanewise binary methods.
    fn lanewise_binary<T: IntoLanes>(lhs: Self, rhs: Self, f: impl Fn(T, T) -> T) -> Self {
        let lhs = <<T as IntoLanes>::Lanes>::from_v128(lhs);
        let rhs = <<T as IntoLanes>::Lanes>::from_v128(rhs);
        lhs.lanewise_binary(rhs, f).into_v128()
    }

    /// Convenience method to help implement lanewise comparison methods.
    fn lanewise_comparison<T: IntoLanes>(lhs: Self, rhs: Self, f: impl Fn(T, T) -> bool) -> Self {
        let lhs = <<T as IntoLanes>::Lanes>::from_v128(lhs);
        let rhs = <<T as IntoLanes>::Lanes>::from_v128(rhs);
        lhs.lanewise_comparison(rhs, f).into_v128()
    }

    /// Convenience method to help implement lanewise unary widening methods.
    fn lanewise_widening_unary<T: IntoLanewiseWidening>(
        self,
        f: impl Fn(T, T) -> <T as IntoLanewiseWidening>::WideItem,
    ) -> Self {
        <<T as IntoLanewiseWidening>::Wide>::lanewise_widening_unary(
            <T as IntoLanes>::Lanes::from_v128(self),
            f,
        )
        .into_v128()
    }

    /// Convenience method to help implement lanewise binary widening methods.
    fn lanewise_widening_binary<T: IntoLanewiseWidening>(
        lhs: Self,
        rhs: Self,
        f: impl Fn([T; 2], [T; 2]) -> <T as IntoLanewiseWidening>::WideItem,
    ) -> Self {
        <<T as IntoLanewiseWidening>::Wide>::lanewise_widening_binary(
            <T as IntoLanes>::Lanes::from_v128(lhs),
            <T as IntoLanes>::Lanes::from_v128(rhs),
            f,
        )
        .into_v128()
    }

    /// Convenience method to help implement lanewise unary narrowing methods.
    fn lanewise_narrowing_unary<T: LanewiseNarrowing>(
        self,
        f: impl Fn(<T::Wide as Lanes>::Item) -> [T::Item; 2],
    ) -> Self {
        T::lanewise_narrowing_unary(<T::Wide as Lanes>::from_v128(self), f).into_v128()
    }

    /// Convenience method to help implement lanewise binary narrowing methods.
    fn lanewise_narrowing_binary<T: LanewiseNarrowing>(
        lhs: Self,
        rhs: Self,
        f: impl Fn(<T::Wide as Lanes>::Item, <T::Wide as Lanes>::Item) -> [T::Item; 2],
    ) -> Self {
        T::lanewise_narrowing_binary(
            <T::Wide as Lanes>::from_v128(lhs),
            <T::Wide as Lanes>::from_v128(rhs),
            f,
        )
        .into_v128()
    }
}

/// Concenience identity helper function.
fn identity<T>(x: T) -> T {
    x
}

macro_rules! impl_splat_for {
    ( $( fn $name:ident(value: $ty:ty) -> Self; )* ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(value: $ty) -> Self {
                Self::splat(value)
            }
        )*
    };
}
impl V128 {
    impl_splat_for! {
        fn i64x2_splat(value: i64) -> Self;
        fn i32x4_splat(value: i32) -> Self;
        fn i16x8_splat(value: i16) -> Self;
        fn i8x16_splat(value: i8) -> Self;
        fn f32x4_splat(value: f32) -> Self;
        fn f64x2_splat(value: f64) -> Self;
    }
}

macro_rules! impl_extract_for {
    ( $( fn $name:ident(self, lane: $lane_ty:ty) -> $ret_ty:ty = $convert:expr; )* ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(self, lane: $lane_ty) -> $ret_ty {
                ($convert)(self.extract_lane(lane))
            }
        )*
    };
}
impl V128 {
    impl_extract_for! {
        fn i64x2_extract_lane(self, lane: ImmLaneIdx2) -> i64 = identity;
        fn i32x4_extract_lane(self, lane: ImmLaneIdx4) -> i32 = identity;
        fn f64x2_extract_lane(self, lane: ImmLaneIdx2) -> f64 = identity;
        fn f32x4_extract_lane(self, lane: ImmLaneIdx4) -> f32 = identity;
        fn i8x16_extract_lane_s(self, lane: ImmLaneIdx16) -> i32 = <i8 as Into<_>>::into;
        fn i8x16_extract_lane_u(self, lane: ImmLaneIdx16) -> u32 = <u8 as Into<_>>::into;
        fn i16x8_extract_lane_s(self, lane: ImmLaneIdx8) -> i32 = <i16 as Into<_>>::into;
        fn i16x8_extract_lane_u(self, lane: ImmLaneIdx8) -> u32 = <u16 as Into<_>>::into;
    }
}

macro_rules! impl_replace_for {
    ( $( fn $name:ident(self, lane: $lane_ty:ty, item: $item_ty:ty) -> Self; )* ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(self, lane: $lane_ty, item: $item_ty) -> Self {
                self.replace_lane(lane, item)
            }
        )*
    };
}
impl V128 {
    impl_replace_for! {
        fn i64x2_replace_lane(self, lane: ImmLaneIdx2, item: i64) -> Self;
        fn i32x4_replace_lane(self, lane: ImmLaneIdx4, item: i32) -> Self;
        fn i16x8_replace_lane(self, lane: ImmLaneIdx8, item: i16) -> Self;
        fn i8x16_replace_lane(self, lane: ImmLaneIdx16, item: i8) -> Self;
        fn f64x2_replace_lane(self, lane: ImmLaneIdx2, item: f64) -> Self;
        fn f32x4_replace_lane(self, lane: ImmLaneIdx4, item: f32) -> Self;
    }
}

macro_rules! impl_unary_for {
    ( $( fn $name:ident(self) -> Self = $lanewise_expr:expr; )* ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(self) -> Self {
                Self::lanewise_unary(self, $lanewise_expr)
            }
        )*
    };
}

/// Lanewise operation for the Wasm `q15mulr_sat` SIMD operation.
fn i16x8_q15mulr_sat(x: i16, y: i16) -> i16 {
    (x * y + 0x4000) >> 15
}

macro_rules! impl_binary_for {
    ( $( fn $name:ident(lhs: Self, rhs: Self) -> Self = $lanewise_expr:expr; )* ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(lhs: Self, rhs: Self) -> Self {
                Self::lanewise_binary(lhs, rhs, $lanewise_expr)
            }
        )*
    };
}
impl V128 {
    impl_binary_for! {
        fn i64x2_add(lhs: Self, rhs: Self) -> Self = i64::wrapping_add;
        fn i32x4_add(lhs: Self, rhs: Self) -> Self = i32::wrapping_add;
        fn i16x8_add(lhs: Self, rhs: Self) -> Self = i16::wrapping_add;
        fn i8x16_add(lhs: Self, rhs: Self) -> Self = i8::wrapping_add;

        fn i64x2_sub(lhs: Self, rhs: Self) -> Self = i64::wrapping_sub;
        fn i32x4_sub(lhs: Self, rhs: Self) -> Self = i32::wrapping_sub;
        fn i16x8_sub(lhs: Self, rhs: Self) -> Self = i16::wrapping_sub;
        fn i8x16_sub(lhs: Self, rhs: Self) -> Self = i8::wrapping_sub;

        fn i64x2_mul(lhs: Self, rhs: Self) -> Self = i64::wrapping_mul;
        fn i32x4_mul(lhs: Self, rhs: Self) -> Self = i32::wrapping_mul;
        fn i16x8_mul(lhs: Self, rhs: Self) -> Self = i16::wrapping_mul;
        fn i8x16_mul(lhs: Self, rhs: Self) -> Self = i8::wrapping_mul;

        fn i8x16_add_sat_s(lhs: Self, rhs: Self) -> Self = i8::saturating_add;
        fn i8x16_add_sat_u(lhs: Self, rhs: Self) -> Self = u8::saturating_add;
        fn i16x8_add_sat_s(lhs: Self, rhs: Self) -> Self = i16::saturating_add;
        fn i16x8_add_sat_u(lhs: Self, rhs: Self) -> Self = u16::saturating_add;
        fn i8x16_sub_sat_s(lhs: Self, rhs: Self) -> Self = i8::saturating_sub;
        fn i8x16_sub_sat_u(lhs: Self, rhs: Self) -> Self = u8::saturating_sub;
        fn i16x8_sub_sat_s(lhs: Self, rhs: Self) -> Self = i16::saturating_sub;
        fn i16x8_sub_sat_u(lhs: Self, rhs: Self) -> Self = u16::saturating_sub;

        fn i16x8_q15mulr_sat_s(lhs: Self, rhs: Self) -> Self = i16x8_q15mulr_sat;

        fn i8x16_min_s(lhs: Self, rhs: Self) -> Self = i8::min;
        fn i8x16_min_u(lhs: Self, rhs: Self) -> Self = u8::min;
        fn i16x8_min_s(lhs: Self, rhs: Self) -> Self = i16::min;
        fn i16x8_min_u(lhs: Self, rhs: Self) -> Self = u16::min;
        fn i32x4_min_s(lhs: Self, rhs: Self) -> Self = i32::min;
        fn i32x4_min_u(lhs: Self, rhs: Self) -> Self = u32::min;
        fn i8x16_max_s(lhs: Self, rhs: Self) -> Self = i8::max;
        fn i8x16_max_u(lhs: Self, rhs: Self) -> Self = u8::max;
        fn i16x8_max_s(lhs: Self, rhs: Self) -> Self = i16::max;
        fn i16x8_max_u(lhs: Self, rhs: Self) -> Self = u16::max;
        fn i32x4_max_s(lhs: Self, rhs: Self) -> Self = i32::max;
        fn i32x4_max_u(lhs: Self, rhs: Self) -> Self = u32::max;

        fn i8x16_avgr_u(lhs: Self, rhs: Self) -> Self = |a: u8, b: u8| (a + b + 1) / 2;
        fn i16x8_avgr_u(lhs: Self, rhs: Self) -> Self = |a: u16, b: u16| (a + b + 1) / 2;

        fn v128_and(lhs: Self, rhs: Self) -> Self = <i64 as BitAnd>::bitand;
        fn v128_or(lhs: Self, rhs: Self) -> Self = <i64 as BitOr>::bitor;
        fn v128_xor(lhs: Self, rhs: Self) -> Self = <i64 as BitXor>::bitxor;
        fn v128_andnot(lhs: Self, rhs: Self) -> Self = |a: i64, b: i64| a & !b;
    }

    impl_unary_for! {
        fn i64x2_neg(self) -> Self = <i64 as Neg>::neg;
        fn i32x4_neg(self) -> Self = <i32 as Neg>::neg;
        fn i16x8_neg(self) -> Self = <i16 as Neg>::neg;
        fn i8x16_neg(self) -> Self = <i8 as Neg>::neg;

        fn i8x16_abs(self) -> Self = i8::abs;
        fn i16x8_abs(self) -> Self = i16::abs;
        fn i32x4_abs(self) -> Self = i32::abs;
        fn i64x2_abs(self) -> Self = i64::abs;

        fn v128_not(self) -> Self = <i64 as Not>::not;
    }
}

macro_rules! impl_extmul_ops {
    (
        $(
            (
                $narrow:ty => $wide:ty;
                fn $extmul_low:ident;
                fn $extmul_high:ident;
            )
        ),* $(,)?
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($extmul_low), "` instruction.")]
            pub fn $extmul_low(lhs: Self, rhs: Self) -> Self {
                fn extmul(a: [$narrow; 2], b: [$narrow; 2]) -> $wide {
                    let a = <$wide>::from(a[0]);
                    let b = <$wide>::from(b[0]);
                    a.wrapping_mul(b)
                }
                Self::lanewise_widening_binary(lhs, rhs, extmul)
            }

            #[doc = concat!("Executes a Wasm `", stringify!($extmul_high), "` instruction.")]
            pub fn $extmul_high(lhs: Self, rhs: Self) -> Self {
                fn extmul(a: [$narrow; 2], b: [$narrow; 2]) -> $wide {
                    let a = <$wide>::from(a[1]);
                    let b = <$wide>::from(b[1]);
                    a.wrapping_mul(b)
                }
                Self::lanewise_widening_binary(lhs, rhs, extmul)
            }
        )*
    };
}
impl V128 {
    impl_extmul_ops! {
        (
            i8 => i16;
            fn i16x8_extmul_low_i8x16_s;
            fn i16x8_extmul_high_i8x16_s;
        ),
        (
            u8 => u16;
            fn i16x8_extmul_low_i8x16_u;
            fn i16x8_extmul_high_i8x16_u;
        ),
        (
            i16 => i32;
            fn i32x4_extmul_low_i16x8_s;
            fn i32x4_extmul_high_i16x8_s;
        ),
        (
            u16 => u32;
            fn i32x4_extmul_low_i16x8_u;
            fn i32x4_extmul_high_i16x8_u;
        ),
        (
            i32 => i64;
            fn i64x2_extmul_low_i32x4_s;
            fn i64x2_extmul_high_i32x4_s;
        ),
        (
            u32 => u64;
            fn i64x2_extmul_low_i32x4_u;
            fn i64x2_extmul_high_i32x4_u;
        ),
    }
}

macro_rules! impl_extadd_pairwise {
    (
        $( fn $name:ident($narrow:ty) -> $wide:ty; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(self) -> Self {
                fn extadd_pairwise(a: $narrow, b: $narrow) -> $wide {
                    let a = <$wide>::from(a);
                    let b = <$wide>::from(b);
                    a.wrapping_add(b)
                }
                self.lanewise_widening_unary(extadd_pairwise)
            }
        )*
    };
}
impl V128 {
    impl_extadd_pairwise! {
        fn i16x8_extadd_pairwise_i8x16_s(i8) -> i16;
        fn i16x8_extadd_pairwise_i8x16_u(i8) -> i16;
        fn i32x4_extadd_pairwise_i16x8_s(i16) -> i32;
        fn i32x4_extadd_pairwise_i16x8_u(i16) -> i32;
    }
}

macro_rules! impl_shift_ops {
    (
        $( fn $name:ident(self, rhs: u32) -> Self = $lanewise_expr:expr; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(self, rhs: u32) -> Self {
                self.lanewise_unary(|v| $lanewise_expr(v, rhs))
            }
        )*
    };
}
impl V128 {
    impl_shift_ops! {
        fn i8x16_shl(self, rhs: u32) -> Self = i8::wrapping_shl;
        fn i16x8_shl(self, rhs: u32) -> Self = i16::wrapping_shl;
        fn i32x4_shl(self, rhs: u32) -> Self = i32::wrapping_shl;
        fn i64x2_shl(self, rhs: u32) -> Self = i64::wrapping_shl;
        fn i8x16_shr_s(self, rhs: u32) -> Self = i8::wrapping_shr;
        fn i8x16_shr_u(self, rhs: u32) -> Self = u8::wrapping_shr;
        fn i16x8_shr_s(self, rhs: u32) -> Self = i16::wrapping_shr;
        fn i16x8_shr_u(self, rhs: u32) -> Self = u16::wrapping_shr;
        fn i32x4_shr_s(self, rhs: u32) -> Self = i32::wrapping_shr;
        fn i32x4_shr_u(self, rhs: u32) -> Self = u32::wrapping_shr;
        fn i64x2_shr_s(self, rhs: u32) -> Self = i64::wrapping_shr;
        fn i64x2_shr_u(self, rhs: u32) -> Self = u64::wrapping_shr;
    }
}

impl V128 {
    /// Execute a Wasm `v128.bitselect` instruction.
    pub fn v128_bitselect(v1: Self, v2: Self, c: Self) -> Self {
        Self::v128_or(Self::v128_and(v1, c), Self::v128_andnot(v2, c))
    }
}
