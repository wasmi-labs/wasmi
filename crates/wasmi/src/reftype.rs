use crate::{
    AsContextMut,
    Func,
    RefType,
    StoreContext,
    core::RawRef,
    handle::RawHandle,
    store::{AsStoreId, Stored},
};
use alloc::boxed::Box;
use core::{any::Any, num::NonZero};

/// A nullable reference type.
#[derive(Debug, Default, Copy, Clone)]
pub enum Nullable<T> {
    /// The [`Ref`] is `null`.
    #[default]
    Null,
    /// The [`Ref`] is a non-`null` value.
    Val(T),
}

impl<T> Nullable<T> {
    /// Returns `true` is `self` is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns `Some` if `self` is a non-`null` value.
    ///
    /// Otherwise returns `None`.
    pub fn val(&self) -> Option<&T> {
        match self {
            Self::Val(val) => Some(val),
            Self::Null => None,
        }
    }

    /// Converts from `&Ref<T>` to `Ref<&T>`.
    pub fn as_ref(&self) -> Nullable<&T> {
        match self {
            Self::Val(val) => Nullable::Val(val),
            Self::Null => Nullable::Null,
        }
    }
}

impl<T> From<T> for Nullable<T> {
    fn from(value: T) -> Self {
        Self::Val(value)
    }
}

impl<T> From<Nullable<T>> for Option<T> {
    fn from(nullable: Nullable<T>) -> Self {
        match nullable {
            Nullable::Val(value) => Some(value),
            Nullable::Null => None,
        }
    }
}

/// A Wasm reference.
#[derive(Debug, Copy, Clone)]
pub enum Ref {
    /// A Wasm `funcref`.
    Func(Nullable<Func>),
    /// A Wasm `externref`.
    Extern(Nullable<ExternRef>),
}

impl From<Nullable<Func>> for Ref {
    fn from(value: Nullable<Func>) -> Self {
        Self::Func(value)
    }
}

impl From<Nullable<ExternRef>> for Ref {
    fn from(value: Nullable<ExternRef>) -> Self {
        Self::Extern(value)
    }
}

impl Ref {
    /// Create a [`Ref`] from its raw parts.
    pub(crate) fn from_raw_parts(val: RawRef, ty: RefType, store: impl AsStoreId) -> Self {
        match ty {
            RefType::Func => Ref::Func(<Nullable<Func>>::from_raw_parts(val, store)),
            RefType::Extern => Ref::Extern(<Nullable<ExternRef>>::from_raw_parts(val, store)),
        }
    }

    /// Creates new default value of given type.
    #[inline]
    pub fn default_for_ty(ty: RefType) -> Self {
        match ty {
            RefType::Func => Self::from(<Nullable<Func>>::Null),
            RefType::Extern => Self::from(<Nullable<ExternRef>>::Null),
        }
    }

    /// Returns `true` if `self` is a `null` rerefence.
    #[inline]
    pub fn is_null(&self) -> bool {
        match self {
            Self::Func(nullable) => nullable.is_null(),
            Self::Extern(nullable) => nullable.is_null(),
        }
    }

    /// Returns `true` if `self` is a `null` rerefence.
    #[inline]
    pub fn is_non_null(&self) -> bool {
        !self.is_null()
    }

    /// Creates a new `null` reference of type `ty`.
    #[inline]
    pub fn null(ty: RefType) -> Self {
        match ty {
            RefType::Extern => Self::Extern(Nullable::Null),
            RefType::Func => Self::Func(Nullable::Null),
        }
    }

    /// Returns the [`RefType`] of `self`.
    #[inline]
    pub fn ty(&self) -> RefType {
        match self {
            Self::Func(_) => RefType::Func,
            Self::Extern(_) => RefType::Extern,
        }
    }

    /// Returns `true` if `self` is a `funcref`.
    #[inline]
    pub fn is_func(&self) -> bool {
        matches!(self, Self::Func(_))
    }

    /// Returns `true` if `self` is a `externref`.
    #[inline]
    pub fn is_extern(&self) -> bool {
        matches!(self, Self::Extern(_))
    }

    /// Returns `Some` if `self` is a `funcref`.
    ///
    /// Otherwise returns `None`.
    #[inline]
    pub fn as_func(&self) -> Option<Nullable<&Func>> {
        if let Self::Func(nullable) = self {
            return Some(nullable.as_ref());
        }
        None
    }

    /// Returns `Some` if `self` is a `externref`.
    ///
    /// Otherwise returns `None`.
    #[inline]
    pub fn as_extern(&self) -> Option<Nullable<&ExternRef>> {
        if let Self::Extern(nullable) = self {
            return Some(nullable.as_ref());
        }
        None
    }

    /// Get the underlying [`Func`] reference.
    ///
    /// # Note
    ///
    /// - Returns `None` if this `Ref` is a null `func` reference.
    /// - Returns `Some(_)` if this `Ref` is a non-null `func` reference.
    ///
    /// # Panics
    ///
    /// If `self` is another kind of reference.
    #[inline]
    pub fn unwrap_func(&self) -> Nullable<&Func> {
        self.as_func()
            .expect("`Ref::unwrap_func` on non-func reference")
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

define_handle! {
    /// Represents an opaque reference to any data within WebAssembly.
    struct ExternRef(NonZero<u32>, Stored) => ExternRefEntity;
}

impl ExternRef {
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
        ctx.into().store.inner.resolve_externref(self).data()
    }
}

#[test]
fn externref_sizeof() {
    // These assertions are important in order to convert `FuncRef`
    // from and to 64-bit `RawValue` instances.
    //
    // The following equation must be true:
    //     size_of(ExternRef) == size_of(ExternObject) == size_of(RawValue)
    use core::mem::size_of;
    assert_eq!(size_of::<ExternRef>(), size_of::<u64>());
    assert_eq!(size_of::<ExternRef>(), size_of::<ExternRef>());
}

#[test]
fn externref_null_to_zero() {
    use crate::Store;
    let store = <Store<()>>::default();
    let null = <Nullable<ExternRef>>::Null;
    assert_eq!(null.unwrap_raw(&store), Some(RawRef::from(0)));
    assert!(<Nullable<ExternRef>>::from_raw_parts(RawRef::from(0), &store).is_null());
}

#[test]
fn funcref_sizeof() {
    // These assertions are important in order to convert `FuncRef`
    // from and to 64-bit `RawValue` instances.
    //
    // The following equation must be true:
    //     size_of(Func) == size_of(RawValue) == size_of(FuncRef)
    use crate::Func;
    use core::mem::size_of;
    assert_eq!(size_of::<Func>(), size_of::<u64>());
    assert_eq!(size_of::<Func>(), size_of::<Nullable<Func>>());
}

#[test]
fn funcref_null_to_zero() {
    use crate::{Func, Store};
    let store = <Store<()>>::default();
    let null = <Nullable<Func>>::Null;
    assert_eq!(null.unwrap_raw(&store), Some(RawRef::from(0)));
    assert!(<Nullable<Func>>::from_raw_parts(RawRef::from(0), &store).is_null());
}

macro_rules! impl_conversions {
    ( $( $reftype:ty: $ty:ident ),* $(,)? ) => {
        $(
            impl $reftype {
                /// Unwraps the underlying [`RawRef`] if `self` originates from `store`.
                ///
                /// Otherwise returns `None`.
                pub(crate) fn unwrap_raw(&self, store: impl AsStoreId) -> Option<RawRef> {
                    use crate::Handle as _;
                    let value = store.unwrap(self.as_raw())?;
                    Some(RawRef::from(value.raw().get()))
                }

                #[doc = concat!("Create a [`", stringify!($reftype), "`] from its raw parts.")]
                pub(crate) fn from_raw_parts(value: NonZero<u32>, store: impl AsStoreId) -> Self {
                    let raw_handle = <RawHandle<$reftype>>::new(value);
                    <$reftype as $crate::Handle>::from_raw(store.wrap(raw_handle))
                }
            }

            impl Nullable<$reftype> {
                /// Unwraps the underlying [`RawRef`] if `self` originates from `store`.
                ///
                /// Otherwise returns `None`.
                pub(crate) fn unwrap_raw(&self, store: impl AsStoreId) -> Option<RawRef> {
                    match self {
                        Self::Null => Some(RawRef::from(0_u32)),
                        Self::Val(value) => <$reftype>::unwrap_raw(value, store),
                    }
                }

                #[doc = concat!("Create a [`Nullable<", stringify!($reftype), ">`] from its raw parts.")]
                pub(crate) fn from_raw_parts(val: RawRef, store: impl AsStoreId) -> Self {
                    match <NonZero<u32>>::new(u32::from(val)) {
                        Some(value) => Self::Val(<$reftype>::from_raw_parts(value, store)),
                        None => Self::Null,
                    }
                }
            }
        )*
    };
}
impl_conversions! {
    ExternRef: Extern,
    Func: Func,
}

impl Ref {
    /// Unwraps the underlying [`RawRef`] if `self` originates from `store`.
    ///
    /// Otherwise returns `None`.
    pub(crate) fn unwrap_raw(&self, store: impl AsStoreId) -> Option<RawRef> {
        match self {
            Self::Func(value) => value.unwrap_raw(store),
            Self::Extern(value) => value.unwrap_raw(store),
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
