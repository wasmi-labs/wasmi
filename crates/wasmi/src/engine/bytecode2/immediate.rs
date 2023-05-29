use wasmi_core::F32;

/// A 16-bit constant value.
///
/// # Note
///
/// Can be used to store information about small integer values.
/// Upon use the small 16-bit value has to be sign-extended to
/// the actual integer type, e.g. `i32` or `i64`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Immediate16(i16);

impl Immediate16 {
    /// Creates an [`Immediate16`] from the given `i32` value if possible.
    pub fn from_i32(value: i32) -> Option<Self> {
        i16::try_from(value).ok().map(Self)
    }

    /// Creates an [`Immediate16`] from the given `i64` value if possible.
    pub fn from_i64(value: i64) -> Option<Self> {
        i16::try_from(value).ok().map(Self)
    }

    /// Returns an `i32` value from `self`.
    pub fn to_i32(self) -> i32 {
        i32::from(self.0)
    }

    /// Returns an `i64` value from `self`.
    pub fn to_i64(self) -> i64 {
        i64::from(self.0)
    }
}

/// A 32-bit constant value.
///
/// # Note
///
/// Can be used to store information about small integer values.
/// Upon use the small 32-bit value has to be sign-extended to
/// the actual integer type, e.g. `i32` or `i64`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Immediate32([u8; 4]);

impl Immediate32 {
    /// Creates an [`Immediate32`] from the given `i32` value.
    pub fn from_i32(value: i32) -> Self {
        Self(value.to_le_bytes())
    }

    /// Creates an [`Immediate32`] from the given [`F32`] value.
    pub fn from_f32(value: F32) -> Self {
        Self(value.to_bits().to_le_bytes())
    }

    /// Creates an [`Immediate32`] from the given `i64` value if possible.
    pub fn from_i64(value: i64) -> Option<Self> {
        i32::try_from(value).ok().map(i32::to_be_bytes).map(Self)
    }

    /// Returns an `i32` value from `self`.
    ///
    /// # Note
    ///
    /// It is the responsibility of the user to validate type safety
    /// since access via this method is not type checked.
    pub fn to_i32(self) -> i32 {
        i32::from_le_bytes(self.0)
    }

    /// Returns an `f32` value from `self`.
    ///
    /// # Note
    ///
    /// It is the responsibility of the user to validate type safety
    /// since access via this method is not type checked.
    pub fn to_f32(self) -> F32 {
        F32::from(f32::from_le_bytes(self.0))
    }

    /// Returns an `i64` value from `self`.
    ///
    /// # Note
    ///
    /// It is the responsibility of the user to validate type safety
    /// since access via this method is not type checked.
    pub fn to_i64(self) -> i64 {
        i64::from(self.to_i32())
    }
}
