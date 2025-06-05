use crate::{
    core::{Typed, TypedVal, ValType},
    ir::{Const16, Sign},
    ExternRef,
    FuncRef,
};

macro_rules! impl_typed_for {
    ( $( $ty:ident ),* $(,)? ) => {
        $(
            impl Typed for $ty {
                const TY: ValType = crate::core::ValType::$ty;
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
    FuncRef,
    ExternRef,
}

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
