use crate::{
    collections::arena::ArenaIndex,
    core::{ReadAs, UntypedVal, WriteAs},
    store::Stored,
    AsContextMut,
    Func,
    StoreContext,
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
    /// Creates a new [`RawRefId`] from the given `id`.
    pub fn new(id: NonZero<u32>) -> Self {
        Self {
            id,
            marker: PhantomData,
        }
    }
}

impl<T> ArenaIndex for RefId<T> {
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
    /// The [`Ref`] is a non-`null` value.
    Val(T),
    /// The [`Ref`] is `null`.
    #[default]
    Null,
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
    ( $( $reftype:ty ),* $(,)? ) => {
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
                    if u64::from(untyped) == 0 {
                        return Self::Null;
                    }
                    // Safety: This operation is safe since there are no invalid
                    //         bit patterns for [`ExternRef`] instances. Therefore
                    //         this operation cannot produce invalid [`ExternRef`]
                    //         instances even though the input [`UntypedVal`]
                    //         was modified arbitrarily.
                    unsafe { mem::transmute::<u64, Self>(untyped.into()) }
                }
            }

            impl From<$reftype> for UntypedVal {
                fn from(reftype: $reftype) -> Self {
                    // Safety: This operation is safe since there are no invalid
                    //         bit patterns for [`UntypedVal`] instances. Therefore
                    //         this operation cannot produce invalid [`UntypedVal`]
                    //         instances even if it was possible to arbitrarily modify
                    //         the input `$reftype` instance.
                    let bits = unsafe { mem::transmute::<$reftype, u64>(reftype) };
                    UntypedVal::from(bits)
                }
            }

            impl From<Nullable<$reftype>> for UntypedVal {
                fn from(reftype: Nullable<$reftype>) -> Self {
                    match reftype {
                        Nullable::Val(reftype) => UntypedVal::from(reftype),
                        Nullable::Null => UntypedVal::from(0_u64),
                    }
                }
            }
        )*
    };
}
impl_conversions! {
    ExternRef,
    Func,
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
