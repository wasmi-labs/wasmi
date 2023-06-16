use wasmi_core::F32;

/// A 16-bit constant value.
///
/// # Note
///
/// Can be used to store information about small integer values.
/// Upon use the small 16-bit value has to be sign-extended to
/// the actual integer type, e.g. `i32` or `i64`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Const16(i16);

/// Error that may occur upon converting values to [`Const16`].
#[derive(Debug, Copy, Clone)]
pub struct OutOfBoundsConst16;

impl TryFrom<i32> for Const16 {
    type Error = OutOfBoundsConst16;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::from_i32(value).ok_or(OutOfBoundsConst16)
    }
}

impl TryFrom<i64> for Const16 {
    type Error = OutOfBoundsConst16;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::from_i64(value).ok_or(OutOfBoundsConst16)
    }
}

impl Const16 {
    /// Creates an [`Const16`] from the given `i16` value.
    pub fn from_i16(value: i16) -> Self {
        Self(value)
    }

    /// Creates an [`Const16`] from the given `i32` value if possible.
    pub fn from_i32(value: i32) -> Option<Self> {
        i16::try_from(value).ok().map(Self)
    }

    /// Creates an [`Const16`] from the given `i64` value if possible.
    pub fn from_i64(value: i64) -> Option<Self> {
        i16::try_from(value).ok().map(Self)
    }

    /// Creates an [`Const16`] from the given `u32` value if possible.
    pub fn from_u32(value: u32) -> Option<Self> {
        let value = u16::try_from(value).ok()? as i16;
        Some(Self(value))
    }

    /// Returns an `i32` value from `self`.
    pub fn to_i32(self) -> i32 {
        i32::from(self.0)
    }

    /// Returns an `i64` value from `self`.
    pub fn to_i64(self) -> i64 {
        i64::from(self.0)
    }

    /// Returns an `u32` value from `self`.
    pub fn to_u32(self) -> u32 {
        u32::from(self.0 as u16)
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
#[repr(align(2))] // 2-byte alignment is sufficient for `wasmi` bytecode
pub struct Const32([u8; 4]);

/// Error that may occur upon converting values to [`Const32`].
#[derive(Debug, Copy, Clone)]
pub struct OutOfBoundsConst32;

impl TryFrom<i64> for Const32 {
    type Error = OutOfBoundsConst32;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::from_i64(value).ok_or(OutOfBoundsConst32)
    }
}

impl From<i32> for Const32 {
    fn from(value: i32) -> Self {
        Self::from_i32(value)
    }
}

impl From<u32> for Const32 {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<f32> for Const32 {
    fn from(value: f32) -> Self {
        Self::from(F32::from(value))
    }
}

impl From<F32> for Const32 {
    fn from(value: F32) -> Self {
        Self::from_f32(value)
    }
}

impl Const32 {
    /// Creates an [`Const32`] from the given `u32` value.
    pub fn from_u32(value: u32) -> Self {
        Self(value.to_ne_bytes())
    }

    /// Creates an [`Const32`] from the given `i32` value.
    pub fn from_i32(value: i32) -> Self {
        Self::from_u32(value as u32)
    }

    /// Creates an [`Const32`] from the given [`F32`] value.
    pub fn from_f32(value: F32) -> Self {
        Self::from_u32(value.to_bits())
    }

    /// Creates an [`Const32`] from the given `i64` value if possible.
    pub fn from_i64(value: i64) -> Option<Self> {
        i32::try_from(value).ok().map(Self::from_i32)
    }

    /// Returns an `u32` value from `self`.
    ///
    /// # Note
    ///
    /// It is the responsibility of the user to validate type safety
    /// since access via this method is not type checked.
    pub fn to_u32(self) -> u32 {
        u32::from_ne_bytes(self.0)
    }

    /// Returns an `i32` value from `self`.
    ///
    /// # Note
    ///
    /// It is the responsibility of the user to validate type safety
    /// since access via this method is not type checked.
    pub fn to_i32(self) -> i32 {
        self.to_u32() as i32
    }

    /// Returns an `f32` value from `self`.
    ///
    /// # Note
    ///
    /// It is the responsibility of the user to validate type safety
    /// since access via this method is not type checked.
    pub fn to_f32(self) -> F32 {
        F32::from(f32::from_bits(self.to_u32()))
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
