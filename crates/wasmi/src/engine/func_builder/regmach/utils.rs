use crate::engine::bytecode2::{Const16, Sign};
use wasmi_core::UntypedValue;

/// A WebAssembly integer. Either `i32` or `i64`.
///
/// # Note
///
/// This trait provides some utility methods useful for translation.
pub trait WasmInteger:
    Copy + Eq + From<i32> + From<UntypedValue> + Into<UntypedValue> + TryInto<Const16>
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

    /// Returns `true` if `self` is equal to minus one (-1).
    fn eq_minus_one(self) -> bool;
}

impl WasmInteger for i32 {
    fn as_shift_amount(self) -> i16 {
        (self % 32) as i16
    }

    fn eq_zero(self) -> bool {
        self == 0
    }

    fn eq_minus_one(self) -> bool {
        self == -1
    }
}

impl WasmInteger for i64 {
    fn as_shift_amount(self) -> i16 {
        (self % 64) as i16
    }

    fn eq_zero(self) -> bool {
        self == 0
    }

    fn eq_minus_one(self) -> bool {
        self == -1
    }
}

/// A WebAssembly float. Either `f32` or `f64`.
///
/// # Note
///
/// This trait provides some utility methods useful for translation.
pub trait WasmFloat: Copy + Into<UntypedValue> + From<UntypedValue> {
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
