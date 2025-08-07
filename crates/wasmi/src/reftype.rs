use crate::{
    collections::arena::ArenaIndex,
    core::UntypedVal,
    store::Stored,
    AsContextMut,
    Func,
    StoreContext,
};
use alloc::boxed::Box;
use core::{any::Any, mem, num::NonZeroU32};

/// A nullable reference type.
#[derive(Debug, Default, Copy, Clone)]
pub enum Ref<T> {
    /// The [`Ref`] is a non-`null` value.
    Val(T),
    /// The [`Ref`] is `null`.
    #[default]
    Null,
}

impl<T> Ref<T> {
    /// Returns `true` is `self` is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns `Some` if `self` is a non-`null` value.
    ///
    /// Otherwise returns `None`.
    pub fn val(&self) -> Option<&T> {
        match self {
            Ref::Val(val) => Some(val),
            Ref::Null => None,
        }
    }

    /// Converts from `&Ref<T>` to `Ref<&T>`.
    pub fn as_ref(&self) -> Ref<&T> {
        match self {
            Ref::Val(val) => Ref::Val(val),
            Ref::Null => Ref::Null,
        }
    }
}

impl<T> From<T> for Ref<T> {
    fn from(value: T) -> Self {
        Self::Val(value)
    }
}

/// A raw index to an external entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExternRefIdx(NonZeroU32);

impl ArenaIndex for ExternRefIdx {
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
pub struct ExternRefEntity {
    inner: Box<dyn 'static + Any + Send + Sync>,
}

impl ExternRefEntity {
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
pub struct ExternRef(Stored<ExternRefIdx>);

impl ExternRef {
    /// Creates a new [`ExternRef`] reference from its raw representation.
    pub(crate) fn from_inner(stored: Stored<ExternRefIdx>) -> Self {
        Self(stored)
    }

    /// Returns the raw representation of the [`ExternRef`].
    pub(crate) fn as_inner(&self) -> &Stored<ExternRefIdx> {
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
            .alloc_extern_object(ExternRefEntity::new(object))
    }

    /// Returns a shared reference to the underlying data for this [`ExternRef`].
    ///
    /// # Panics
    ///
    /// Panics if `ctx` does not own this [`ExternRef`].
    pub fn data<'a, T: 'a>(&self, ctx: impl Into<StoreContext<'a, T>>) -> &'a dyn Any {
        ctx.into().store.inner.resolve_external_object(self).data()
    }
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
    assert_eq!(size_of::<ExternRef>(), size_of::<ExternRef>());
}

#[test]
fn externref_null_to_zero() {
    assert_eq!(
        UntypedVal::from(<Ref<ExternRef>>::Null),
        UntypedVal::from(0)
    );
    assert!(<Ref<ExternRef>>::from(UntypedVal::from(0)).is_null());
}

#[test]
fn funcref_sizeof() {
    // These assertions are important in order to convert `FuncRef`
    // from and to 64-bit `UntypedValue` instances.
    //
    // The following equation must be true:
    //     size_of(Func) == size_of(UntypedValue) == size_of(FuncRef)
    use crate::Func;
    use core::mem::size_of;
    assert_eq!(size_of::<Func>(), size_of::<u64>());
    assert_eq!(size_of::<Func>(), size_of::<Ref<Func>>());
}

#[test]
fn funcref_null_to_zero() {
    use crate::Func;
    assert_eq!(UntypedVal::from(<Ref<Func>>::Null), UntypedVal::from(0));
    assert!(<Ref<Func>>::from(UntypedVal::from(0)).is_null());
}

impl From<UntypedVal> for Ref<ExternRef> {
    fn from(untyped: UntypedVal) -> Self {
        if u64::from(untyped) == 0 {
            return Ref::Null;
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

impl From<Ref<ExternRef>> for UntypedVal {
    fn from(externref: Ref<ExternRef>) -> Self {
        match externref {
            Ref::Val(externref) => UntypedVal::from(externref),
            Ref::Null => UntypedVal::from(0_u64),
        }
    }
}

impl From<UntypedVal> for Ref<Func> {
    fn from(untyped: UntypedVal) -> Self {
        if u64::from(untyped) == 0 {
            return <Ref<Func>>::Null;
        }
        // Safety: This union access is safe since there are no invalid
        //         bit patterns for [`FuncRef`] instances. Therefore
        //         this operation cannot produce invalid [`FuncRef`]
        //         instances even though the input [`UntypedVal`]
        //         was modified arbitrarily.
        unsafe { mem::transmute::<u64, Self>(untyped.into()) }
    }
}

impl From<Func> for UntypedVal {
    fn from(func: Func) -> Self {
        // Safety: This operation is safe since there are no invalid
        //         bit patterns for [`UntypedVal`] instances. Therefore
        //         this operation cannot produce invalid [`UntypedVal`]
        //         instances even if it was possible to arbitrarily modify
        //         the input [`ExternRef`] instance.
        let bits = unsafe { mem::transmute::<Func, u64>(func) };
        UntypedVal::from(bits)
    }
}

impl From<Ref<Func>> for UntypedVal {
    fn from(funcref: Ref<Func>) -> Self {
        match funcref {
            Ref::Val(funcref) => UntypedVal::from(funcref),
            Ref::Null => UntypedVal::from(0_u64),
        }
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
        let obj = ExternRef::new::<i32>(&mut store, value);
        assert_eq!(obj.data(&store).downcast_ref::<i32>(), Some(&value),);
    }
}
