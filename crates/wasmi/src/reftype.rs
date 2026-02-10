use crate::{
    AsContextMut,
    Func,
    RefType,
    StoreContext,
    core::{RawRef, RawVal, ReadAs, TypedRawRef, WriteAs},
    store::Stored,
};
use alloc::boxed::Box;
use core::{any::Any, mem, num::NonZero};

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

impl From<TypedRawRef> for Ref {
    fn from(value: TypedRawRef) -> Self {
        let raw = value.raw();
        match value.ty() {
            RefType::Func => Self::Func(raw.into()),
            RefType::Extern => Self::Extern(raw.into()),
        }
    }
}

impl From<Ref> for TypedRawRef {
    fn from(value: Ref) -> Self {
        match value {
            Ref::Func(nullable) => nullable.into(),
            Ref::Extern(nullable) => nullable.into(),
        }
    }
}

impl WriteAs<Ref> for RawVal {
    fn write_as(&mut self, value: Ref) {
        match value {
            Ref::Func(nullable) => self.write_as(nullable),
            Ref::Extern(nullable) => self.write_as(nullable),
        }
    }
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
    pub(crate) fn from_raw_parts(val: UntypedRef, ty: RefType, _store: StoreId) -> Self {
        match ty {
            RefType::Func => Ref::Func(val.into()),
            RefType::Extern => Ref::Extern(val.into()),
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
    assert_eq!(RawVal::from(<Nullable<ExternRef>>::Null), RawVal::from(0));
    assert!(<Nullable<ExternRef>>::from(RawVal::from(0)).is_null());
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
    use crate::Func;
    assert_eq!(RawVal::from(<Nullable<Func>>::Null), RawVal::from(0));
    assert!(<Nullable<Func>>::from(RawVal::from(0)).is_null());
}

macro_rules! impl_conversions {
    ( $( $reftype:ty: $ty:ident ),* $(,)? ) => {
        $(
            impl ReadAs<$reftype> for RawVal {
                fn read_as(&self) -> $reftype {
                    let bits = u64::from(*self);
                    unsafe { mem::transmute::<u64, $reftype>(bits) }
                }
            }

            impl ReadAs<Nullable<$reftype>> for RawVal {
                fn read_as(&self) -> Nullable<$reftype> {
                    let bits = u64::from(*self);
                    if bits == 0 {
                        return <Nullable<$reftype>>::Null;
                    }
                    <Nullable<$reftype>>::Val(<Self as ReadAs<$reftype>>::read_as(self))
                }
            }

            impl WriteAs<$reftype> for RawVal {
                fn write_as(&mut self, value: $reftype) {
                    let bits = unsafe { mem::transmute::<$reftype, u64>(value) };
                    self.write_as(bits)
                }
            }

            impl WriteAs<Nullable<$reftype>> for RawVal {
                fn write_as(&mut self, value: Nullable<$reftype>) {
                    match value {
                        Nullable::Null => self.write_as(0_u64),
                        Nullable::Val(value) => self.write_as(value),
                    }
                }
            }

            impl From<RawVal> for Nullable<$reftype> {
                fn from(value: RawVal) -> Self {
                    <RawVal as ReadAs<Nullable<$reftype>>>::read_as(&value)
                }
            }

            impl From<Nullable<$reftype>> for RawVal {
                fn from(reftype: Nullable<$reftype>) -> Self {
                    RawRef::from(reftype).into()
                }
            }

            impl From<RawRef> for Nullable<$reftype> {
                fn from(value: RawRef) -> Self {
                    let bits = u64::from(value);
                    if bits == 0 {
                        return <Nullable<$reftype>>::Null;
                    }
                    let value = unsafe { mem::transmute::<u64, $reftype>(bits) };
                    Nullable::Val(value)
                }
            }

            impl From<$reftype> for RawRef {
                fn from(reftype: $reftype) -> Self {
                    let bits = unsafe { mem::transmute::<$reftype, u64>(reftype) };
                    Self::from(bits)
                }
            }

            impl From<Nullable<$reftype>> for RawRef {
                fn from(reftype: Nullable<$reftype>) -> Self {
                    match reftype {
                        Nullable::Val(reftype) => RawRef::from(reftype),
                        Nullable::Null => RawRef::from(0_u64),
                    }
                }
            }

            impl From<$reftype> for TypedRawRef {
                fn from(value: $reftype) -> Self {
                    let ty = RefType::$ty;
                    let value = RawRef::from(value);
                    TypedRawRef::new(ty, value)
                }
            }

            impl From<Nullable<$reftype>> for TypedRawRef {
                fn from(value: Nullable<$reftype>) -> Self {
                    let ty = RefType::$ty;
                    let value = RawRef::from(value);
                    TypedRawRef::new(ty, value)
                }
            }
        )*
    };
}
impl_conversions! {
    ExternRef: Extern,
    Func: Func,
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
