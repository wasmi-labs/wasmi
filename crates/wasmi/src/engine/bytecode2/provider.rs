use super::{Const32, ConstRef, Register};

/// A reference to a [`ProviderSlice`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ProviderSliceRef(u32);

/// A [`Provider`] slice.
pub struct ProviderSlice<'a> {
    /// The [`Provider`] values of the slice.
    values: &'a [Provider],
}

/// A provider value.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Provider {
    /// A [`Register`] value.
    Register(Register),
    /// A reference to a constant value.
    ConstRef(ConstRef),
    /// A 32-bit constant value.
    Const32(Const32),
    /// A 32-bit encoded constant `i64` value.
    Const32AsI64(Const32),
}
