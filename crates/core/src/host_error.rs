use alloc::boxed::Box;
use core::{
    any::{Any, type_name},
    fmt::{Debug, Display},
};

/// Trait that allows the host to return custom error.
///
/// It should be useful for representing custom traps,
/// troubles at instantiation time or other host specific conditions.
///
/// Types that implement this trait can automatically be converted to `wasmi::Error` and `wasmi::Trap`
/// and will be represented as a boxed `HostError`. You can then use the various methods on `wasmi::Error`
/// to get your custom error type back
///
/// # Examples
///
/// ```rust
/// use std::fmt;
/// use wasmi_core::{Trap, HostError};
///
/// #[derive(Debug, Copy, Clone)]
/// struct MyError {
///     code: u32,
/// }
///
/// impl fmt::Display for MyError {
///     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
///         write!(f, "MyError, code={}", self.code)
///     }
/// }
///
/// impl HostError for MyError { }
///
/// fn failable_fn() -> Result<(), Trap> {
///     let my_error = MyError { code: 42 };
///     // Note how you can just convert your errors to `wasmi::Error`
///     Err(my_error.into())
/// }
///
/// // Get a reference to the concrete error
/// match failable_fn() {
///     Err(trap) => {
///         let my_error: &MyError = trap.downcast_ref().unwrap();
///         assert_eq!(my_error.code, 42);
///     }
///     _ => panic!(),
/// }
///
/// // get the concrete error itself
/// match failable_fn() {
///     Err(err) => {
///         let my_error = match err.downcast_ref::<MyError>() {
///             Some(host_error) => host_error.clone(),
///             None => panic!("expected host error `MyError` but found: {}", err),
///         };
///         assert_eq!(my_error.code, 42);
///     }
///     _ => panic!(),
/// }
/// ```
pub trait HostError: 'static + Display + Debug + Any + Send + Sync {}

impl dyn HostError {
    /// Returns `true` if `self` is of type `T`.
    pub fn is<T: HostError>(&self) -> bool {
        (self as &dyn Any).is::<T>()
    }

    /// Downcasts the [`HostError`] into a shared reference to a `T` if possible.
    ///
    /// Returns `None` otherwise.
    #[inline]
    pub fn downcast_ref<T: HostError>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref::<T>()
    }

    /// Downcasts the [`HostError`] into an exclusive reference to a `T` if possible.
    ///
    /// Returns `None` otherwise.
    #[inline]
    pub fn downcast_mut<T: HostError>(&mut self) -> Option<&mut T> {
        (self as &mut dyn Any).downcast_mut::<T>()
    }

    /// Consumes `self` to downcast the [`HostError`] into the `T` if possible.
    ///
    /// # Errors
    ///
    /// If `self` cannot be downcast to `T`.
    #[inline]
    pub fn downcast<T: HostError>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if self.is::<T>() {
            let Ok(value) = (self as Box<dyn Any>).downcast::<T>() else {
                unreachable!(
                    "failed to downcast `HostError` to T (= {})",
                    type_name::<T>()
                );
            };
            Ok(value)
        } else {
            Err(self)
        }
    }
}
