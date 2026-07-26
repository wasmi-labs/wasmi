use core::{fmt, marker::PhantomData, ptr, ptr::NonNull};

/// A thin pointer to a `T`.
///
/// # Note
///
/// Unlike `NonNull<T>` this is always pointer sized, even where `T` is dynamically sized.
/// Consequently it only allows for operations that a thin pointer can perform: everything
/// requiring the pointer metadata of `T` has to be provided by an inherent `impl` for the
/// concrete `ThinPtr<T>`, such as the one for [`ThinPtr<InstanceEntity>`].
///
/// [`ThinPtr<InstanceEntity>`]: crate::InstanceEntity
#[repr(transparent)]
pub struct ThinPtr<T: ?Sized> {
    /// The pointer to the first byte of the pointee.
    ptr: NonNull<u8>,
    /// Marks `self` as logically containing a shared raw pointer to a `T`.
    marker: PhantomData<*const T>,
}

impl<T: ?Sized> ThinPtr<T> {
    /// Creates a new [`ThinPtr`] to `value`.
    ///
    /// # Note
    ///
    /// This exposes the provenance of `value` which spans the entire pointee, including the
    /// dynamically sized part where `T` is a dynamically sized type.
    #[inline]
    pub fn from_ref(value: &T) -> Self {
        let addr = (value as *const T).cast::<u8>().expose_provenance();
        // Safety: `addr` stems from a `&T` whose provenance has just been exposed.
        unsafe { Self::with_exposed_provenance(addr) }
    }

    /// Creates a new [`ThinPtr`] from the given exposed `addr`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `addr` originates from [`ThinPtr::expose_provenance`], or
    /// equivalently from a `&T` whose provenance has been exposed. Dereferencing the returned
    /// [`ThinPtr`] additionally requires its pointee to still be alive.
    #[inline]
    pub unsafe fn with_exposed_provenance(addr: usize) -> Self {
        let ptr = ptr::with_exposed_provenance_mut::<u8>(addr);
        Self {
            // Safety: `addr` originates from a `&T` which is never null.
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            marker: PhantomData,
        }
    }

    /// Returns the address of `self`, exposing its provenance.
    ///
    /// # Note
    ///
    /// This mirrors `<*const T>::expose_provenance` and is deliberately *not* named `addr`:
    /// `core`'s `addr` is the strict-provenance accessor that does not expose. The returned
    /// address may be fed back to [`ThinPtr::with_exposed_provenance`].
    #[inline]
    pub fn expose_provenance(self) -> usize {
        self.ptr.as_ptr().expose_provenance()
    }

    /// Returns the underlying pointer to the first byte of the pointee.
    #[inline]
    pub fn as_byte_ptr(self) -> NonNull<u8> {
        self.ptr
    }
}

// Note: the `Copy`, `Clone`, `PartialEq`, `Eq` and `Debug` impls below are hand-written since
//       deriving them would needlessly bound `T` by those traits.

impl<T: ?Sized> Clone for ThinPtr<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: ?Sized> Copy for ThinPtr<T> {}

impl<T: ?Sized> PartialEq for ThinPtr<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}
impl<T: ?Sized> Eq for ThinPtr<T> {}

impl<T: ?Sized> fmt::Debug for ThinPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ThinPtr")
            .field("ptr", &self.ptr)
            .field("marker", &::core::any::type_name::<T>())
            .finish()
    }
}
