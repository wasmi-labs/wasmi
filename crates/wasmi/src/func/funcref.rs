use super::Func;
use crate::core::UntypedVal;
use core::mem;

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
    assert_eq!(size_of::<Func>(), size_of::<UntypedVal>());
    assert_eq!(size_of::<Func>(), size_of::<FuncRef>());
}

#[test]
fn funcref_null_to_zero() {
    assert_eq!(UntypedVal::from(FuncRef::null()), UntypedVal::from(0));
    assert!(FuncRef::from(UntypedVal::from(0)).is_null());
}

impl From<UntypedVal> for FuncRef {
    fn from(untyped: UntypedVal) -> Self {
        if u64::from(untyped) == 0 {
            return FuncRef::null();
        }
        // Safety: This union access is safe since there are no invalid
        //         bit patterns for [`FuncRef`] instances. Therefore
        //         this operation cannot produce invalid [`FuncRef`]
        //         instances even though the input [`UntypedVal`]
        //         was modified arbitrarily.
        unsafe { mem::transmute::<u64, Self>(untyped.into()) }
    }
}

impl From<FuncRef> for UntypedVal {
    fn from(funcref: FuncRef) -> Self {
        if funcref.is_null() {
            return UntypedVal::from(0_u64);
        }
        // Safety: This operation is safe since there are no invalid
        //         bit patterns for [`UntypedVal`] instances. Therefore
        //         this operation cannot produce invalid [`UntypedVal`]
        //         instances even if it was possible to arbitrarily modify
        //         the input [`FuncRef`] instance.
        let bits = unsafe { mem::transmute::<FuncRef, u64>(funcref) };
        UntypedVal::from(bits)
    }
}

impl FuncRef {
    /// Returns `true` if [`FuncRef`] is `null`.
    pub fn is_null(&self) -> bool {
        self.inner.is_none()
    }

    /// Creates a `null` [`FuncRef`].
    pub fn null() -> Self {
        Self::new(None)
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
    }

    /// Returns the inner [`Func`] if [`FuncRef`] is not `null`.
    ///
    /// Otherwise returns `None`.
    pub fn func(&self) -> Option<&Func> {
        self.inner.as_ref()
    }
}
