use super::{stack::ValueStack, TypedProvider, TypedValue};
use crate::{
    engine::bytecode::{AnyConst16, Const16, Provider, Register, RegisterSpanIter, Sign},
    Error,
};

/// A WebAssembly integer. Either `i32` or `i64`.
///
/// # Note
///
/// This trait provides some utility methods useful for translation.
pub trait WasmInteger:
    Copy + Eq + From<TypedValue> + Into<TypedValue> + TryInto<AnyConst16> + TryInto<Const16<Self>>
{
    /// Returns the `i16` shift amount.
    ///
    /// This computes `self % bitwsize<Self>` and returns the result as `i16` value.
    ///
    /// # Note
    ///
    /// This is commonly needed for Wasm translations of shift and rotate instructions
    /// since Wasm mandates that the shift amount operand is taken modulo the bitsize
    /// of the shifted value type.
    fn as_shift_amount(self) -> i16;

    /// Returns `true` if `self` is equal to zero (0).
    fn eq_zero(self) -> bool;
}

impl WasmInteger for i32 {
    fn as_shift_amount(self) -> i16 {
        (self % 32) as i16
    }

    fn eq_zero(self) -> bool {
        self == 0
    }
}

impl WasmInteger for u32 {
    fn as_shift_amount(self) -> i16 {
        (self % 32) as i16
    }

    fn eq_zero(self) -> bool {
        self == 0
    }
}

impl WasmInteger for i64 {
    fn as_shift_amount(self) -> i16 {
        (self % 64) as i16
    }

    fn eq_zero(self) -> bool {
        self == 0
    }
}

impl WasmInteger for u64 {
    fn as_shift_amount(self) -> i16 {
        (self % 64) as i16
    }

    fn eq_zero(self) -> bool {
        self == 0
    }
}

/// A WebAssembly float. Either `f32` or `f64`.
///
/// # Note
///
/// This trait provides some utility methods useful for translation.
pub trait WasmFloat: Copy + Into<TypedValue> + From<TypedValue> {
    /// Returns `true` if `self` is any kind of NaN value.
    fn is_nan(self) -> bool;

    /// Returns the [`Sign`] of `self`.
    fn sign(self) -> Sign;
}

impl WasmFloat for f32 {
    fn is_nan(self) -> bool {
        self.is_nan()
    }

    fn sign(self) -> Sign {
        match self.is_sign_positive() {
            true => Sign::Pos,
            false => Sign::Neg,
        }
    }
}

impl WasmFloat for f64 {
    fn is_nan(self) -> bool {
        self.is_nan()
    }

    fn sign(self) -> Sign {
        match self.is_sign_positive() {
            true => Sign::Pos,
            false => Sign::Neg,
        }
    }
}

impl Provider<u8> {
    /// Creates a new `memory` value [`Provider`] from the general [`TypedProvider`].
    pub fn new(provider: TypedProvider) -> Self {
        match provider {
            TypedProvider::Const(value) => Self::Const(u32::from(value) as u8),
            TypedProvider::Register(register) => Self::Register(register),
        }
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

impl RegisterSpanIter {
    /// Creates a [`RegisterSpanIter`] from the given slice of [`TypedProvider`] if possible.
    ///
    /// All [`TypedProvider`] must be [`Register`] and have
    /// contiguous indices for the conversion to succeed.
    ///
    /// Returns `None` if the `providers` slice is empty.
    pub fn from_providers(providers: &[TypedProvider]) -> Option<Self> {
        /// Returns the `i16` [`Register`] index if the [`TypedProvider`] is a [`Register`].
        fn register_index(provider: &TypedProvider) -> Option<i16> {
            match provider {
                TypedProvider::Register(index) => Some(index.to_i16()),
                TypedProvider::Const(_) => None,
            }
        }
        let (first, rest) = providers.split_first()?;
        let first_index = register_index(first)?;
        let mut prev_index = first_index;
        for next in rest {
            let next_index = register_index(next)?;
            if next_index.checked_sub(prev_index)? != 1 {
                return None;
            }
            prev_index = next_index;
        }
        let end_index = prev_index.checked_add(1)?;
        Some(Self::from_raw_parts(
            Register::from_i16(first_index),
            Register::from_i16(end_index),
        ))
    }
}
