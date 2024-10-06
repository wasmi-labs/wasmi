use super::{stack::ValueStack, Provider, TypedProvider, TypedVal};
use crate::{
    ir::{BoundedRegSpan, Const16, Reg, RegSpan, Sign},
    Error,
};

/// A WebAssembly integer. Either `i32` or `i64`.
///
/// # Note
///
/// This trait provides some utility methods useful for translation.
pub trait WasmInteger:
    Copy + Eq + From<TypedVal> + Into<TypedVal> + TryInto<Const16<Self>>
{
    /// Returns `true` if `self` is equal to zero (0).
    fn eq_zero(self) -> bool;
}

impl WasmInteger for i32 {
    fn eq_zero(self) -> bool {
        self == 0
    }
}

impl WasmInteger for u32 {
    fn eq_zero(self) -> bool {
        self == 0
    }
}

impl WasmInteger for i64 {
    fn eq_zero(self) -> bool {
        self == 0
    }
}

impl WasmInteger for u64 {
    fn eq_zero(self) -> bool {
        self == 0
    }
}

/// A WebAssembly float. Either `f32` or `f64`.
///
/// # Note
///
/// This trait provides some utility methods useful for translation.
pub trait WasmFloat: Copy + Into<TypedVal> + From<TypedVal> {
    /// Returns `true` if `self` is any kind of NaN value.
    fn is_nan(self) -> bool;

    /// Returns the [`Sign`] of `self`.
    fn sign(self) -> Sign<Self>;
}

impl WasmFloat for f32 {
    fn is_nan(self) -> bool {
        self.is_nan()
    }

    fn sign(self) -> Sign<Self> {
        Sign::from(self)
    }
}

impl WasmFloat for f64 {
    fn is_nan(self) -> bool {
        self.is_nan()
    }

    fn sign(self) -> Sign<Self> {
        Sign::from(self)
    }
}

impl Provider<Const16<u32>> {
    /// Creates a new `table` or `memory` index [`Provider`] from the general [`TypedProvider`].
    ///
    /// # Note
    ///
    /// This is a convenience function and used by translation
    /// procedures for certain Wasm `table` instructions.
    pub fn new(provider: TypedProvider, stack: &mut ValueStack) -> Result<Self, Error> {
        match provider {
            TypedProvider::Const(value) => match Const16::try_from(u32::from(value)).ok() {
                Some(value) => Ok(Self::Const(value)),
                None => {
                    let register = stack.alloc_const(value)?;
                    Ok(Self::Register(register))
                }
            },
            TypedProvider::Register(index) => Ok(Self::Register(index)),
        }
    }
}

impl TypedProvider {
    /// Returns the `i16` [`Reg`] index if the [`TypedProvider`] is a [`Reg`].
    fn register_index(&self) -> Option<i16> {
        match self {
            TypedProvider::Register(index) => Some(i16::from(*index)),
            TypedProvider::Const(_) => None,
        }
    }
}

/// Extension trait to create a [`BoundedRegSpan`] from a slice of [`TypedProvider`]s.
pub trait FromProviders: Sized {
    /// Creates a [`BoundedRegSpan`] from the given slice of [`TypedProvider`] if possible.
    ///
    /// All [`TypedProvider`] must be [`Reg`] and have
    /// contiguous indices for the conversion to succeed.
    ///
    /// Returns `None` if the `providers` slice is empty.
    fn from_providers(providers: &[TypedProvider]) -> Option<Self>;
}

impl FromProviders for BoundedRegSpan {
    fn from_providers(providers: &[TypedProvider]) -> Option<Self> {
        let (first, rest) = providers.split_first()?;
        let first_index = first.register_index()?;
        let mut prev_index = first_index;
        for next in rest {
            let next_index = next.register_index()?;
            if next_index.checked_sub(prev_index)? != 1 {
                return None;
            }
            prev_index = next_index;
        }
        let end_index = prev_index.checked_add(1)?;
        let len = (end_index - first_index) as u16;
        Some(Self::new(RegSpan::new(Reg::from(first_index)), len))
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
