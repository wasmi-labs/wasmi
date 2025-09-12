use crate::{
    core::{Typed, TypedVal, UntypedVal},
    ir::{Op, Sign},
    Error,
    ExternRef,
    Func,
    Ref,
    ValType,
};
use core::{convert::identity, num::NonZero};

impl Typed for ExternRef {
    const TY: ValType = ValType::ExternRef;
}

macro_rules! impl_typed_for {
    ( $( $ty:ty as $ident:ident ),* $(,)? ) => {
        $(
            impl Typed for $ty {
                const TY: ValType = crate::ValType::$ident;
            }

            impl From<TypedVal> for $ty {
                fn from(typed_value: TypedVal) -> Self {
                    // # Note
                    //
                    // We only use a `debug_assert` here instead of a proper `assert`
                    // since the whole translation process assumes that Wasm validation
                    // was already performed and thus type checking does not necessarily
                    // need to happen redundantly outside of debug builds.
                    debug_assert!(matches!(typed_value.ty(), <$ty as Typed>::TY));
                    Self::from(typed_value.untyped())
                }
            }
        )*
    };
}
impl_typed_for! {
    Ref<Func> as FuncRef,
    Ref<ExternRef> as ExternRef,
}

/// A WebAssembly integer. Either `i32` or `i64`.
///
/// # Note
///
/// This trait provides some utility methods useful for translation.
pub trait WasmInteger:
    Copy + Eq + Typed + From<TypedVal> + Into<TypedVal> + From<UntypedVal> + Into<UntypedVal>
{
    /// The non-zero type of the [`WasmInteger`].
    type NonZero: Copy + Into<Self> + Into<UntypedVal>;

    /// Returns `self` as [`Self::NonZero`] if possible.
    ///
    /// Returns `None` if `self` is zero.
    fn non_zero(self) -> Option<Self::NonZero>;

    /// Returns `true` if `self` is equal to zero (0).
    fn is_zero(self) -> bool;

    /// Returns the wrapped negated `self`.
    fn wrapping_neg(self) -> Self;
}

macro_rules! impl_wasm_integer {
    ($($ty:ty),*) => {
        $(
            impl WasmInteger for $ty {
                type NonZero = NonZero<Self>;

                fn non_zero(self) -> Option<Self::NonZero> {
                    Self::NonZero::new(self)
                }

                fn is_zero(self) -> bool {
                    self == 0
                }

                fn wrapping_neg(self) -> Self {
                    Self::wrapping_neg(self)
                }
            }
        )*
    };
}
impl_wasm_integer!(i32, u32, i64, u64);

/// A WebAssembly float. Either `f32` or `f64`.
///
/// # Note
///
/// This trait provides some utility methods useful for translation.
pub trait WasmFloat: Typed + Copy + Into<TypedVal> + From<TypedVal> {
    /// Returns the [`Sign`] of `self`.
    fn sign(self) -> Sign<Self>;
}

impl WasmFloat for f32 {
    fn sign(self) -> Sign<Self> {
        Sign::from(self)
    }
}

impl WasmFloat for f64 {
    fn sign(self) -> Sign<Self> {
        Sign::from(self)
    }
}

/// Implemented by integer types to wrap them to another (smaller) integer type.
pub trait Wrap<T> {
    /// Wraps `self` into a value of type `T`.
    fn wrap(self) -> T;
}

impl<T> Wrap<T> for T {
    #[inline]
    fn wrap(self) -> T {
        self
    }
}

macro_rules! impl_wrap_for {
    ( $($from_ty:ty => $to_ty:ty),* $(,)? ) => {
        $(
            impl Wrap<$to_ty> for $from_ty {
                #[inline]
                fn wrap(self) -> $to_ty { self as _ }
            }
        )*
    };
}
impl_wrap_for! {
    // signed
    i16 => i8,
    i32 => i8,
    i32 => i16,
    i64 => i8,
    i64 => i16,
    i64 => i32,
    // unsigned
    u16 => u8,
    u32 => u8,
    u32 => u16,
    u64 => u8,
    u64 => u16,
    u64 => u32,
}

/// Extension trait to bump the consumed fuel of [`Op::ConsumeFuel`].
pub trait BumpFuelConsumption {
    /// Increases the fuel consumption of the [`Op::ConsumeFuel`] instruction by `delta`.
    ///
    /// # Error
    ///
    /// - If `self` is not a [`Op::ConsumeFuel`] instruction.
    /// - If the new fuel consumption overflows the internal `u64` value.
    fn bump_fuel_consumption(&mut self, delta: u64) -> Result<(), Error>;
}

impl BumpFuelConsumption for Op {
    fn bump_fuel_consumption(&mut self, delta: u64) -> Result<(), Error> {
        match self {
            Self::ConsumeFuel { fuel } => fuel.bump_by(delta).map_err(Error::from),
            instr => panic!("expected `Op::ConsumeFuel` but found: {instr:?}"),
        }
    }
}

/// A reference to an encoded [`Op`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instr(u32);

impl From<u32> for Instr {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl From<Instr> for u32 {
    fn from(instr: Instr) -> Self {
        instr.0
    }
}

impl Instr {
    /// Creates an [`Instr`] from the given `usize` value.
    ///
    /// # Note
    ///
    /// This intentionally is an API intended for test purposes only.
    ///
    /// # Panics
    ///
    /// If the `value` exceeds limitations for [`Instr`].
    pub fn from_usize(value: usize) -> Self {
        let Ok(index) = u32::try_from(value) else {
            panic!("out of bounds index {value} for `Instr`")
        };
        Self(index)
    }

    /// Returns an `usize` representation of the instruction index.
    pub fn into_usize(self) -> usize {
        match usize::try_from(self.0) {
            Ok(index) => index,
            Err(error) => {
                panic!("out of bound index {} for `Instr`: {error}", self.0)
            }
        }
    }

    /// Returns the absolute distance between `self` and `other`.
    ///
    /// - Returns `0` if `self == other`.
    /// - Returns `1` if `self` is adjacent to `other` in the sequence of instructions.
    /// - etc..
    pub fn distance(self, other: Self) -> u32 {
        self.0.abs_diff(other.0)
    }
}

/// Types that can be converted into bits.
pub trait ToBits {
    /// The output bits type of [`ToBits`].
    type Out: Copy;

    /// Converts `self` into a 32-bit `u32` value.
    fn to_bits(self) -> Self::Out;
}

macro_rules! impl_to_bits {
    ( $($ty:ty as $bits_ty:ty = $expr:expr),* $(,)? ) => {
        $(
            impl ToBits for $ty {
                type Out = $bits_ty;
                fn to_bits(self) -> Self::Out {
                    $expr(self)
                }
            }
        )*
    };
}
impl_to_bits! {
    u8 as u8 = identity,
    u16 as u16 = identity,
    u32 as u32 = identity,
    u64 as u64 = identity,

    f32 as u32 = f32::to_bits,
    f64 as u64 = f64::to_bits,

    i8 as u8 = |v: i8| u8::from_ne_bytes(v.to_ne_bytes()),
    i16 as u16 = |v: i16| u16::from_ne_bytes(v.to_ne_bytes()),
    i32 as u32 = |v: i32| u32::from_ne_bytes(v.to_ne_bytes()),
    i64 as u64 = |v: i64| u64::from_ne_bytes(v.to_ne_bytes()),
}

pub trait IntoShiftAmount {
    /// The type denoting the shift amount.
    ///
    /// This is an unsigned integer ranging from `1..N` where `N` is the number of bits in `Self`.
    type Value: Copy;

    /// Returns `self` wrapped into a proper shift amount for `Self`.
    ///
    /// Returns `None` if the resulting shift amount is 0, a.k.a. a no-op.
    fn into_shift_amount(self) -> Option<Self::Value>;
}

macro_rules! impl_into_shift_amount {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl IntoShiftAmount for $ty {
                type Value = u8;

                fn into_shift_amount(self) -> Option<Self::Value> {
                    let len_bits = (::core::mem::size_of::<Self>() * 8) as Self;
                    let shamt = self.checked_rem_euclid(len_bits)?;
                    Some(shamt as _)
                }
            }
        )*
    };
}
impl_into_shift_amount!(i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);
