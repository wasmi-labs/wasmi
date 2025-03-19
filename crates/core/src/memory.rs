use crate::TrapCode;

/// Convert one type to another by wrapping.
pub trait WrapInto<T> {
    /// Convert one type to another by wrapping.
    fn wrap_into(self) -> T;
}

macro_rules! impl_wrap_into {
    ($from:ident, $into:ident) => {
        impl WrapInto<$into> for $from {
            #[inline]
            fn wrap_into(self) -> $into {
                self as $into
            }
        }
    };
}

impl_wrap_into!(i32, i8);
impl_wrap_into!(i32, i16);
impl_wrap_into!(i64, i8);
impl_wrap_into!(i64, i16);
impl_wrap_into!(i64, i32);

impl_wrap_into!(u32, u32);
impl_wrap_into!(u64, u64);

/// Convert one type to another by extending with leading zeroes.
pub trait ExtendInto<T> {
    /// Convert one type to another by extending with leading zeroes.
    fn extend_into(self) -> T;
}

macro_rules! impl_extend_into {
    ($from:ident, $into:ident) => {
        impl ExtendInto<$into> for $from {
            #[inline]
            #[allow(clippy::cast_lossless)]
            fn extend_into(self) -> $into {
                self as $into
            }
        }
    };
}

impl_extend_into!(i8, i32);
impl_extend_into!(u8, i32);
impl_extend_into!(i16, i32);
impl_extend_into!(u16, i32);
impl_extend_into!(i8, i64);
impl_extend_into!(u8, i64);
impl_extend_into!(i16, i64);
impl_extend_into!(u16, i64);
impl_extend_into!(i32, i64);
impl_extend_into!(u32, i64);
impl_extend_into!(u32, u64);

// Casting to self
impl_extend_into!(u32, u32);
impl_extend_into!(u64, u64);

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
impl_little_endian_convert_primitive!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

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
    let mut buffer = <<U as LittleEndianConvert>::Bytes as Default>::default();
    buffer.load_into(memory, address)?;
    let value: T = <U as LittleEndianConvert>::from_le_bytes(buffer).extend_into();
    Ok(value)
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
    let wrapped = value.wrap_into();
    let buffer = <U as LittleEndianConvert>::into_le_bytes(wrapped);
    buffer.store_from(memory, address)?;
    Ok(())
}
