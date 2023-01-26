use alloc::{boxed::Box, sync::Arc};
use core::{any::Any, fmt};

/// Represents a nullable opaque reference to any data within WebAssembly.
#[derive(Debug, Clone)]
pub struct ExternRef {
    // We are using an `Arc<Box<dyn Any>>` construct instead of just `Arc<dyn Any>`
    // so that `ExternRef` is pointer-sized: `size_of<ExternRef> == size_of<*const ()>`
    //
    // This is a useful property in general but also important since we do not
    // want to bloat up the `size_of` the commonly used `Value` type.
    inner: Arc<Box<dyn Any>>,
}

impl ExternRef {
    /// Creates a new instance of `ExternRef` wrapping the given value.
    pub fn new<T>(value: T) -> ExternRef
    where
        T: 'static + Any + Send + Sync,
    {
        ExternRef {
            inner: Arc::new(Box::new(value)),
        }
    }

    /// Returns a shared reference to the underlying data for this [`ExternRef`].
    pub fn data(&self) -> &dyn Any {
        &**self.inner
    }

    /// Returns the strong reference count for this [`ExternRef`].
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// Returns `true` if this [`ExternRef`] point to the same inner value as `other`.
    ///
    /// # Note
    ///
    /// This is only checks for pointer equality, and does not actually
    /// compare the inner value via its [`Eq`](core::cmp::Eq) implementation.
    pub fn ptr_eq(&self, other: &ExternRef) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl fmt::Pointer for ExternRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.inner, f)
    }
}

#[test]
fn externref_sizeof() {
    use core::mem::size_of;
    assert_eq!(size_of::<ExternRef>(), size_of::<*const ()>());
}
