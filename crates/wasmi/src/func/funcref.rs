use super::Func;
use crate::{core::UntypedValue, reftype::Transposer};

/// A nullable [`Func`] reference.
#[derive(Debug, Default, Copy, Clone)]
#[repr(transparent)]
pub struct FuncRef {
    inner: Option<Func>,
}

impl From<Func> for FuncRef {
    fn from(func: Func) -> Self {
        Self::new(func)
    }
}

#[test]
fn funcref_sizeof() {
    // These assertions are important in order to convert `FuncRef`
    // from and to 64-bit `UntypedValue` instances.
    //
    // The following equation must be true:
    //     size_of(Func) == size_of(UntypedValue) == size_of(FuncRef)
    use core::mem::size_of;
    assert_eq!(size_of::<Func>(), size_of::<u64>());
    assert_eq!(size_of::<Func>(), size_of::<UntypedValue>());
    assert_eq!(size_of::<Func>(), size_of::<FuncRef>());
}

#[test]
fn funcref_null_to_zero() {
    assert_eq!(UntypedValue::from(FuncRef::null()), UntypedValue::from(0));
    assert!(FuncRef::from(UntypedValue::from(0)).is_null());
}

impl From<UntypedValue> for FuncRef {
    fn from(untyped: UntypedValue) -> Self {
        // Safety: This union access is safe since there are no invalid
        //         bit patterns for [`FuncRef`] instances. Therefore
        //         this operation cannot produce invalid [`FuncRef`]
        //         instances even though the input [`UntypedValue`]
        //         was modified arbitrarily.
        unsafe { <Transposer<Self>>::from(untyped).reftype }.canonicalize()
    }
}

impl From<FuncRef> for UntypedValue {
    fn from(funcref: FuncRef) -> Self {
        let funcref = funcref.canonicalize();
        // Safety: This operation is safe since there are no invalid
        //         bit patterns for [`UntypedValue`] instances. Therefore
        //         this operation cannot produce invalid [`UntypedValue`]
        //         instances even if it was possible to arbitrarily modify
        //         the input [`FuncRef`] instance.
        Self::from(unsafe { <Transposer<FuncRef>>::new(funcref).value })
    }
}

impl FuncRef {
    /// Returns `true` if [`FuncRef`] is `null`.
    pub fn is_null(&self) -> bool {
        self.inner.is_none()
    }

    /// Canonicalize `self` so that all `null` values have the same representation.
    ///
    /// # Note
    ///
    /// The underlying issue is that `FuncRef` has many possible values for the
    /// `null` value. However, to simplify operating on encoded `FuncRef` instances
    /// (encoded as `UntypedValue`) we want it to encode to exactly one `null`
    /// value. The most trivial of all possible `null` values is `0_u64`, therefore
    /// we canonicalize all `null` values to be represented by `0_u64`.
    fn canonicalize(self) -> Self {
        if self.is_null() {
            // Safety: This is safe since `0u64` can be bit
            //         interpreted as a valid `FuncRef` value.
            return unsafe { <Transposer<Self>>::null().reftype };
        }
        self
    }

    /// Creates a new [`FuncRef`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use wasmi::{Func, FuncRef, Store, Engine};
    /// # let engine = Engine::default();
    /// # let mut store = <Store<()>>::new(&engine, ());
    /// assert!(FuncRef::new(None).is_null());
    /// assert!(FuncRef::new(Func::wrap(&mut store, |x: i32| x)).func().is_some());
    /// ```
    pub fn new(nullable_func: impl Into<Option<Func>>) -> Self {
        Self {
            inner: nullable_func.into(),
        }
        .canonicalize()
    }

    /// Returns the inner [`Func`] if [`FuncRef`] is not `null`.
    ///
    /// Otherwise returns `None`.
    pub fn func(&self) -> Option<&Func> {
        self.inner.as_ref()
    }

    /// Creates a `null` [`FuncRef`].
    pub fn null() -> Self {
        Self::new(None).canonicalize()
    }
}
