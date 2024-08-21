use core::{cmp, marker::PhantomData, slice};

/// Wrapper around a `T` with an alignment of 1.
///
/// # Note
///
/// Due to `#[repr(C, packed)]` it is guaranteed that the contained
/// type has an alignment of 1.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C, packed)]
pub struct Unalign<T>(T);

impl<T: Copy> Unalign<T> {
    /// Returns the underlying `T`.
    #[inline]
    pub fn get(&self) -> T {
        self.0
    }
}

impl<T> Unalign<T> {
    /// Wraps `value` in an [`Unalign`].
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Sets the wrapped value to `value`.
    pub fn set(&mut self, value: T) {
        self.0 = value;
    }
}

impl<T> From<T> for Unalign<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self(value)
    }
}

/// A slice of items with alignment of 1 and a `u16` length field.
#[derive(Debug, Copy, Clone, Eq)]
pub struct Slice<'a, T: Copy> {
    /// The pointer to the Unalign items of the slice.
    pub(crate) data: *const Unalign<T>,
    /// The number of items in the slice.
    pub(crate) len: u16,
    /// Associated lifetime capture for the compiler.
    pub(crate) lt: PhantomData<&'a [Unalign<T>]>,
}

impl<'a, T> PartialEq for Slice<'a, T>
where
    T: Copy + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<'a, T> PartialOrd for Slice<'a, T>
where
    T: Copy + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<'a, T> Ord for Slice<'a, T>
where
    T: Copy + Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<'a, T: Copy> From<&'a [Unalign<T>]> for Slice<'a, T> {
    fn from(slice: &'a [Unalign<T>]) -> Self {
        let Ok(len) = u16::try_from(slice.len()) else {
            panic!(
                "cannot crate a `Slice` with more than `u16::MAX` registers but got: {}",
                slice.len()
            )
        };
        let data = slice.as_ptr();
        Self {
            len,
            data,
            lt: PhantomData,
        }
    }
}

impl<'a, T: Copy> Slice<'a, T> {
    /// Returns a [`Slice`] from `slice` if its length is within valid bounds.
    ///
    /// Returns `None` if `slice` contains more than `u16::MAX` items.
    pub fn new(slice: &'a [Unalign<T>]) -> Option<Self> {
        let len = u16::try_from(slice.len()).ok()?;
        let data = slice.as_ptr();
        Some(Self {
            len,
            data,
            lt: PhantomData,
        })
    }

    /// Returns the number of items in `self` as `u16` value.
    pub fn len(&self) -> u16 {
        self.len
    }

    /// Returns `true` if the [`Slice`] is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a, T: Copy> AsRef<[Unalign<T>]> for Slice<'a, T> {
    fn as_ref(&self) -> &[Unalign<T>] {
        // Safety: valid `data` and `len` values for slice construction
        //         are asserted upon construction of `Slice`.
        unsafe { slice::from_raw_parts(self.data, self.len as usize) }
    }
}

/// A slice of items with alignment of 1 and a `u16` length field.
#[derive(Debug, Eq)]
pub struct SliceMut<'a, T: Copy> {
    /// The pointer to the Unalign items of the slice.
    pub(crate) data: *mut Unalign<T>,
    /// The number of items in the slice.
    pub(crate) len: u16,
    /// Associated lifetime capture for the compiler.
    pub(crate) lt: PhantomData<&'a mut [Unalign<T>]>,
}

impl<'a, T> PartialEq for SliceMut<'a, T>
where
    T: Copy + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<'a, T> PartialOrd for SliceMut<'a, T>
where
    T: Copy + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<'a, T> Ord for SliceMut<'a, T>
where
    T: Copy + Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<'a, T: Copy> From<&'a mut [Unalign<T>]> for SliceMut<'a, T> {
    fn from(slice: &'a mut [Unalign<T>]) -> Self {
        let Ok(len) = u16::try_from(slice.len()) else {
            panic!(
                "cannot crate a `Slice` with more than `u16::MAX` registers but got: {}",
                slice.len()
            )
        };
        let data = slice.as_mut_ptr();
        Self {
            len,
            data,
            lt: PhantomData,
        }
    }
}

impl<'a, T: Copy> SliceMut<'a, T> {
    /// Returns a [`Slice`] from `slice` if its length is within valid bounds.
    ///
    /// Returns `None` if `slice` contains more than `u16::MAX` items.
    pub fn new(slice: &'a mut [Unalign<T>]) -> Option<Self> {
        let len = u16::try_from(slice.len()).ok()?;
        let data = slice.as_mut_ptr();
        Some(Self {
            len,
            data,
            lt: PhantomData,
        })
    }

    /// Returns the number of items in `self` as `u16` value.
    pub fn len(&self) -> u16 {
        self.len
    }

    /// Returns `true` if the [`Slice`] is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a, T: Copy> AsRef<[Unalign<T>]> for SliceMut<'a, T> {
    fn as_ref(&self) -> &[Unalign<T>] {
        // Safety: valid `data` and `len` values for slice construction
        //         are asserted upon construction of `Slice`.
        unsafe { slice::from_raw_parts(self.data, self.len as usize) }
    }
}

impl<'a, T: Copy> AsMut<[Unalign<T>]> for SliceMut<'a, T> {
    fn as_mut(&mut self) -> &mut [Unalign<T>] {
        // Safety: valid `data` and `len` values for slice construction
        //         are asserted upon construction of `Slice`.
        unsafe { slice::from_raw_parts_mut(self.data, self.len as usize) }
    }
}
