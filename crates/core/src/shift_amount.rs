#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ShiftAmount(u8);

impl From<u8> for ShiftAmount {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<ShiftAmount> for u8 {
    #[inline]
    fn from(value: ShiftAmount) -> Self {
        value.0
    }
}

pub trait IntoShiftAmount {
    /// The source type expected by the Wasm specification.
    type ShiftSource: Copy;

    /// Returns `self` wrapped into a proper shift amount for `Self`.
    ///
    /// Returns `None` if the resulting shift amount is 0, a.k.a. a no-op.
    fn into_shift_amount(source: Self::ShiftSource) -> Option<ShiftAmount>;
}

macro_rules! impl_into_shift_amount {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl IntoShiftAmount for $ty {
                type ShiftSource = Self;

                fn into_shift_amount(source: Self::ShiftSource) -> Option<ShiftAmount> {
                    let len_bits = (::core::mem::size_of::<Self::ShiftSource>() * 8) as Self;
                    let shamt = source.checked_rem_euclid(len_bits)?;
                    Some(ShiftAmount(shamt as _))
                }
            }
        )*
    };
}
impl_into_shift_amount!(i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);

macro_rules! impl_into_simd_shift_amount {
    ( $([$ty:ty; $n:literal]),* $(,)? ) => {
        $(
            impl IntoShiftAmount for [$ty; $n] {
                type ShiftSource = u32;

                fn into_shift_amount(source: Self::ShiftSource) -> Option<ShiftAmount> {
                    let len_bits = (::core::mem::size_of::<$ty>() * 8) as Self::ShiftSource;
                    let shamt = source.checked_rem_euclid(len_bits)?;
                    Some(ShiftAmount(shamt as _))
                }
            }
        )*
    };
}
impl_into_simd_shift_amount!([u8; 16], [u16; 8], [u32; 4], [u64; 2]);
