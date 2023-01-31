use crate::{store::Stored, AsContextMut, StoreContext};
use alloc::boxed::Box;
use core::{any::Any, num::NonZeroU32};
use wasmi_arena::ArenaIndex;
use wasmi_core::UntypedValue;

/// A raw index to a function entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExternObjectIdx(NonZeroU32);

impl ArenaIndex for ExternObjectIdx {
    fn into_usize(self) -> usize {
        self.0.get().wrapping_sub(1) as usize
    }

    fn from_usize(index: usize) -> Self {
        index
            .try_into()
            .ok()
            .map(|index: u32| index.wrapping_add(1))
            .and_then(NonZeroU32::new)
            .map(Self)
            .unwrap_or_else(|| panic!("out of bounds extern object index {index}"))
    }
}

/// An externally defined object.
#[derive(Debug)]
pub struct ExternObjectEntity {
    inner: Box<dyn 'static + Any + Send + Sync>,
}

impl ExternObjectEntity {
    /// Creates a new instance of `ExternRef` wrapping the given value.
    pub fn new<T>(object: T) -> Self
    where
        T: 'static + Any + Send + Sync,
    {
        Self {
            inner: Box::new(object),
        }
    }

    /// Returns a shared reference to the external object.
    pub fn data(&self) -> &dyn Any {
        &*self.inner
    }
}

/// Represents an opaque reference to any data within WebAssembly.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct ExternObject(Stored<ExternObjectIdx>);

impl ExternObject {
    /// Creates a new [`ExternObject`] reference from its raw representation.
    pub(crate) fn from_inner(stored: Stored<ExternObjectIdx>) -> Self {
        Self(stored)
    }

    /// Returns the raw representation of the [`ExternObject`].
    pub(crate) fn as_inner(&self) -> &Stored<ExternObjectIdx> {
        &self.0
    }

    /// Creates a new instance of `ExternRef` wrapping the given value.
    pub fn new<T>(mut ctx: impl AsContextMut, object: T) -> Self
    where
        T: 'static + Any + Send + Sync,
    {
        ctx.as_context_mut()
            .store
            .inner
            .alloc_extern_object(ExternObjectEntity::new(object))
    }

    /// Returns a shared reference to the underlying data for this [`ExternRef`].
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`ExternObject`].
    pub fn data<'a, T: 'a>(&self, ctx: impl Into<StoreContext<'a, T>>) -> &'a dyn Any {
        ctx.into().store.inner.resolve_external_object(self).data()
    }
}

/// Represents a nullable opaque reference to any data within WebAssembly.
#[derive(Debug, Default, Copy, Clone)]
#[repr(transparent)]
pub struct ExternRef {
    inner: Option<ExternObject>,
}

/// Type used to convert between [`ExternRef`] and [`UntypedValue`].
union Transposer {
    externref: ExternRef,
    untyped: UntypedValue,
}

#[test]
fn externref_sizeof() {
    // These assertions are important in order to convert `FuncRef`
    // from and to 64-bit `UntypedValue` instances.
    //
    // The following equation must be true:
    //     size_of(ExternRef) == size_of(ExternObject) == size_of(UntypedValue)
    use core::mem::size_of;
    assert_eq!(size_of::<ExternRef>(), size_of::<UntypedValue>());
    assert_eq!(size_of::<ExternRef>(), size_of::<ExternObject>());
}

#[test]
fn externref_null_to_zero() {
    assert_eq!(UntypedValue::from(ExternRef::null()), UntypedValue::from(0));
    assert!(ExternRef::from(UntypedValue::from(0)).is_null());
}

impl From<UntypedValue> for ExternRef {
    fn from(untyped: UntypedValue) -> Self {
        // Safety: This operation is safe since there are no invalid
        //         bit patterns for [`ExternRef`] instances. Therefore
        //         this operation cannot produce invalid [`ExternRef`]
        //         instances even though the input [`UntypedValue`]
        //         was modified arbitrarily.
        unsafe { Transposer { untyped }.externref }.canonicalize()
    }
}

impl From<ExternRef> for UntypedValue {
    fn from(externref: ExternRef) -> Self {
        let externref = externref.canonicalize();
        // Safety: This operation is safe since there are no invalid
        //         bit patterns for [`UntypedValue`] instances. Therefore
        //         this operation cannot produce invalid [`UntypedValue`]
        //         instances even if it was possible to arbitrarily modify
        //         the input [`ExternRef`] instance.
        unsafe { Transposer { externref }.untyped }
    }
}

impl ExternRef {
    /// Creates a new [`ExternRef`] wrapping the given value.
    pub fn new<T>(ctx: impl AsContextMut, object: impl Into<Option<T>>) -> Self
    where
        T: 'static + Any + Send + Sync,
    {
        object
            .into()
            .map(|object| ExternObject::new(ctx, object))
            .map(Self::from_object)
            .unwrap_or_else(Self::null)
            .canonicalize()
    }

    /// Canonicalize `self` so that all `null` values have the same representation.
    ///
    /// # Note
    ///
    /// The underlying issue is that `ExternRef` has many possible values for the
    /// `null` value. However, to simplify operating on encoded `ExternRef` instances
    /// (encoded as `UntypedValue`) we want it to encode to exactly one `null`
    /// value. The most trivial of all possible `null` values is `0_u64`, therefore
    /// we canonicalize all `null` values to be represented by `0_u64`.
    fn canonicalize(self) -> Self {
        if self.is_null() {
            // Safety: This is safe since `0u64` can be bit
            //         interpreted as a valid `ExternRef` value.
            return unsafe {
                Transposer {
                    untyped: UntypedValue::from(0u64),
                }
                .externref
            };
        }
        self
    }

    /// Creates a new [`ExternRef`] to the given [`ExternObject`].
    fn from_object(object: ExternObject) -> Self {
        Self {
            inner: Some(object),
        }
    }

    /// Creates a new [`ExternRef`] which is `null`.
    pub fn null() -> Self {
        Self { inner: None }.canonicalize()
    }

    /// Returns `true` if [`ExternRef`] is `null`.
    pub fn is_null(&self) -> bool {
        self.inner.is_none()
    }

    /// Returns a shared reference to the underlying data for this [`ExternRef`].
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`ExternRef`].
    pub fn data<'a, T: 'a>(&self, ctx: impl Into<StoreContext<'a, T>>) -> Option<&'a dyn Any> {
        self.inner.map(|object| object.data(ctx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Engine, Store};

    #[test]
    fn it_works() {
        let engine = Engine::default();
        let mut store = <Store<()>>::new(&engine, ());
        let value = 42_i32;
        let obj = ExternObject::new::<i32>(&mut store, value);
        assert_eq!(obj.data(&store).downcast_ref::<i32>(), Some(&value),);
    }
}
