use crate::TrapCode;

/// Convert one type to another by wrapping.
pub trait WrapInto<T> {
    /// Convert one type to another by wrapping.
    fn wrap_into(self) -> T;
}

macro_rules! impl_wrap_into {
    (
        $( impl WrapInto<$into:ident> for $from:ident; )*
    ) => {
        $(
            impl WrapInto<$into> for $from {
                #[inline]
                fn wrap_into(self) -> $into {
                    self as $into
                }
            }
        )*
    };
}
impl_wrap_into! {
    impl WrapInto<i8> for i32;
    impl WrapInto<i16> for i32;
    impl WrapInto<i8> for i64;
    impl WrapInto<i16> for i64;
    impl WrapInto<i32> for i64;
}

/// Convert one type to another by extending with leading zeroes.
pub trait ExtendInto<T> {
    /// Convert one type to another by extending with leading zeroes.
    fn extend_into(self) -> T;
}

macro_rules! impl_extend_into {
    (
        $( impl ExtendInto<$into:ident> for $from:ident; )*
    ) => {
        $(
            impl ExtendInto<$into> for $from {
                #[inline]
                #[allow(clippy::cast_lossless)]
                fn extend_into(self) -> $into {
                    self as $into
                }
            }
        )*
    };
}
impl_extend_into! {
    // unsigned -> unsigned
    impl ExtendInto<u16> for u8;
    impl ExtendInto<u32> for u16;
    impl ExtendInto<u64> for u32;

    // signed -> signed
    impl ExtendInto<i16> for i8;
    impl ExtendInto<i32> for i8;
    impl ExtendInto<i64> for i8;
    impl ExtendInto<i32> for i16;
    impl ExtendInto<i64> for i16;
    impl ExtendInto<i64> for i32;

    // unsigned -> signed
    impl ExtendInto<i32> for u8;
    impl ExtendInto<i64> for u8;
    impl ExtendInto<i32> for u16;
    impl ExtendInto<i64> for u16;
    impl ExtendInto<i64> for u32;
}

/// Allows to efficiently load bytes from `memory` into a buffer.
pub trait LoadInto {
    /// Loads bytes from `memory` into `self`.
    ///
    /// # Errors
    ///
    /// Traps if the `memory` access is out of bounds.
    fn load_into(&mut self, memory: &[u8], address: usize) -> Result<(), TrapCode>;
}

impl<const N: usize> LoadInto for [u8; N] {
    #[inline]
    fn load_into(&mut self, memory: &[u8], address: usize) -> Result<(), TrapCode> {
        let slice: &Self = memory
            .get(address..)
            .and_then(|slice| slice.get(..N))
            .and_then(|slice| slice.try_into().ok())
            .ok_or(TrapCode::MemoryOutOfBounds)?;
        *self = *slice;
        Ok(())
    }
}

/// Allows to efficiently write bytes from a buffer into `memory`.
pub trait StoreFrom {
    /// Writes bytes from `self` to `memory`.
    ///
    /// # Errors
    ///
    /// Traps if the `memory` access is out of bounds.
    fn store_from(&self, memory: &mut [u8], address: usize) -> Result<(), TrapCode>;
}

impl<const N: usize> StoreFrom for [u8; N] {
    #[inline]
    fn store_from(&self, memory: &mut [u8], address: usize) -> Result<(), TrapCode> {
        let slice: &mut Self = memory
            .get_mut(address..)
            .and_then(|slice| slice.get_mut(..N))
            .and_then(|slice| slice.try_into().ok())
            .ok_or(TrapCode::MemoryOutOfBounds)?;
        *slice = *self;
        Ok(())
    }
}

/// Types that can be converted from and to little endian bytes.
pub trait LittleEndianConvert {
    /// The little endian bytes representation.
    type Bytes: Default + LoadInto + StoreFrom;

    /// Converts `self` into little endian bytes.
    fn into_le_bytes(self) -> Self::Bytes;

    /// Converts little endian bytes into `Self`.
    fn from_le_bytes(bytes: Self::Bytes) -> Self;
}

macro_rules! impl_little_endian_convert_primitive {
    ( $($primitive:ty),* $(,)? ) => {
        $(
            impl LittleEndianConvert for $primitive {
                type Bytes = [::core::primitive::u8; ::core::mem::size_of::<$primitive>()];

                #[inline]
                fn into_le_bytes(self) -> Self::Bytes {
                    <$primitive>::to_le_bytes(self)
                }

                #[inline]
                fn from_le_bytes(bytes: Self::Bytes) -> Self {
                    <$primitive>::from_le_bytes(bytes)
                }
            }
        )*
    };
}
impl_little_endian_convert_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

/// Calculates the effective address of a linear memory access.
///
/// # Errors
///
/// If the resulting effective address overflows.
fn effective_address(ptr: u64, offset: u64) -> Result<usize, TrapCode> {
    let Some(address) = ptr.checked_add(offset) else {
        return Err(TrapCode::MemoryOutOfBounds);
    };
    usize::try_from(address).map_err(|_| TrapCode::MemoryOutOfBounds)
}

/// Executes a generic `T.load` Wasm operation.
///
/// # Errors
///
/// - If `ptr + offset` overflows.
/// - If `ptr + offset` loads out of bounds from `memory`.
pub fn load<T>(memory: &[u8], ptr: u64, offset: u64) -> Result<T, TrapCode>
where
    T: LittleEndianConvert,
{
    let address = effective_address(ptr, offset)?;
    load_at::<T>(memory, address)
}

/// Executes a generic `T.load` Wasm operation.
///
/// # Errors
///
/// If `address` loads out of bounds from `memory`.
pub fn load_at<T>(memory: &[u8], address: usize) -> Result<T, TrapCode>
where
    T: LittleEndianConvert,
{
    let mut buffer = <<T as LittleEndianConvert>::Bytes as Default>::default();
    buffer.load_into(memory, address)?;
    let value: T = <T as LittleEndianConvert>::from_le_bytes(buffer);
    Ok(value)
}

/// Executes a generic `T.loadN_[s|u]` Wasm operation.
///
/// # Errors
///
/// - If `ptr + offset` overflows.
/// - If `ptr + offset` loads out of bounds from `memory`.
pub fn load_extend<T, U>(memory: &[u8], ptr: u64, offset: u64) -> Result<T, TrapCode>
where
    U: LittleEndianConvert + ExtendInto<T>,
{
    let address = effective_address(ptr, offset)?;
    load_extend_at::<T, U>(memory, address)
}

/// Executes a generic `T.loadN_[s|u]` Wasm operation.
///
/// # Errors
///
/// If `address` loads out of bounds from `memory`.
pub fn load_extend_at<T, U>(memory: &[u8], address: usize) -> Result<T, TrapCode>
where
    U: LittleEndianConvert + ExtendInto<T>,
{
    load_at::<U>(memory, address).map(ExtendInto::extend_into)
}

/// Executes a generic `T.store` Wasm operation.
///
/// # Errors
///
/// - If `ptr + offset` overflows.
/// - If `ptr + offset` stores out of bounds from `memory`.
pub fn store<T>(memory: &mut [u8], ptr: u64, offset: u64, value: T) -> Result<(), TrapCode>
where
    T: LittleEndianConvert,
{
    let address = effective_address(ptr, offset)?;
    store_at::<T>(memory, address, value)
}

/// Executes a generic `T.load` Wasm operation.
///
/// # Errors
///
/// If `address` loads out of bounds from `memory`.
pub fn store_at<T>(memory: &mut [u8], address: usize, value: T) -> Result<(), TrapCode>
where
    T: LittleEndianConvert,
{
    let buffer = <T as LittleEndianConvert>::into_le_bytes(value);
    buffer.store_from(memory, address)?;
    Ok(())
}

/// Executes a generic `T.store[N]` Wasm operation.
///
/// # Errors
///
/// - If `ptr + offset` overflows.
/// - If `ptr + offset` stores out of bounds from `memory`.
pub fn store_wrap<T, U>(memory: &mut [u8], ptr: u64, offset: u64, value: T) -> Result<(), TrapCode>
where
    T: WrapInto<U>,
    U: LittleEndianConvert,
{
    let address = effective_address(ptr, offset)?;
    store_wrap_at::<T, U>(memory, address, value)
}

/// Executes a generic `T.store[N]` Wasm operation.
///
/// # Errors
///
/// - If `address` stores out of bounds from `memory`.
pub fn store_wrap_at<T, U>(memory: &mut [u8], address: usize, value: T) -> Result<(), TrapCode>
where
    T: WrapInto<U>,
    U: LittleEndianConvert,
{
    store_at::<U>(memory, address, value.wrap_into())
}
