/// A single 64-bit cell of the [`ValueStack`].
///
/// This stores values on the [`ValueStack`] in an untyped manner.
/// For values of type [`V128`](crate::V128) two consecutive 64-bit [`Cell`]s are used.
#[derive(Debug, Default, Copy, Clone)]
#[repr(transparent)]
pub struct Cell(u64);

/// Loads a value of type `T` from `self`.
pub trait LoadAs<T> {
    fn load_as(&self) -> T;
}

/// Stores a value of type `T` to `self`.
pub trait StoreAs<T> {
    fn store_as(&mut self, value: T);
}

macro_rules! impl_load_store_int_for_cell {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl LoadAs<$ty> for Cell {
                #[inline]
                fn load_as(&self) -> $ty {
                    self.0 as _
                }
            }

            impl StoreAs<$ty> for Cell {
                #[inline]
                #[allow(clippy::cast_lossless)]
                fn store_as(&mut self, value: $ty) {
                    self.0 = value as _;
                }
            }
        )*
    };
}
impl_load_store_int_for_cell!(u8, u16, u32, u64, i8, i16, i32, i64);

impl LoadAs<bool> for Cell {
    #[inline]
    fn load_as(&self) -> bool {
        self.0 != 0
    }
}

impl StoreAs<bool> for Cell {
    #[inline]
    #[allow(clippy::cast_lossless)]
    fn store_as(&mut self, value: bool) {
        self.0 = value as _;
    }
}

macro_rules! impl_load_store_float_for_cell {
    ( $($float_ty:ty as $bits_ty:ty),* $(,)? ) => {
        $(
            impl LoadAs<$float_ty> for Cell {
                #[inline]
                fn load_as(&self) -> $float_ty {
                    <$float_ty>::from_bits(<Cell as LoadAs<$bits_ty>>::load_as(self))
                }
            }

            impl StoreAs<$float_ty> for Cell {
                #[inline]
                fn store_as(&mut self, value: $float_ty) {
                    <Cell as StoreAs<$bits_ty>>::store_as(self, value.to_bits())
                }
            }
        )*
    }
}
impl_load_store_float_for_cell! {
    f32 as u32,
    f64 as u64,
}
