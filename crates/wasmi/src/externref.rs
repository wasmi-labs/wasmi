use crate::{
    collections::arena::ArenaIndex,
    core::UntypedVal,
    store::Stored,
    AsContextMut,
    StoreContext,
};
use alloc::boxed::Box;
use core::{any::Any, mem, num::NonZeroU32};

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

#[test]
fn externref_sizeof() {
    // These assertions are important in order to convert `FuncRef`
    // from and to 64-bit `UntypedValue` instances.
    //
    // The following equation must be true:
    //     size_of(ExternRef) == size_of(ExternObject) == size_of(UntypedValue)
    use core::mem::size_of;
    assert_eq!(size_of::<ExternRef>(), size_of::<u64>());
    assert_eq!(size_of::<ExternRef>(), size_of::<ExternObject>());
}

#[test]
fn externref_null_to_zero() {
    assert_eq!(UntypedVal::from(ExternRef::null()), UntypedVal::from(0));
    assert!(ExternRef::from(UntypedVal::from(0)).is_null());
}

impl From<UntypedVal> for Option<ExternRef> {
    fn from(untyped: UntypedVal) -> Self {
        if u64::from(untyped) == 0 {
            return None;
        }
        // Safety: This operation is safe since there are no invalid
        //         bit patterns for [`ExternRef`] instances. Therefore
        //         this operation cannot produce invalid [`ExternRef`]
        //         instances even though the input [`UntypedVal`]
        //         was modified arbitrarily.
        unsafe { mem::transmute::<u64, Self>(untyped.into()) }
    }
}

impl From<ExternRef> for UntypedVal {
    fn from(externref: ExternRef) -> Self {
        // Safety: This operation is safe since there are no invalid
        //         bit patterns for [`UntypedVal`] instances. Therefore
        //         this operation cannot produce invalid [`UntypedVal`]
        //         instances even if it was possible to arbitrarily modify
        //         the input [`ExternRef`] instance.
        let bits = unsafe { mem::transmute::<ExternRef, u64>(externref) };
        UntypedVal::from(bits)
    }
}

impl From<Option<ExternRef>> for UntypedVal {
    fn from(externref: Option<ExternRef>) -> Self {
        if externref.is_none() {
            return UntypedVal::from(0_u64);
        }
        // Safety: This operation is safe since there are no invalid
        //         bit patterns for [`UntypedVal`] instances. Therefore
        //         this operation cannot produce invalid [`UntypedVal`]
        //         instances even if it was possible to arbitrarily modify
        //         the input [`ExternRef`] instance.
        let bits = unsafe { mem::transmute::<Option<ExternRef>, u64>(externref) };
        UntypedVal::from(bits)
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
