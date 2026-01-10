//! Defines the entire Wasm `simd` proposal API.

use crate::{
    TrapCode,
    V128,
    memory::{self, ExtendInto},
    simd,
    value::Float,
    wasm,
};
use core::{
    array,
    ops::{BitAnd, BitOr, BitXor, Neg, Not},
};

macro_rules! op {
    ($ty:ty, $op:tt) => {{
        |lhs: $ty, rhs: $ty| lhs $op rhs
    }};
}

/// An error that may occur when constructing an out of bounds lane index.
pub struct OutOfBoundsLaneIdx;

/// Helper trait used to infer the [`ImmLaneIdx`] from a given primitive.
pub trait IntoLaneIdx {
    /// The associated lane index type.
    type LaneIdx: Sized + Copy + TryFrom<u8, Error = OutOfBoundsLaneIdx> + Into<u8>;
}

macro_rules! impl_into_lane_idx {
    (
        $( impl IntoLaneIdx for $ty:ty = $lane_idx:ty; )*
    ) => {
        $(
            impl IntoLaneIdx for $ty {
                type LaneIdx = $lane_idx;
            }
        )*
    };
}
impl_into_lane_idx! {
    impl IntoLaneIdx for i8 = ImmLaneIdx<16>;
    impl IntoLaneIdx for u8 = ImmLaneIdx<16>;
    impl IntoLaneIdx for i16 = ImmLaneIdx<8>;
    impl IntoLaneIdx for u16 = ImmLaneIdx<8>;
    impl IntoLaneIdx for i32 = ImmLaneIdx<4>;
    impl IntoLaneIdx for u32 = ImmLaneIdx<4>;
    impl IntoLaneIdx for f32 = ImmLaneIdx<4>;
    impl IntoLaneIdx for i64 = ImmLaneIdx<2>;
    impl IntoLaneIdx for u64 = ImmLaneIdx<2>;
    impl IntoLaneIdx for f64 = ImmLaneIdx<2>;
}

/// A byte with values in the range 0–N identifying a lane.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ImmLaneIdx<const N: u8>(u8);

impl<const N: u8> ImmLaneIdx<N> {
    /// Helper bit mask for construction and getter.
    const MASK: u8 = (1_u8 << u8::ilog2(N)) - 1;

    fn zero() -> Self {
        Self(0)
    }
}

impl<const N: u8> From<ImmLaneIdx<N>> for u8 {
    fn from(lane: ImmLaneIdx<N>) -> u8 {
        lane.0 & <ImmLaneIdx<N>>::MASK
    }
}

impl<const N: u8> TryFrom<u8> for ImmLaneIdx<N> {
    type Error = OutOfBoundsLaneIdx;

    fn try_from(lane: u8) -> Result<Self, Self::Error> {
        if lane > Self::MASK {
            return Err(OutOfBoundsLaneIdx);
        }
        Ok(Self(lane))
    }
}

/// A byte with values in the range 0–1 identifying a lane.
pub type ImmLaneIdx2 = ImmLaneIdx<2>;
/// A byte with values in the range 0–3 identifying a lane.
pub type ImmLaneIdx4 = ImmLaneIdx<4>;
/// A byte with values in the range 0–7 identifying a lane.
pub type ImmLaneIdx8 = ImmLaneIdx<8>;
/// A byte with values in the range 0–15 identifying a lane.
pub type ImmLaneIdx16 = ImmLaneIdx<16>;
/// A byte with values in the range 0–31 identifying a lane.
pub type ImmLaneIdx32 = ImmLaneIdx<32>;

/// Internal helper trait to help the type inference to do its jobs with fewer type annotations.
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

/// Internal helper trait implemented by `Lanes` types.
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

    /// Apply `f` for all triplets of lane items in `self` and `other`.
    fn lanewise_ternary(
        self,
        b: Self,
        c: Self,
        f: impl Fn(Self::Item, Self::Item, Self::Item) -> Self::Item,
    ) -> Self;

    /// Apply `f` comparison for all pairs of lane items in `self` and `other`.
    ///
    /// Storing [`Self::ALL_ONES`] if `f` evaluates to `true` or [`Self::ALL_ZEROS`] otherwise per item.
    fn lanewise_comparison(self, other: Self, f: impl Fn(Self::Item, Self::Item) -> bool) -> Self;

    /// Apply `f(i, n, acc)` for all lane items `i` at pos `n` in `self` and return the result.
    fn lanewise_reduce<T>(self, acc: T, f: impl Fn(u8, Self::Item, T) -> T) -> T;
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
                type LaneIdx = ImmLaneIdx<$n>;
            }

            impl From<[$ty; $n]> for $name {
                fn from(array: [$ty; $n]) -> Self {
                    Self(array)
                }
            }

            impl Lanes for $name {
                type Item = $ty;
                type LaneIdx = ImmLaneIdx<$n>;

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
                    self.0[u8::from(lane) as usize]
                }

                fn replace_lane(self, lane: Self::LaneIdx, item: Self::Item) -> Self {
                    let mut this = self;
                    this.0[u8::from(lane) as usize] = item;
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

                fn lanewise_ternary(self, b: Self, c: Self, f: impl Fn(Self::Item, Self::Item, Self::Item) -> Self::Item) -> Self {
                    let mut a = self.0;
                    let b = b.0;
                    let c = c.0;
                    for i in 0..Self::LANES {
                        a[i] = f(a[i], b[i], c[i]);
                    }
                    Self(a)
                }

                fn lanewise_comparison(self, other: Self, f: impl Fn(Self::Item, Self::Item) -> bool) -> Self {
                    self.lanewise_binary(other, |lhs, rhs| match f(lhs, rhs) {
                        true => Self::ALL_ONES,
                        false => Self::ALL_ZEROS,
                    })
                }

                fn lanewise_reduce<T>(self, acc: T, f: impl Fn(u8, Self::Item, T) -> T) -> T {
                    let this = self.0;
                    let mut acc = acc;
                    for i in 0..Self::LANES {
                        acc = f(i as u8, this[i], acc);
                    }
                    acc
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

/// `Self` can be constructed from the narrower lanes.
///
/// For example a `i64x2` vector can be constructed from the two lower lanes of a `i32x4`.
trait FromNarrow<NarrowLanes: Lanes>: Lanes {
    /// Construct `Self` from the pairwise application of `f` of items in `narrow`.
    fn pairwise_unary(
        narrow: NarrowLanes,
        f: impl Fn(NarrowLanes::Item, NarrowLanes::Item) -> Self::Item,
    ) -> Self;

    /// Construct `Self` from the pairwise application of `f` of items in `lhs` and `rhs`.
    fn pairwise_binary(
        lhs: NarrowLanes,
        rhs: NarrowLanes,
        f: impl Fn([NarrowLanes::Item; 2], [NarrowLanes::Item; 2]) -> Self::Item,
    ) -> Self;

    /// Construct `Self` from the application of `f` to the lower half lanes of `narrow`.
    fn low_unary(narrow: NarrowLanes, f: impl Fn(NarrowLanes::Item) -> Self::Item) -> Self;

    /// Construct `Self` from the application of `f` to the higher half lanes of `narrow`.
    fn high_unary(narrow: NarrowLanes, f: impl Fn(NarrowLanes::Item) -> Self::Item) -> Self;

    /// Construct `Self` from the binary application of `f` to the lower half lanes of `narrow_lhs` and `narrow_rhs`.
    fn low_binary(
        narrow_lhs: NarrowLanes,
        narrow_rhs: NarrowLanes,
        f: impl Fn(NarrowLanes::Item, NarrowLanes::Item) -> Self::Item,
    ) -> Self;

    /// Construct `Self` from the binary application of `f` to the higher half lanes of `narrow_lhs` and `narrow_rhs`.
    fn high_binary(
        narrow_lhs: NarrowLanes,
        narrow_rhs: NarrowLanes,
        f: impl Fn(NarrowLanes::Item, NarrowLanes::Item) -> Self::Item,
    ) -> Self;
}

macro_rules! impl_from_narrow_for {
    ( $( impl FromNarrow<$narrow_ty:ty> for $self_ty:ty; )* ) => {
        $(
            impl FromNarrow<$narrow_ty> for $self_ty {
                fn pairwise_unary(
                    narrow: $narrow_ty,
                    f: impl Fn(<$narrow_ty as Lanes>::Item, <$narrow_ty as Lanes>::Item) -> Self::Item,
                ) -> Self {
                    let narrow = narrow.0;
                    Self(array::from_fn(|i| f(narrow[2 * i], narrow[2 * i + 1])))
                }

                fn pairwise_binary(
                    lhs: $narrow_ty,
                    rhs: $narrow_ty,
                    f: impl Fn([<$narrow_ty as Lanes>::Item; 2], [<$narrow_ty as Lanes>::Item; 2]) -> Self::Item,
                ) -> Self {
                    let lhs = lhs.0;
                    let rhs = rhs.0;
                    Self(array::from_fn(|i| {
                        f(
                            [lhs[2 * i], lhs[2 * i + 1]],
                            [rhs[2 * i], rhs[2 * i + 1]],
                        )
                    }))
                }

                fn low_unary(narrow: $narrow_ty, f: impl Fn(<$narrow_ty as Lanes>::Item) -> Self::Item) -> Self {
                    Self(array::from_fn(|i| f(narrow.0[i])))
                }

                fn high_unary(narrow: $narrow_ty, f: impl Fn(<$narrow_ty as Lanes>::Item) -> Self::Item) -> Self {
                    Self(array::from_fn(|i| f(narrow.0[i + Self::LANES])))
                }

                fn low_binary(
                    narrow_lhs: $narrow_ty,
                    narrow_rhs: $narrow_ty,
                    f: impl Fn(<$narrow_ty as Lanes>::Item, <$narrow_ty as Lanes>::Item) -> Self::Item,
                ) -> Self {
                    let narrow_lhs = narrow_lhs.0;
                    let narrow_rhs = narrow_rhs.0;
                    Self(array::from_fn(|i| f(narrow_lhs[i], narrow_rhs[i])))
                }

                fn high_binary(
                    narrow_lhs: $narrow_ty,
                    narrow_rhs: $narrow_ty,
                    f: impl Fn(<$narrow_ty as Lanes>::Item, <$narrow_ty as Lanes>::Item) -> Self::Item,
                ) -> Self {
                    let narrow_lhs = narrow_lhs.0;
                    let narrow_rhs = narrow_rhs.0;
                    Self(array::from_fn(|i| {
                        f(narrow_lhs[i + Self::LANES], narrow_rhs[i + Self::LANES])
                    }))
                }
            }
        )*
    };
}
impl_from_narrow_for! {
    impl FromNarrow<I32x4> for I64x2;
    impl FromNarrow<U32x4> for U64x2;
    impl FromNarrow<I16x8> for I32x4;
    impl FromNarrow<U16x8> for U32x4;
    impl FromNarrow<I8x16> for I16x8;
    impl FromNarrow<U8x16> for U16x8;
    impl FromNarrow<F32x4> for I64x2;
    impl FromNarrow<F32x4> for U64x2;
    impl FromNarrow<I32x4> for F64x2;
    impl FromNarrow<U32x4> for F64x2;
    impl FromNarrow<F32x4> for F64x2;
}

/// `Self` can be constructed from the wider lanes.
///
/// For example a `i32x4` vector can be constructed from a `i64x2`.
trait FromWide<WideLanes: Lanes>: Lanes {
    /// Construct `Self` from the application of `f` to the wide `low` and `high` items.
    fn from_low_high(
        low: WideLanes,
        high: WideLanes,
        f: impl Fn(WideLanes::Item) -> Self::Item,
    ) -> Self;

    /// Construct `Self` from the application of `f` to the wide `low` or evaluate `high`.
    fn from_low_or(
        low: WideLanes,
        high: impl Fn() -> Self::Item,
        f: impl Fn(WideLanes::Item) -> Self::Item,
    ) -> Self;
}

macro_rules! impl_from_wide_for {
    (
        $( impl FromWide<$wide_ty:ty> for $narrow_ty:ty; )*
    ) => {
        $(
            impl FromWide<$wide_ty> for $narrow_ty {
                fn from_low_high(
                    low: $wide_ty,
                    high: $wide_ty,
                    f: impl Fn(<$wide_ty as Lanes>::Item) -> Self::Item,
                ) -> Self {
                    let low = low.0;
                    let high = high.0;
                    Self(array::from_fn(|i| {
                        match i < <$wide_ty as Lanes>::LANES {
                            true => f(low[i]),
                            false => f(high[i - <$wide_ty as Lanes>::LANES]),
                        }
                    }))
                }

                fn from_low_or(
                    low: $wide_ty,
                    high: impl Fn() -> Self::Item,
                    f: impl Fn(<$wide_ty as Lanes>::Item) -> Self::Item,
                ) -> Self {
                    let low = low.0;
                    Self(array::from_fn(|i| {
                        match i < <$wide_ty as Lanes>::LANES {
                            true => f(low[i]),
                            false => high(),
                        }
                    }))
                }
            }
        )*
    };
}
impl_from_wide_for! {
    impl FromWide<F64x2> for I32x4;
    impl FromWide<F64x2> for U32x4;
    impl FromWide<F64x2> for F32x4;
    impl FromWide<I32x4> for I16x8;
    impl FromWide<U32x4> for U16x8;
    impl FromWide<I16x8> for I8x16;
    impl FromWide<U16x8> for U8x16;
}

trait ReinterpretAs<T> {
    fn reinterpret_as(self) -> T;
}

macro_rules! impl_reinterpret_as_for {
    ( $ty0:ty, $ty1:ty ) => {
        impl ReinterpretAs<$ty0> for $ty1 {
            fn reinterpret_as(self) -> $ty0 {
                <$ty0>::from_ne_bytes(self.to_ne_bytes())
            }
        }

        impl ReinterpretAs<$ty1> for $ty0 {
            fn reinterpret_as(self) -> $ty1 {
                <$ty1>::from_ne_bytes(self.to_ne_bytes())
            }
        }
    };
}
impl_reinterpret_as_for!(i32, f32);
impl_reinterpret_as_for!(u32, f32);
impl_reinterpret_as_for!(i64, f64);
impl_reinterpret_as_for!(u64, f64);

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

    /// Convenience method to help implement lanewise unary cast methods.
    fn lanewise_unary_cast<T: IntoLanes, U>(self, f: impl Fn(T) -> U) -> Self
    where
        U: ReinterpretAs<T>,
    {
        <<T as IntoLanes>::Lanes>::from_v128(self)
            .lanewise_unary(|v| f(v).reinterpret_as())
            .into_v128()
    }

    /// Convenience method to help implement lanewise binary methods.
    fn lanewise_binary<T: IntoLanes>(lhs: Self, rhs: Self, f: impl Fn(T, T) -> T) -> Self {
        let lhs = <<T as IntoLanes>::Lanes>::from_v128(lhs);
        let rhs = <<T as IntoLanes>::Lanes>::from_v128(rhs);
        lhs.lanewise_binary(rhs, f).into_v128()
    }

    /// Convenience method to help implement lanewise ternary methods.
    fn lanewise_ternary<T: IntoLanes>(a: Self, b: Self, c: Self, f: impl Fn(T, T, T) -> T) -> Self {
        let a = <<T as IntoLanes>::Lanes>::from_v128(a);
        let b = <<T as IntoLanes>::Lanes>::from_v128(b);
        let c = <<T as IntoLanes>::Lanes>::from_v128(c);
        a.lanewise_ternary(b, c, f).into_v128()
    }

    /// Convenience method to help implement lanewise comparison methods.
    fn lanewise_comparison<T: IntoLanes>(lhs: Self, rhs: Self, f: impl Fn(T, T) -> bool) -> Self {
        let lhs = <<T as IntoLanes>::Lanes>::from_v128(lhs);
        let rhs = <<T as IntoLanes>::Lanes>::from_v128(rhs);
        lhs.lanewise_comparison(rhs, f).into_v128()
    }

    /// Convenience method to help implement lanewise reduce methods.
    fn lanewise_reduce<T: IntoLanes, V>(self, acc: V, f: impl Fn(T, V) -> V) -> V {
        self.lanewise_reduce_enumerate::<T, V>(acc, |_, v: T, acc: V| f(v, acc))
    }

    /// Convenience method to help implement lanewise reduce methods with a loop-index.
    fn lanewise_reduce_enumerate<T: IntoLanes, V>(self, acc: V, f: impl Fn(u8, T, V) -> V) -> V {
        <<T as IntoLanes>::Lanes>::from_v128(self).lanewise_reduce(acc, f)
    }

    /// Convenience method to help implement pairwise unary methods.
    fn pairwise_unary<Narrow: IntoLanes, Wide: IntoLanes>(
        self,
        f: impl Fn(Narrow, Narrow) -> Wide,
    ) -> Self
    where
        <Wide as IntoLanes>::Lanes: FromNarrow<<Narrow as IntoLanes>::Lanes>,
    {
        <<Wide as IntoLanes>::Lanes as FromNarrow<<Narrow as IntoLanes>::Lanes>>::pairwise_unary(
            <<Narrow as IntoLanes>::Lanes>::from_v128(self),
            f,
        )
        .into_v128()
    }

    /// Convenience method to help implement pairwise binary methods.
    fn pairwise_binary<Narrow: IntoLanes, Wide: IntoLanes>(
        lhs: Self,
        rhs: Self,
        f: impl Fn([Narrow; 2], [Narrow; 2]) -> Wide,
    ) -> Self
    where
        <Wide as IntoLanes>::Lanes: FromNarrow<<Narrow as IntoLanes>::Lanes>,
    {
        <<Wide as IntoLanes>::Lanes as FromNarrow<<Narrow as IntoLanes>::Lanes>>::pairwise_binary(
            <<Narrow as IntoLanes>::Lanes>::from_v128(lhs),
            <<Narrow as IntoLanes>::Lanes>::from_v128(rhs),
            f,
        )
        .into_v128()
    }

    /// Convenience method to help implement extend-low unary methods.
    fn low_unary<Narrow: IntoLanes, Wide: IntoLanes>(self, f: impl Fn(Narrow) -> Wide) -> Self
    where
        <Wide as IntoLanes>::Lanes: FromNarrow<<Narrow as IntoLanes>::Lanes>,
    {
        <<Wide as IntoLanes>::Lanes as FromNarrow<<Narrow as IntoLanes>::Lanes>>::low_unary(
            <<Narrow as IntoLanes>::Lanes>::from_v128(self),
            f,
        )
        .into_v128()
    }

    /// Convenience method to help implement extend-high unary methods.
    fn high_unary<Narrow: IntoLanes, Wide: IntoLanes>(self, f: impl Fn(Narrow) -> Wide) -> Self
    where
        <Wide as IntoLanes>::Lanes: FromNarrow<<Narrow as IntoLanes>::Lanes>,
    {
        <<Wide as IntoLanes>::Lanes as FromNarrow<<Narrow as IntoLanes>::Lanes>>::high_unary(
            <<Narrow as IntoLanes>::Lanes>::from_v128(self),
            f,
        )
        .into_v128()
    }

    /// Convenience method to help implement extend-low binary methods.
    fn from_low_binary<Narrow: IntoLanes, Wide: IntoLanes>(
        lhs: Self,
        rhs: Self,
        f: impl Fn(Narrow, Narrow) -> Wide,
    ) -> Self
    where
        <Wide as IntoLanes>::Lanes: FromNarrow<<Narrow as IntoLanes>::Lanes>,
    {
        <<Wide as IntoLanes>::Lanes as FromNarrow<<Narrow as IntoLanes>::Lanes>>::low_binary(
            <<Narrow as IntoLanes>::Lanes>::from_v128(lhs),
            <<Narrow as IntoLanes>::Lanes>::from_v128(rhs),
            f,
        )
        .into_v128()
    }

    /// Convenience method to help implement extend-high binary methods.
    fn from_high_binary<Narrow: IntoLanes, Wide: IntoLanes>(
        lhs: Self,
        rhs: Self,
        f: impl Fn(Narrow, Narrow) -> Wide,
    ) -> Self
    where
        <Wide as IntoLanes>::Lanes: FromNarrow<<Narrow as IntoLanes>::Lanes>,
    {
        <<Wide as IntoLanes>::Lanes as FromNarrow<<Narrow as IntoLanes>::Lanes>>::high_binary(
            <<Narrow as IntoLanes>::Lanes>::from_v128(lhs),
            <<Narrow as IntoLanes>::Lanes>::from_v128(rhs),
            f,
        )
        .into_v128()
    }

    /// Convenience method to help implement narrowing low-high methods.
    fn from_low_high<Narrow: IntoLanes, Wide: IntoLanes>(
        lhs: Self,
        rhs: Self,
        f: impl Fn(Wide) -> Narrow,
    ) -> Self
    where
        <Narrow as IntoLanes>::Lanes: FromWide<<Wide as IntoLanes>::Lanes>,
    {
        <<Narrow as IntoLanes>::Lanes as FromWide<<Wide as IntoLanes>::Lanes>>::from_low_high(
            <<Wide as IntoLanes>::Lanes>::from_v128(lhs),
            <<Wide as IntoLanes>::Lanes>::from_v128(rhs),
            f,
        )
        .into_v128()
    }

    /// Convenience method to help implement narrowing low-or methods.
    fn low_or<Narrow: IntoLanes, Wide: IntoLanes>(
        self,
        high: impl Fn() -> Narrow,
        f: impl Fn(Wide) -> Narrow,
    ) -> Self
    where
        <Narrow as IntoLanes>::Lanes: FromWide<<Wide as IntoLanes>::Lanes>,
    {
        <<Narrow as IntoLanes>::Lanes as FromWide<<Wide as IntoLanes>::Lanes>>::from_low_or(
            <<Wide as IntoLanes>::Lanes>::from_v128(self),
            high,
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
    ( $( fn $name:ident(value: $ty:ty) -> V128; )* ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(value: $ty) -> V128 {
                V128::splat(value)
            }
        )*
    };
}
impl_splat_for! {
    fn i64x2_splat(value: i64) -> V128;
    fn i32x4_splat(value: i32) -> V128;
    fn i16x8_splat(value: i16) -> V128;
    fn i8x16_splat(value: i8) -> V128;
    fn f32x4_splat(value: f32) -> V128;
    fn f64x2_splat(value: f64) -> V128;
}

macro_rules! impl_extract_for {
    ( $( fn $name:ident(v128: V128, lane: $lane_ty:ty) -> $ret_ty:ty = $convert:expr; )* ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(v128: V128, lane: $lane_ty) -> $ret_ty {
                ($convert)(v128.extract_lane(lane))
            }
        )*
    };
}
impl_extract_for! {
    fn i64x2_extract_lane(v128: V128, lane: ImmLaneIdx2) -> i64 = identity;
    fn i32x4_extract_lane(v128: V128, lane: ImmLaneIdx4) -> i32 = identity;
    fn f64x2_extract_lane(v128: V128, lane: ImmLaneIdx2) -> f64 = identity;
    fn f32x4_extract_lane(v128: V128, lane: ImmLaneIdx4) -> f32 = identity;
    fn i8x16_extract_lane_s(v128: V128, lane: ImmLaneIdx16) -> i32 = <i8 as Into<_>>::into;
    fn i8x16_extract_lane_u(v128: V128, lane: ImmLaneIdx16) -> u32 = <u8 as Into<_>>::into;
    fn i16x8_extract_lane_s(v128: V128, lane: ImmLaneIdx8) -> i32 = <i16 as Into<_>>::into;
    fn i16x8_extract_lane_u(v128: V128, lane: ImmLaneIdx8) -> u32 = <u16 as Into<_>>::into;
}

macro_rules! impl_replace_for {
    ( $( fn $name:ident(v128: V128, lane: $lane_ty:ty, item: $item_ty:ty) -> V128; )* ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(v128: V128, lane: $lane_ty, item: $item_ty) -> V128 {
                v128.replace_lane(lane, item)
            }
        )*
    };
}
impl_replace_for! {
    fn i64x2_replace_lane(v128: V128, lane: ImmLaneIdx2, item: i64) -> V128;
    fn i32x4_replace_lane(v128: V128, lane: ImmLaneIdx4, item: i32) -> V128;
    fn i16x8_replace_lane(v128: V128, lane: ImmLaneIdx8, item: i16) -> V128;
    fn i8x16_replace_lane(v128: V128, lane: ImmLaneIdx16, item: i8) -> V128;
    fn f64x2_replace_lane(v128: V128, lane: ImmLaneIdx2, item: f64) -> V128;
    fn f32x4_replace_lane(v128: V128, lane: ImmLaneIdx4, item: f32) -> V128;
}

/// Executes a Wasm `i8x16.shuffle` instruction.
pub fn i8x16_shuffle(a: V128, b: V128, s: [ImmLaneIdx32; 16]) -> V128 {
    let a = I8x16::from_v128(a).0;
    let b = I8x16::from_v128(b).0;
    I8x16(array::from_fn(|i| match usize::from(u8::from(s[i])) {
        i @ 0..16 => a[i],
        i => b[i - 16],
    }))
    .into_v128()
}

/// Executes a Wasm `i8x16.swizzle` instruction.
pub fn i8x16_swizzle(a: V128, s: V128) -> V128 {
    let a = U8x16::from_v128(a).0;
    let s = U8x16::from_v128(s).0;
    U8x16(array::from_fn(|i| match usize::from(s[i]) {
        i @ 0..16 => a[i],
        _ => 0,
    }))
    .into_v128()
}

macro_rules! impl_unary_for {
    ( $( fn $name:ident(v128: V128) -> V128 = $lanewise_expr:expr; )* ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(v128: V128) -> V128 {
                V128::lanewise_unary(v128, $lanewise_expr)
            }
        )*
    };
}

macro_rules! impl_unary_cast_for {
    ( $( fn $name:ident(v128: V128) -> V128 = $lanewise_expr:expr; )* ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(v128: V128) -> V128 {
                V128::lanewise_unary_cast(v128, $lanewise_expr)
            }
        )*
    };
}

/// Lanewise operation for the Wasm `q15mulr_sat` SIMD operation.
fn i16x8_q15mulr_sat(x: i16, y: i16) -> i16 {
    const MIN: i32 = i16::MIN as i32;
    const MAX: i32 = i16::MAX as i32;
    let x = i32::from(x);
    let y = i32::from(y);
    let q15mulr = (x * y + (1 << 14)) >> 15;
    q15mulr.clamp(MIN, MAX) as i16
}

macro_rules! avgr {
    ($ty:ty as $wide_ty:ty) => {{
        |a: $ty, b: $ty| {
            let a = <$wide_ty as ::core::convert::From<$ty>>::from(a);
            let b = <$wide_ty as ::core::convert::From<$ty>>::from(b);
            a.wrapping_add(b).div_ceil(2) as $ty
        }
    }};
}

/// Wasm SIMD `pmin` (pseudo-min) definition.
fn pmin<T: PartialOrd>(lhs: T, rhs: T) -> T {
    if rhs < lhs { rhs } else { lhs }
}

/// Wasm SIMD `pmax` (pseudo-max) definition.
fn pmax<T: PartialOrd>(lhs: T, rhs: T) -> T {
    if lhs < rhs { rhs } else { lhs }
}

macro_rules! impl_binary_for {
    ( $( fn $name:ident(lhs: V128, rhs: V128) -> V128 = $lanewise_expr:expr; )* ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(lhs: V128, rhs: V128) -> V128 {
                V128::lanewise_binary(lhs, rhs, $lanewise_expr)
            }
        )*
    };
}
impl_binary_for! {
    fn i64x2_add(lhs: V128, rhs: V128) -> V128 = i64::wrapping_add;
    fn i32x4_add(lhs: V128, rhs: V128) -> V128 = i32::wrapping_add;
    fn i16x8_add(lhs: V128, rhs: V128) -> V128 = i16::wrapping_add;
    fn i8x16_add(lhs: V128, rhs: V128) -> V128 = i8::wrapping_add;

    fn i64x2_sub(lhs: V128, rhs: V128) -> V128 = i64::wrapping_sub;
    fn i32x4_sub(lhs: V128, rhs: V128) -> V128 = i32::wrapping_sub;
    fn i16x8_sub(lhs: V128, rhs: V128) -> V128 = i16::wrapping_sub;
    fn i8x16_sub(lhs: V128, rhs: V128) -> V128 = i8::wrapping_sub;

    fn i64x2_mul(lhs: V128, rhs: V128) -> V128 = i64::wrapping_mul;
    fn i32x4_mul(lhs: V128, rhs: V128) -> V128 = i32::wrapping_mul;
    fn i16x8_mul(lhs: V128, rhs: V128) -> V128 = i16::wrapping_mul;
    fn i8x16_mul(lhs: V128, rhs: V128) -> V128 = i8::wrapping_mul;

    fn i8x16_add_sat_s(lhs: V128, rhs: V128) -> V128 = i8::saturating_add;
    fn i8x16_add_sat_u(lhs: V128, rhs: V128) -> V128 = u8::saturating_add;
    fn i16x8_add_sat_s(lhs: V128, rhs: V128) -> V128 = i16::saturating_add;
    fn i16x8_add_sat_u(lhs: V128, rhs: V128) -> V128 = u16::saturating_add;
    fn i8x16_sub_sat_s(lhs: V128, rhs: V128) -> V128 = i8::saturating_sub;
    fn i8x16_sub_sat_u(lhs: V128, rhs: V128) -> V128 = u8::saturating_sub;
    fn i16x8_sub_sat_s(lhs: V128, rhs: V128) -> V128 = i16::saturating_sub;
    fn i16x8_sub_sat_u(lhs: V128, rhs: V128) -> V128 = u16::saturating_sub;

    fn i16x8_q15mulr_sat_s(lhs: V128, rhs: V128) -> V128 = i16x8_q15mulr_sat;

    fn i8x16_min_s(lhs: V128, rhs: V128) -> V128 = i8::min;
    fn i8x16_min_u(lhs: V128, rhs: V128) -> V128 = u8::min;
    fn i16x8_min_s(lhs: V128, rhs: V128) -> V128 = i16::min;
    fn i16x8_min_u(lhs: V128, rhs: V128) -> V128 = u16::min;
    fn i32x4_min_s(lhs: V128, rhs: V128) -> V128 = i32::min;
    fn i32x4_min_u(lhs: V128, rhs: V128) -> V128 = u32::min;
    fn i8x16_max_s(lhs: V128, rhs: V128) -> V128 = i8::max;
    fn i8x16_max_u(lhs: V128, rhs: V128) -> V128 = u8::max;
    fn i16x8_max_s(lhs: V128, rhs: V128) -> V128 = i16::max;
    fn i16x8_max_u(lhs: V128, rhs: V128) -> V128 = u16::max;
    fn i32x4_max_s(lhs: V128, rhs: V128) -> V128 = i32::max;
    fn i32x4_max_u(lhs: V128, rhs: V128) -> V128 = u32::max;

    fn i8x16_avgr_u(lhs: V128, rhs: V128) -> V128 = avgr!(u8 as u16);
    fn i16x8_avgr_u(lhs: V128, rhs: V128) -> V128 = avgr!(u16 as u32);

    fn v128_and(lhs: V128, rhs: V128) -> V128 = <u64 as BitAnd>::bitand;
    fn v128_or(lhs: V128, rhs: V128) -> V128 = <u64 as BitOr>::bitor;
    fn v128_xor(lhs: V128, rhs: V128) -> V128 = <u64 as BitXor>::bitxor;
    fn v128_andnot(lhs: V128, rhs: V128) -> V128 = |a: u64, b: u64| a & !b;

    fn f32x4_min(lhs: V128, rhs: V128) -> V128 = wasm::f32_min;
    fn f64x2_min(lhs: V128, rhs: V128) -> V128 = wasm::f64_min;
    fn f32x4_max(lhs: V128, rhs: V128) -> V128 = wasm::f32_max;
    fn f64x2_max(lhs: V128, rhs: V128) -> V128 = wasm::f64_max;
    fn f32x4_pmin(lhs: V128, rhs: V128) -> V128 = pmin::<f32>;
    fn f64x2_pmin(lhs: V128, rhs: V128) -> V128 = pmin::<f64>;
    fn f32x4_pmax(lhs: V128, rhs: V128) -> V128 = pmax::<f32>;
    fn f64x2_pmax(lhs: V128, rhs: V128) -> V128 = pmax::<f64>;
    fn f32x4_add(lhs: V128, rhs: V128) -> V128 = op!(f32, +);
    fn f64x2_add(lhs: V128, rhs: V128) -> V128 = op!(f64, +);
    fn f32x4_sub(lhs: V128, rhs: V128) -> V128 = op!(f32, -);
    fn f64x2_sub(lhs: V128, rhs: V128) -> V128 = op!(f64, -);
    fn f32x4_div(lhs: V128, rhs: V128) -> V128 = op!(f32, /);
    fn f64x2_div(lhs: V128, rhs: V128) -> V128 = op!(f64, /);
    fn f32x4_mul(lhs: V128, rhs: V128) -> V128 = op!(f32, *);
    fn f64x2_mul(lhs: V128, rhs: V128) -> V128 = op!(f64, *);
}

impl_unary_for! {
    fn i64x2_neg(v128: V128) -> V128 = i64::wrapping_neg;
    fn i32x4_neg(v128: V128) -> V128 = i32::wrapping_neg;
    fn i16x8_neg(v128: V128) -> V128 = i16::wrapping_neg;
    fn i8x16_neg(v128: V128) -> V128 = i8::wrapping_neg;

    fn i8x16_abs(v128: V128) -> V128 = i8::wrapping_abs;
    fn i16x8_abs(v128: V128) -> V128 = i16::wrapping_abs;
    fn i32x4_abs(v128: V128) -> V128 = i32::wrapping_abs;
    fn i64x2_abs(v128: V128) -> V128 = i64::wrapping_abs;

    fn v128_not(v128: V128) -> V128 = <i64 as Not>::not;

    fn i8x16_popcnt(v128: V128) -> V128 = |v: u8| v.count_ones() as u8;

    fn f32x4_neg(v128: V128) -> V128 = <f32 as Neg>::neg;
    fn f64x2_neg(v128: V128) -> V128 = <f64 as Neg>::neg;
    fn f32x4_abs(v128: V128) -> V128 = f32::abs;
    fn f64x2_abs(v128: V128) -> V128 = f64::abs;
    fn f32x4_sqrt(v128: V128) -> V128 = wasm::f32_sqrt;
    fn f64x2_sqrt(v128: V128) -> V128 = wasm::f64_sqrt;
    fn f32x4_ceil(v128: V128) -> V128 = wasm::f32_ceil;
    fn f64x2_ceil(v128: V128) -> V128 = wasm::f64_ceil;
    fn f32x4_floor(v128: V128) -> V128 = wasm::f32_floor;
    fn f64x2_floor(v128: V128) -> V128 = wasm::f64_floor;
    fn f32x4_trunc(v128: V128) -> V128 = wasm::f32_trunc;
    fn f64x2_trunc(v128: V128) -> V128 = wasm::f64_trunc;
    fn f32x4_nearest(v128: V128) -> V128 = wasm::f32_nearest;
    fn f64x2_nearest(v128: V128) -> V128 = wasm::f64_nearest;
}

impl_unary_cast_for! {
    fn f32x4_convert_i32x4_s(v128: V128) -> V128 = wasm::f32_convert_i32_s;
    fn f32x4_convert_i32x4_u(v128: V128) -> V128 = wasm::f32_convert_i32_u;
    fn i32x4_trunc_sat_f32x4_s(v128: V128) -> V128 = wasm::i32_trunc_sat_f32_s;
    fn i32x4_trunc_sat_f32x4_u(v128: V128) -> V128 = wasm::i32_trunc_sat_f32_u;
}

macro_rules! impl_comparison_for {
    ( $( fn $name:ident(lhs: V128, rhs: V128) -> V128 = $lanewise_expr:expr; )* ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(lhs: V128, rhs: V128) -> V128 {
                V128::lanewise_comparison(lhs, rhs, $lanewise_expr)
            }
        )*
    };
}
impl_comparison_for! {
    fn i8x16_eq(lhs: V128, rhs: V128) -> V128 = op!(i8, ==);
    fn i16x8_eq(lhs: V128, rhs: V128) -> V128 = op!(i16, ==);
    fn i32x4_eq(lhs: V128, rhs: V128) -> V128 = op!(i32, ==);
    fn i64x2_eq(lhs: V128, rhs: V128) -> V128 = op!(i64, ==);
    fn f32x4_eq(lhs: V128, rhs: V128) -> V128 = op!(f32, ==);
    fn f64x2_eq(lhs: V128, rhs: V128) -> V128 = op!(f64, ==);

    fn i8x16_ne(lhs: V128, rhs: V128) -> V128 = op!(i8, !=);
    fn i16x8_ne(lhs: V128, rhs: V128) -> V128 = op!(i16, !=);
    fn i32x4_ne(lhs: V128, rhs: V128) -> V128 = op!(i32, !=);
    fn i64x2_ne(lhs: V128, rhs: V128) -> V128 = op!(i64, !=);
    fn f32x4_ne(lhs: V128, rhs: V128) -> V128 = op!(f32, !=);
    fn f64x2_ne(lhs: V128, rhs: V128) -> V128 = op!(f64, !=);

    fn i8x16_lt_s(lhs: V128, rhs: V128) -> V128 = op!(i8, <);
    fn i8x16_lt_u(lhs: V128, rhs: V128) -> V128 = op!(u8, <);
    fn i16x8_lt_s(lhs: V128, rhs: V128) -> V128 = op!(i16, <);
    fn i16x8_lt_u(lhs: V128, rhs: V128) -> V128 = op!(u16, <);
    fn i32x4_lt_s(lhs: V128, rhs: V128) -> V128 = op!(i32, <);
    fn i32x4_lt_u(lhs: V128, rhs: V128) -> V128 = op!(u32, <);
    fn i64x2_lt_s(lhs: V128, rhs: V128) -> V128 = op!(i64, <);
    fn f32x4_lt(lhs: V128, rhs: V128) -> V128 = op!(f32, <);
    fn f64x2_lt(lhs: V128, rhs: V128) -> V128 = op!(f64, <);

    fn i8x16_le_s(lhs: V128, rhs: V128) -> V128 = op!(i8, <=);
    fn i8x16_le_u(lhs: V128, rhs: V128) -> V128 = op!(u8, <=);
    fn i16x8_le_s(lhs: V128, rhs: V128) -> V128 = op!(i16, <=);
    fn i16x8_le_u(lhs: V128, rhs: V128) -> V128 = op!(u16, <=);
    fn i32x4_le_s(lhs: V128, rhs: V128) -> V128 = op!(i32, <=);
    fn i32x4_le_u(lhs: V128, rhs: V128) -> V128 = op!(u32, <=);
    fn i64x2_le_s(lhs: V128, rhs: V128) -> V128 = op!(i64, <=);
    fn f32x4_le(lhs: V128, rhs: V128) -> V128 = op!(f32, <=);
    fn f64x2_le(lhs: V128, rhs: V128) -> V128 = op!(f64, <=);

    fn i8x16_gt_s(lhs: V128, rhs: V128) -> V128 = op!(i8, >);
    fn i8x16_gt_u(lhs: V128, rhs: V128) -> V128 = op!(u8, >);
    fn i16x8_gt_s(lhs: V128, rhs: V128) -> V128 = op!(i16, >);
    fn i16x8_gt_u(lhs: V128, rhs: V128) -> V128 = op!(u16, >);
    fn i32x4_gt_s(lhs: V128, rhs: V128) -> V128 = op!(i32, >);
    fn i32x4_gt_u(lhs: V128, rhs: V128) -> V128 = op!(u32, >);
    fn i64x2_gt_s(lhs: V128, rhs: V128) -> V128 = op!(i64, >);
    fn f32x4_gt(lhs: V128, rhs: V128) -> V128 = op!(f32, >);
    fn f64x2_gt(lhs: V128, rhs: V128) -> V128 = op!(f64, >);

    fn i8x16_ge_s(lhs: V128, rhs: V128) -> V128 = op!(i8, >=);
    fn i8x16_ge_u(lhs: V128, rhs: V128) -> V128 = op!(u8, >=);
    fn i16x8_ge_s(lhs: V128, rhs: V128) -> V128 = op!(i16, >=);
    fn i16x8_ge_u(lhs: V128, rhs: V128) -> V128 = op!(u16, >=);
    fn i32x4_ge_s(lhs: V128, rhs: V128) -> V128 = op!(i32, >=);
    fn i32x4_ge_u(lhs: V128, rhs: V128) -> V128 = op!(u32, >=);
    fn i64x2_ge_s(lhs: V128, rhs: V128) -> V128 = op!(i64, >=);
    fn f32x4_ge(lhs: V128, rhs: V128) -> V128 = op!(f32, >=);
    fn f64x2_ge(lhs: V128, rhs: V128) -> V128 = op!(f64, >=);
}

macro_rules! impl_widen_low_unary {
    (
        $( fn $name:ident(v128: V128) -> V128 = $convert:expr; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(v128: V128) -> V128 {
                v128.low_unary($convert)
            }
        )*
    };
}
impl_widen_low_unary! {
    fn i16x8_extend_low_i8x16_s(v128: V128) -> V128 = <i8 as Into<i16>>::into;
    fn i16x8_extend_low_i8x16_u(v128: V128) -> V128 = <u8 as Into<u16>>::into;
    fn i32x4_extend_low_i16x8_s(v128: V128) -> V128 = <i16 as Into<i32>>::into;
    fn i32x4_extend_low_i16x8_u(v128: V128) -> V128 = <u16 as Into<u32>>::into;
    fn i64x2_extend_low_i32x4_s(v128: V128) -> V128 = <i32 as Into<i64>>::into;
    fn i64x2_extend_low_i32x4_u(v128: V128) -> V128 = <u32 as Into<u64>>::into;

    fn f64x2_convert_low_i32x4_s(v128: V128) -> V128 = wasm::f64_convert_i32_s;
    fn f64x2_convert_low_i32x4_u(v128: V128) -> V128 = wasm::f64_convert_i32_u;
    fn f64x2_promote_low_f32x4(v128: V128) -> V128 = wasm::f64_promote_f32;
}

macro_rules! impl_widen_high_unary {
    (
        $( fn $name:ident(v128: V128) -> V128 = $convert:expr; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(v128: V128) -> V128 {
                v128.high_unary($convert)
            }
        )*
    };
}
impl_widen_high_unary! {
    fn i16x8_extend_high_i8x16_s(v128: V128) -> V128 = <i8 as Into<i16>>::into;
    fn i16x8_extend_high_i8x16_u(v128: V128) -> V128 = <u8 as Into<u16>>::into;
    fn i32x4_extend_high_i16x8_s(v128: V128) -> V128 = <i16 as Into<i32>>::into;
    fn i32x4_extend_high_i16x8_u(v128: V128) -> V128 = <u16 as Into<u32>>::into;
    fn i64x2_extend_high_i32x4_s(v128: V128) -> V128 = <i32 as Into<i64>>::into;
    fn i64x2_extend_high_i32x4_u(v128: V128) -> V128 = <u32 as Into<u64>>::into;
}

macro_rules! extmul {
    ($narrow:ty => $wide:ty) => {{
        |a: $narrow, b: $narrow| -> $wide {
            let a = <$wide as From<$narrow>>::from(a);
            let b = <$wide as From<$narrow>>::from(b);
            a.wrapping_mul(b)
        }
    }};
}

macro_rules! impl_ext_binary_low {
    (
        $( fn $name:ident(lhs: V128, rhs: V128) -> V128 = $f:expr; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($extmul_low), "` instruction.")]
            pub fn $name(lhs: V128, rhs: V128) -> V128 {
                V128::from_low_binary(lhs, rhs, $f)
            }
        )*
    };
}
impl_ext_binary_low! {
    fn i16x8_extmul_low_i8x16_s(lhs: V128, rhs: V128) -> V128 = extmul!( i8 => i16);
    fn i16x8_extmul_low_i8x16_u(lhs: V128, rhs: V128) -> V128 = extmul!( u8 => u16);
    fn i32x4_extmul_low_i16x8_s(lhs: V128, rhs: V128) -> V128 = extmul!(i16 => i32);
    fn i32x4_extmul_low_i16x8_u(lhs: V128, rhs: V128) -> V128 = extmul!(u16 => u32);
    fn i64x2_extmul_low_i32x4_s(lhs: V128, rhs: V128) -> V128 = extmul!(i32 => i64);
    fn i64x2_extmul_low_i32x4_u(lhs: V128, rhs: V128) -> V128 = extmul!(u32 => u64);
}

macro_rules! impl_ext_binary_high {
    (
        $( fn $name:ident(lhs: V128, rhs: V128) -> V128 = $f:expr; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($extmul_low), "` instruction.")]
            pub fn $name(lhs: V128, rhs: V128) -> V128 {
                V128::from_high_binary(lhs, rhs, $f)
            }
        )*
    };
}
impl_ext_binary_high! {
    fn i16x8_extmul_high_i8x16_s(lhs: V128, rhs: V128) -> V128 = extmul!( i8 => i16);
    fn i16x8_extmul_high_i8x16_u(lhs: V128, rhs: V128) -> V128 = extmul!( u8 => u16);
    fn i32x4_extmul_high_i16x8_s(lhs: V128, rhs: V128) -> V128 = extmul!(i16 => i32);
    fn i32x4_extmul_high_i16x8_u(lhs: V128, rhs: V128) -> V128 = extmul!(u16 => u32);
    fn i64x2_extmul_high_i32x4_s(lhs: V128, rhs: V128) -> V128 = extmul!(i32 => i64);
    fn i64x2_extmul_high_i32x4_u(lhs: V128, rhs: V128) -> V128 = extmul!(u32 => u64);
}

macro_rules! impl_extadd_pairwise {
    (
        $( fn $name:ident(v128: V128) -> V128 = $narrow:ty => $wide:ty; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(v128: V128) -> V128 {
                fn extadd_pairwise(a: $narrow, b: $narrow) -> $wide {
                    let a = <$wide>::from(a);
                    let b = <$wide>::from(b);
                    a.wrapping_add(b)
                }
                v128.pairwise_unary(extadd_pairwise)
            }
        )*
    };
}
impl_extadd_pairwise! {
    fn i16x8_extadd_pairwise_i8x16_s(v128: V128) -> V128 = i8 => i16;
    fn i16x8_extadd_pairwise_i8x16_u(v128: V128) -> V128 = u8 => u16;
    fn i32x4_extadd_pairwise_i16x8_s(v128: V128) -> V128 = i16 => i32;
    fn i32x4_extadd_pairwise_i16x8_u(v128: V128) -> V128 = u16 => u32;
}

macro_rules! impl_shift_ops {
    (
        $( fn $name:ident(v128: V128, rhs: u32) -> V128 = $lanewise_expr:expr; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(v128: V128, rhs: u32) -> V128 {
                v128.lanewise_unary(|v| $lanewise_expr(v, rhs))
            }
        )*
    };
}
impl_shift_ops! {
    fn i8x16_shl(v128: V128, rhs: u32) -> V128 = i8::wrapping_shl;
    fn i16x8_shl(v128: V128, rhs: u32) -> V128 = i16::wrapping_shl;
    fn i32x4_shl(v128: V128, rhs: u32) -> V128 = i32::wrapping_shl;
    fn i64x2_shl(v128: V128, rhs: u32) -> V128 = i64::wrapping_shl;
    fn i8x16_shr_s(v128: V128, rhs: u32) -> V128 = i8::wrapping_shr;
    fn i8x16_shr_u(v128: V128, rhs: u32) -> V128 = u8::wrapping_shr;
    fn i16x8_shr_s(v128: V128, rhs: u32) -> V128 = i16::wrapping_shr;
    fn i16x8_shr_u(v128: V128, rhs: u32) -> V128 = u16::wrapping_shr;
    fn i32x4_shr_s(v128: V128, rhs: u32) -> V128 = i32::wrapping_shr;
    fn i32x4_shr_u(v128: V128, rhs: u32) -> V128 = u32::wrapping_shr;
    fn i64x2_shr_s(v128: V128, rhs: u32) -> V128 = i64::wrapping_shr;
    fn i64x2_shr_u(v128: V128, rhs: u32) -> V128 = u64::wrapping_shr;
}

macro_rules! impl_narrowing_low_high_ops {
    (
        $( fn $name:ident(low: V128, high: V128) -> V128 = $f:expr; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(low: V128, high: V128) -> V128 {
                V128::from_low_high(low, high, $f)
            }
        )*
    };
}
impl_narrowing_low_high_ops! {
    fn i8x16_narrow_i16x8_s(low: V128, high: V128) -> V128 = narrow_i16_to_i8;
    fn i8x16_narrow_i16x8_u(low: V128, high: V128) -> V128 = narrow_u16_to_u8;
    fn i16x8_narrow_i32x4_s(low: V128, high: V128) -> V128 = narrow_i32_to_i16;
    fn i16x8_narrow_i32x4_u(low: V128, high: V128) -> V128 = narrow_u32_to_u16;
}

macro_rules! def_narrow_from_to {
    (
        $( fn $name:ident(value: $from:ty $(as $as:ty)? ) -> $to:ty );* $(;)?
    ) => {
        $(
            #[doc = concat!("Narrows `value` from type `", stringify!($from), "` to type `", stringify!($to), "`.")]
            fn $name(value: $from) -> $to {
                $( let value: $as = value as $as; )?
                value.clamp(<$to>::MIN.into(), <$to>::MAX.into()) as $to
            }
        )*
    };
}
def_narrow_from_to! {
    fn narrow_i16_to_i8(value: i16) -> i8;
    fn narrow_u16_to_u8(value: u16 as i16) -> u8;
    fn narrow_i32_to_i16(value: i32) -> i16;
    fn narrow_u32_to_u16(value: u32 as i32) -> u16;
}

macro_rules! impl_narrowing_low_high_ops {
    (
        $( fn $name:ident(v128: V128) -> V128 = (high: $high:expr, f: $f:expr); )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(low: V128) -> V128 {
                V128::low_or(low, $high, $f)
            }
        )*
    };
}
impl_narrowing_low_high_ops! {
    fn i32x4_trunc_sat_f64x2_s_zero(v128: V128) -> V128 = (high: || 0, f: wasm::i32_trunc_sat_f64_s);
    fn i32x4_trunc_sat_f64x2_u_zero(v128: V128) -> V128 = (high: || 0, f: wasm::i32_trunc_sat_f64_u);
    fn f32x4_demote_f64x2_zero(v128: V128) -> V128 = (high: || 0.0, f: wasm::f32_demote_f64);
}

macro_rules! all_true {
    ($ty:ty) => {{ |v: $ty, acc: bool| acc & (v != 0) }};
}
macro_rules! impl_all_true_ops {
    (
        $( fn $name:ident(v128: V128) -> bool = $f:expr; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(v128: V128) -> bool {
                v128.lanewise_reduce(true, $f)
            }
        )*
    };
}
impl_all_true_ops! {
    fn i8x16_all_true(v128: V128) -> bool = all_true!(i8);
    fn i16x8_all_true(v128: V128) -> bool = all_true!(i16);
    fn i32x4_all_true(v128: V128) -> bool = all_true!(i32);
    fn i64x2_all_true(v128: V128) -> bool = all_true!(i64);
}

macro_rules! bitmask {
    ($ty:ty) => {{ |n: u8, v: $ty, acc| acc | (i32::from(v < 0).wrapping_shl(u32::from(n))) }};
}
macro_rules! impl_bitmask_ops {
    (
        $( fn $name:ident(v128: V128) -> u32 = $f:expr; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            pub fn $name(v128: V128) -> u32 {
                v128.lanewise_reduce_enumerate(0_i32, $f) as _
            }
        )*
    };
}
impl_bitmask_ops! {
    fn i8x16_bitmask(v128: V128) -> u32 = bitmask!(i8);
    fn i16x8_bitmask(v128: V128) -> u32 = bitmask!(i16);
    fn i32x4_bitmask(v128: V128) -> u32 = bitmask!(i32);
    fn i64x2_bitmask(v128: V128) -> u32 = bitmask!(i64);
}

/// Executes a Wasm `v128.any_true` instruction.
pub fn v128_any_true(v128: V128) -> bool {
    v128.as_u128() != 0
}

/// Executes a Wasm `i32x4.dot_i16x8_s` instruction.
pub fn i32x4_dot_i16x8_s(lhs: V128, rhs: V128) -> V128 {
    fn dot(a: [i16; 2], b: [i16; 2]) -> i32 {
        let a = a.map(i32::from);
        let b = b.map(i32::from);
        let dot0 = a[0].wrapping_mul(b[0]);
        let dot1 = a[1].wrapping_mul(b[1]);
        dot0.wrapping_add(dot1)
    }
    V128::pairwise_binary(lhs, rhs, dot)
}

/// Executes a Wasm `i16x8.relaxed_dot_i8x16_i7x16_s` instruction.
///
/// # Note
///
/// This is part of the `relaxed-simd` Wasm proposal.
pub fn i16x8_relaxed_dot_i8x16_i7x16_s(lhs: V128, rhs: V128) -> V128 {
    fn dot(a: [i8; 2], b: [i8; 2]) -> i16 {
        let a = a.map(i16::from);
        let b = b.map(i16::from);
        let dot0 = a[0].wrapping_mul(b[0]);
        let dot1 = a[1].wrapping_mul(b[1]);
        dot0.wrapping_add(dot1)
    }
    V128::pairwise_binary(lhs, rhs, dot)
}

/// Executes a Wasm `i32x4.relaxed_dot_i8x16_i7x16_add_s` instruction.
///
/// # Note
///
/// This is part of the `relaxed-simd` Wasm proposal.
pub fn i32x4_relaxed_dot_i8x16_i7x16_add_s(lhs: V128, rhs: V128, c: V128) -> V128 {
    let dot = i16x8_relaxed_dot_i8x16_i7x16_s(lhs, rhs);
    let ext = i32x4_extadd_pairwise_i16x8_s(dot);
    i32x4_add(ext, c)
}

/// Executes a Wasm `v128.bitselect` instruction.
pub fn v128_bitselect(v1: V128, v2: V128, c: V128) -> V128 {
    simd::v128_or(simd::v128_and(v1, c), simd::v128_andnot(v2, c))
}

/// Computes the negative `mul_add`: `-(a * b) + c`
fn neg_mul_add<T>(a: T, b: T, c: T) -> T
where
    T: Float + Neg<Output = T>,
{
    <T as Float>::mul_add(a.neg(), b, c)
}

macro_rules! impl_ternary_for {
    ( $( fn $name:ident(a: V128, b: V128, c: V128) -> V128 = $lanewise_expr:expr; )* ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            #[doc = ""]
            #[doc = "# Note"]
            #[doc = ""]
            #[doc = "This is part of the `relaxed-simd` Wasm proposal."]
            pub fn $name(a: V128, b: V128, c: V128) -> V128 {
                V128::lanewise_ternary(a, b, c, $lanewise_expr)
            }
        )*
    };
}
impl_ternary_for! {
    fn f32x4_relaxed_madd(a: V128, b: V128, c: V128) -> V128 = <f32 as Float>::mul_add;
    fn f32x4_relaxed_nmadd(a: V128, b: V128, c: V128) -> V128 = neg_mul_add::<f32>;
    fn f64x2_relaxed_madd(a: V128, b: V128, c: V128) -> V128 = <f64 as Float>::mul_add;
    fn f64x2_relaxed_nmadd(a: V128, b: V128, c: V128) -> V128 = neg_mul_add::<f64>;
}

/// Executes a Wasm `v128.store` instruction.
///
/// # Errors
///
/// - If `ptr + offset` overflows.
/// - If `ptr + offset` stores out of bounds from `memory`.
pub fn v128_store(memory: &mut [u8], ptr: u64, offset: u64, value: V128) -> Result<(), TrapCode> {
    memory::store(memory, ptr, offset, value.as_u128())
}

/// Executes a Wasm `v128.store` instruction.
///
/// # Errors
///
/// If `address` stores out of bounds from `memory`.
pub fn v128_store_at(memory: &mut [u8], address: usize, value: V128) -> Result<(), TrapCode> {
    memory::store_at(memory, address, value.as_u128())
}

macro_rules! impl_v128_storeN_lane {
    (
        $(
            fn $name:ident(
                memory: &mut [u8],
                ptr: u64,
                offset: u64,
                value: V128,
                imm: $lane_idx:ty $(,)?
            ) -> Result<(), TrapCode>
            = $store_ty:ty;
        )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            ///
            /// # Errors
            ///
            /// - If `ptr + offset` overflows.
            /// - If `ptr + offset` stores out of bounds from `memory`.
            pub fn $name(memory: &mut [u8], ptr: u64, offset: u64, value: V128, imm: $lane_idx) -> Result<(), TrapCode> {
                memory::store(memory, ptr, offset, value.extract_lane::<$store_ty>(imm))
            }
        )*
    };
}
impl_v128_storeN_lane! {
    fn v128_store8_lane(
        memory: &mut [u8],
        ptr: u64,
        offset: u64,
        value: V128,
        imm: ImmLaneIdx16,
    ) -> Result<(), TrapCode> = u8;

    fn v128_store16_lane(
        memory: &mut [u8],
        ptr: u64,
        offset: u64,
        value: V128,
        imm: ImmLaneIdx8,
    ) -> Result<(), TrapCode> = u16;

    fn v128_store32_lane(
        memory: &mut [u8],
        ptr: u64,
        offset: u64,
        value: V128,
        imm: ImmLaneIdx4,
    ) -> Result<(), TrapCode> = u32;

    fn v128_store64_lane(
        memory: &mut [u8],
        ptr: u64,
        offset: u64,
        value: V128,
        imm: ImmLaneIdx2,
    ) -> Result<(), TrapCode> = u64;
}

macro_rules! impl_v128_storeN_lane_at {
    (
        $( fn $name:ident(memory: &mut [u8], address: usize, value: V128, imm: $lane_idx:ty) -> Result<(), TrapCode> = $store_ty:ty; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            ///
            /// # Errors
            ///
            /// If `address` stores out of bounds from `memory`.
            pub fn $name(memory: &mut [u8], address: usize, value: V128, imm: $lane_idx) -> Result<(), TrapCode> {
                memory::store_at(memory, address, value.extract_lane::<$store_ty>(imm))
            }
        )*
    };
}
impl_v128_storeN_lane_at! {
    fn v128_store8_lane_at(
        memory: &mut [u8], address: usize, value: V128, imm: ImmLaneIdx16
    ) -> Result<(), TrapCode> = u8;
    fn v128_store16_lane_at(
        memory: &mut [u8], address: usize, value: V128, imm: ImmLaneIdx8
    ) -> Result<(), TrapCode> = u16;
    fn v128_store32_lane_at(
        memory: &mut [u8], address: usize, value: V128, imm: ImmLaneIdx4
    ) -> Result<(), TrapCode> = u32;
    fn v128_store64_lane_at(
        memory: &mut [u8], address: usize, value: V128, imm: ImmLaneIdx2
    ) -> Result<(), TrapCode> = u64;
}

/// Executes a Wasmi `v128.load` instruction.
///
/// # Errors
///
/// - If `ptr + offset` overflows.
/// - If `ptr + offset` loads out of bounds from `memory`.
pub fn v128_load(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> {
    memory::load::<u128>(memory, ptr, offset).map(V128::from)
}

/// Executes a Wasmi `v128.load` instruction.
///
/// # Errors
///
/// If `address` loads out of bounds from `memory`.
pub fn v128_load_at(memory: &[u8], address: usize) -> Result<V128, TrapCode> {
    memory::load_at::<u128>(memory, address).map(V128::from)
}

macro_rules! impl_v128_loadN_zero_for {
    (
        $( fn $name:ident(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> = $ty:ty; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            ///
            /// # Errors
            ///
            /// - If `ptr + offset` overflows.
            /// - If `ptr + offset` loads out of bounds from `memory`.
            pub fn $name(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> {
                let bits = memory::load::<$ty>(memory, ptr, offset)?;
                Ok(V128::splat::<$ty>(0).replace_lane::<$ty>(<$ty as IntoLaneIdx>::LaneIdx::zero(), bits))
            }
        )*
    };
}
impl_v128_loadN_zero_for! {
    fn v128_load32_zero(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> = u32;
    fn v128_load64_zero(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> = u64;
}

macro_rules! impl_v128_loadN_zero_at_for {
    (
        $( fn $name:ident(memory: &[u8], address: usize) -> Result<V128, TrapCode> = $ty:ty; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            ///
            /// # Errors
            ///
            /// If `address` loads out of bounds from `memory`.
            pub fn $name(memory: &[u8], address: usize) -> Result<V128, TrapCode> {
                let bits = memory::load_at::<$ty>(memory, address)?;
                Ok(V128::splat::<$ty>(0).replace_lane::<$ty>(<$ty as IntoLaneIdx>::LaneIdx::zero(), bits))
            }
        )*
    };
}
impl_v128_loadN_zero_at_for! {
    fn v128_load32_zero_at(memory: &[u8], address: usize) -> Result<V128, TrapCode> = u32;
    fn v128_load64_zero_at(memory: &[u8], address: usize) -> Result<V128, TrapCode> = u64;
}

macro_rules! impl_v128_loadN_splat_for {
    (
        $( fn $name:ident(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> = $ty:ty; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            ///
            /// # Errors
            ///
            /// - If `ptr + offset` overflows.
            /// - If `ptr + offset` loads out of bounds from `memory`.
            pub fn $name(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> {
                memory::load::<$ty>(memory, ptr, offset).map(V128::splat)
            }
        )*
    };
}
impl_v128_loadN_splat_for! {
    fn v128_load8_splat(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> = u8;
    fn v128_load16_splat(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> = u16;
    fn v128_load32_splat(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> = u32;
    fn v128_load64_splat(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> = u64;
}

macro_rules! impl_v128_loadN_splat_at_for {
    (
        $( fn $name:ident(memory: &[u8], address: usize) -> Result<V128, TrapCode> = $ty:ty; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            ///
            /// # Errors
            ///
            /// If `address` loads out of bounds from `memory`.
            pub fn $name(memory: &[u8], address: usize) -> Result<V128, TrapCode> {
                memory::load_at::<$ty>(memory, address).map(V128::splat)
            }
        )*
    };
}
impl_v128_loadN_splat_at_for! {
    fn v128_load8_splat_at(memory: &[u8], address: usize) -> Result<V128, TrapCode> = u8;
    fn v128_load16_splat_at(memory: &[u8], address: usize) -> Result<V128, TrapCode> = u16;
    fn v128_load32_splat_at(memory: &[u8], address: usize) -> Result<V128, TrapCode> = u32;
    fn v128_load64_splat_at(memory: &[u8], address: usize) -> Result<V128, TrapCode> = u64;
}

macro_rules! impl_v128_loadN_lane_for {
    (
        $(
            fn $name:ident(
                memory: &[u8],
                ptr: u64,
                offset: u64,
                x: V128,
                lane: $lane_idx:ty $(,)?
            ) -> Result<V128, TrapCode> = $ty:ty; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            ///
            /// # Errors
            ///
            /// - If `ptr + offset` overflows.
            /// - If `ptr + offset` loads out of bounds from `memory`.
            pub fn $name(memory: &[u8], ptr: u64, offset: u64, x: V128, lane: $lane_idx) -> Result<V128, TrapCode> {
                memory::load::<$ty>(memory, ptr, offset).map(|value| x.replace_lane(lane, value))
            }
        )*
    };
}
impl_v128_loadN_lane_for! {
    fn v128_load8_lane(memory: &[u8], ptr: u64, offset: u64, x: V128, lane: ImmLaneIdx16) -> Result<V128, TrapCode> = u8;
    fn v128_load16_lane(memory: &[u8], ptr: u64, offset: u64, x: V128, lane: ImmLaneIdx8) -> Result<V128, TrapCode> = u16;
    fn v128_load32_lane(memory: &[u8], ptr: u64, offset: u64, x: V128, lane: ImmLaneIdx4) -> Result<V128, TrapCode> = u32;
    fn v128_load64_lane(memory: &[u8], ptr: u64, offset: u64, x: V128, lane: ImmLaneIdx2) -> Result<V128, TrapCode> = u64;
}

macro_rules! impl_v128_loadN_lane_at_for {
    (
        $( fn $name:ident(memory: &[u8], address: usize, x: V128, lane: $lane_idx:ty) -> Result<V128, TrapCode> = $ty:ty; )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            ///
            /// # Errors
            ///
            /// If `address` loads out of bounds from `memory`.
            pub fn $name(memory: &[u8], address: usize, x: V128, lane: $lane_idx) -> Result<V128, TrapCode> {
                memory::load_at::<$ty>(memory, address).map(|value| x.replace_lane(lane, value))
            }
        )*
    };
}
impl_v128_loadN_lane_at_for! {
    fn v128_load8_lane_at(memory: &[u8], address: usize, x: V128, lane: ImmLaneIdx16) -> Result<V128, TrapCode> = u8;
    fn v128_load16_lane_at(memory: &[u8], address: usize, x: V128, lane: ImmLaneIdx8) -> Result<V128, TrapCode> = u16;
    fn v128_load32_lane_at(memory: &[u8], address: usize, x: V128, lane: ImmLaneIdx4) -> Result<V128, TrapCode> = u32;
    fn v128_load64_lane_at(memory: &[u8], address: usize, x: V128, lane: ImmLaneIdx2) -> Result<V128, TrapCode> = u64;
}

/// Allows `Self` to be safely and efficiently split into `T`.
///
/// Usually `T` is an array of `U` where `U` fits multiple times into `Self`.
/// An example of this is that `u64` can be split into `[u32; 2]`.
///
/// This is a helper trait to implement [`V128::load_nxm`] generically.
trait SplitInto<T> {
    type Output;
    fn split_into(self) -> Self::Output;
}

macro_rules! impl_split_into_for {
    ( $( impl SplitInto<$ty:ty> for u64; )* ) => {
        $(
            impl SplitInto<$ty> for u64 {
                type Output = [$ty; core::mem::size_of::<u64>() / core::mem::size_of::<$ty>()];

                fn split_into(self) -> Self::Output {
                    let bytes = self.to_ne_bytes();
                    array::from_fn(|i| {
                        <$ty>::from_ne_bytes(array::from_fn(|j| {
                            bytes[core::mem::size_of::<$ty>() * i + j]
                        }))
                    })
                }
            }
        )*
    };
}
impl_split_into_for! {
    impl SplitInto<u8> for u64;
    impl SplitInto<i8> for u64;
    impl SplitInto<u16> for u64;
    impl SplitInto<i16> for u64;
    impl SplitInto<u32> for u64;
    impl SplitInto<i32> for u64;
}

/// Allows to extend all items in an array from `T` to `Ext`.
///
/// This is a helper trait to implement [`V128::load_nxm`] generically.
trait ExtendArray<T> {
    type Output;
    fn extend_array(self) -> Self::Output;
}

impl<const N: usize, Ext, T> ExtendArray<Ext> for [T; N]
where
    T: ExtendInto<Ext>,
{
    type Output = [Ext; N];
    fn extend_array(self) -> Self::Output {
        self.map(<T as ExtendInto<Ext>>::extend_into)
    }
}

impl V128 {
    /// Interprets `bits` as array of `Narrow` and distribute the (sign) extended items as [`V128`].
    fn load_nxm<Narrow, Wide>(bits: u64) -> V128
    where
        u64: SplitInto<Narrow, Output: ExtendArray<Wide, Output: Into<<Wide as IntoLanes>::Lanes>>>,
        Wide: IntoLanes,
    {
        bits.split_into().extend_array().into().into_v128()
    }
}

macro_rules! impl_v128_load_mxn {
    (
        $( fn $name:ident(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> = ($n:ty => $w:ty); )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            ///
            /// # Errors
            ///
            /// - If `ptr + offset` overflows.
            /// - If `ptr + offset` loads out of bounds from `memory`.
            pub fn $name(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> {
                memory::load::<u64>(memory, ptr, offset).map(V128::load_nxm::<$n, $w>)
            }
        )*
    };
}
impl_v128_load_mxn! {
    fn v128_load8x8_s(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> = (i8 => i16);
    fn v128_load8x8_u(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> = (u8 => u16);
    fn v128_load16x4_s(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> = (i16 => i32);
    fn v128_load16x4_u(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> = (u16 => u32);
    fn v128_load32x2_s(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> = (i32 => i64);
    fn v128_load32x2_u(memory: &[u8], ptr: u64, offset: u64) -> Result<V128, TrapCode> = (u32 => u64);
}

macro_rules! impl_v128_load_mxn_at {
    (
        $( fn $name:ident(memory: &[u8], address: usize) -> Result<V128, TrapCode> = ($n:ty => $w:ty); )*
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            ///
            /// # Errors
            ///
            /// If `address` loads out of bounds from `memory`.
            pub fn $name(memory: &[u8], address: usize) -> Result<V128, TrapCode> {
                memory::load_at::<u64>(memory, address).map(V128::load_nxm::<$n, $w>)
            }
        )*
    };
}
impl_v128_load_mxn_at! {
    fn v128_load8x8_s_at(memory: &[u8], address: usize) -> Result<V128, TrapCode> = (i8 => i16);
    fn v128_load8x8_u_at(memory: &[u8], address: usize) -> Result<V128, TrapCode> = (u8 => u16);
    fn v128_load16x4_s_at(memory: &[u8], address: usize) -> Result<V128, TrapCode> = (i16 => i32);
    fn v128_load16x4_u_at(memory: &[u8], address: usize) -> Result<V128, TrapCode> = (u16 => u32);
    fn v128_load32x2_s_at(memory: &[u8], address: usize) -> Result<V128, TrapCode> = (i32 => i64);
    fn v128_load32x2_u_at(memory: &[u8], address: usize) -> Result<V128, TrapCode> = (u32 => u64);
}

macro_rules! impl_forwarding_relaxed_ops {
    (
        $(
            fn $name:ident(
                $( $param_name:ident: $param_ty:ty ),* $(,)?
            ) -> $ret_ty:ty
            = $forward_fn:expr
        );* $(;)?
    ) => {
        $(
            #[doc = concat!("Executes a Wasm `", stringify!($name), "` instruction.")]
            #[doc = ""]
            #[doc = "# Note"]
            #[doc = ""]
            #[doc = "This is part of the `relaxed-simd` Wasm proposal."]
            pub fn $name( $( $param_name: $param_ty ),* ) -> $ret_ty {
                $forward_fn( $( $param_name ),* )
            }
        )*
    };
}
impl_forwarding_relaxed_ops! {
    fn i8x16_relaxed_swizzle(a: V128, s: V128) -> V128 = i8x16_swizzle;

    fn i8x16_relaxed_laneselect(a: V128, b: V128, c: V128) -> V128 = v128_bitselect;
    fn i16x8_relaxed_laneselect(a: V128, b: V128, c: V128) -> V128 = v128_bitselect;
    fn i32x4_relaxed_laneselect(a: V128, b: V128, c: V128) -> V128 = v128_bitselect;
    fn i64x2_relaxed_laneselect(a: V128, b: V128, c: V128) -> V128 = v128_bitselect;

    fn f32x4_relaxed_min(lhs: V128, rhs: V128) -> V128 = f32x4_min;
    fn f32x4_relaxed_max(lhs: V128, rhs: V128) -> V128 = f32x4_max;
    fn f64x2_relaxed_min(lhs: V128, rhs: V128) -> V128 = f64x2_min;
    fn f64x2_relaxed_max(lhs: V128, rhs: V128) -> V128 = f64x2_max;

    fn i16x8_relaxed_q15mulr_s(a: V128, b: V128) -> V128 = i16x8_q15mulr_sat_s;

    fn i32x4_relaxed_trunc_f32x4_s(input: V128) -> V128 = i32x4_trunc_sat_f32x4_s;
    fn i32x4_relaxed_trunc_f32x4_u(input: V128) -> V128 = i32x4_trunc_sat_f32x4_u;
    fn i32x4_relaxed_trunc_f64x2_s_zero(input: V128) -> V128 = i32x4_trunc_sat_f64x2_s_zero;
    fn i32x4_relaxed_trunc_f64x2_u_zero(input: V128) -> V128 = i32x4_trunc_sat_f64x2_u_zero;
}

#[test]
fn i32x4_dot_i16x8_s_works() {
    assert_eq!(
        simd::i32x4_dot_i16x8_s(simd::i16x8_splat(16383_i16), simd::i16x8_splat(16384_i16)),
        simd::i32x4_splat(536838144_i32)
    );
}

#[test]
fn v128_or_works() {
    assert_eq!(
        simd::v128_or(simd::i16x8_splat(0), simd::i16x8_splat(0xffff_u16 as i16),),
        simd::i16x8_splat(0xffff_u16 as i16),
    );
}

#[test]
fn i8x16_narrow_i16x8_s_works() {
    assert_eq!(
        simd::i8x16_narrow_i16x8_s(simd::i16x8_splat(0x80_i16), simd::i16x8_splat(0x80_i16)),
        simd::i8x16_splat(0x7f),
    );
}
