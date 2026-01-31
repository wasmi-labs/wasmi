use crate::{
    AsContextMut,
    Func,
    RefType,
    StoreContext,
    collections::arena::ArenaKey,
    core::{ReadAs, TypedRef, UntypedRef, UntypedVal, WriteAs},
    store::Stored,
};
use alloc::boxed::Box;
use core::{any, any::Any, cmp, fmt, marker::PhantomData, mem, num::NonZero};

/// The typed base type for all reference type identifiers.
pub struct RefId<T> {
    /// The underlying non-zero identifier.
    id: NonZero<u32>,
    /// Marker for the compiler to differentiate reference types.
    marker: PhantomData<fn() -> T>,
}

impl<T> fmt::Debug for RefId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RawRefId")
            .field("id", &self.id)
            .field("marker", &any::type_name::<T>())
            .finish()
    }
}

impl<T> Copy for RefId<T> {}

impl<T> Clone for RefId<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for RefId<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for RefId<T> {}

impl<T> PartialOrd for RefId<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for RefId<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl<T> RefId<T> {
    /// Creates a new [`RefId`] from the given `id`.
    pub fn new(id: NonZero<u32>) -> Self {
        Self {
            id,
            marker: PhantomData,
        }
    }
}

impl<T> ArenaKey for RefId<T> {
    fn into_usize(self) -> usize {
        u32::from(self.id).wrapping_sub(1) as usize
    }

    fn from_usize(index: usize) -> Self {
        index
            .try_into()
            .ok()
            .map(|index: u32| index.wrapping_add(1))
            .and_then(<NonZero<u32>>::new)
            .map(Self::new)
            .unwrap_or_else(|| {
                panic!(
                    "out of bounds ID for `RawRefId<{}>`: {index}",
                    any::type_name::<T>()
                )
            })
    }
}

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

impl From<TypedRef> for Ref {
    fn from(value: TypedRef) -> Self {
        let untyped = value.untyped();
        match value.ty() {
            RefType::Func => Self::Func(untyped.into()),
            RefType::Extern => Self::Extern(untyped.into()),
        }
    }
}

impl From<Ref> for TypedRef {
    fn from(value: Ref) -> Self {
        match value {
            Ref::Func(nullable) => nullable.into(),
            Ref::Extern(nullable) => nullable.into(),
        }
    }
}

impl WriteAs<Ref> for UntypedVal {
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

/// A raw index to an external entity.
pub type ExternRefIdx = RefId<ExternRef>;

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
        ctx.into().store.inner.resolve_externref(self).data()
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
        UntypedVal::from(<Nullable<ExternRef>>::Null),
        UntypedVal::from(0)
    );
    assert!(<Nullable<ExternRef>>::from(UntypedVal::from(0)).is_null());
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
    assert_eq!(size_of::<Func>(), size_of::<Nullable<Func>>());
}

#[test]
fn funcref_null_to_zero() {
    use crate::Func;
    assert_eq!(
        UntypedVal::from(<Nullable<Func>>::Null),
        UntypedVal::from(0)
    );
    assert!(<Nullable<Func>>::from(UntypedVal::from(0)).is_null());
}

macro_rules! impl_conversions {
    ( $( $reftype:ty: $ty:ident ),* $(,)? ) => {
        $(
            impl ReadAs<$reftype> for UntypedVal {
                fn read_as(&self) -> $reftype {
                    let bits = u64::from(*self);
                    unsafe { mem::transmute::<u64, $reftype>(bits) }
                }
            }

            impl ReadAs<Nullable<$reftype>> for UntypedVal {
                fn read_as(&self) -> Nullable<$reftype> {
                    let bits = u64::from(*self);
                    if bits == 0 {
                        return <Nullable<$reftype>>::Null;
                    }
                    <Nullable<$reftype>>::Val(<Self as ReadAs<$reftype>>::read_as(self))
                }
            }

            impl WriteAs<$reftype> for UntypedVal {
                fn write_as(&mut self, value: $reftype) {
                    let bits = unsafe { mem::transmute::<$reftype, u64>(value) };
                    self.write_as(bits)
                }
            }

            impl WriteAs<Nullable<$reftype>> for UntypedVal {
                fn write_as(&mut self, value: Nullable<$reftype>) {
                    match value {
                        Nullable::Null => self.write_as(0_u64),
                        Nullable::Val(value) => self.write_as(value),
                    }
                }
            }

            impl From<UntypedVal> for Nullable<$reftype> {
                fn from(untyped: UntypedVal) -> Self {
                    <UntypedVal as ReadAs<Nullable<$reftype>>>::read_as(&untyped)
                }
            }

            impl From<Nullable<$reftype>> for UntypedVal {
                fn from(reftype: Nullable<$reftype>) -> Self {
                    UntypedRef::from(reftype).into()
                }
            }

            impl From<UntypedRef> for Nullable<$reftype> {
                fn from(value: UntypedRef) -> Self {
                    let bits = u64::from(value);
                    if bits == 0 {
                        return <Nullable<$reftype>>::Null;
                    }
                    let value = unsafe { mem::transmute::<u64, $reftype>(bits) };
                    Nullable::Val(value)
                }
            }

            impl From<$reftype> for UntypedRef {
                fn from(reftype: $reftype) -> Self {
                    let bits = unsafe { mem::transmute::<$reftype, u64>(reftype) };
                    Self::from(bits)
                }
            }

            impl From<Nullable<$reftype>> for UntypedRef {
                fn from(reftype: Nullable<$reftype>) -> Self {
                    match reftype {
                        Nullable::Val(reftype) => UntypedRef::from(reftype),
                        Nullable::Null => UntypedRef::from(0_u64),
                    }
                }
            }

            impl From<$reftype> for TypedRef {
                fn from(value: $reftype) -> Self {
                    let ty = RefType::$ty;
                    let value = UntypedRef::from(value);
                    TypedRef::new(ty, value)
                }
            }

            impl From<Nullable<$reftype>> for TypedRef {
                fn from(value: Nullable<$reftype>) -> Self {
                    let ty = RefType::$ty;
                    let value = UntypedRef::from(value);
                    TypedRef::new(ty, value)
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
