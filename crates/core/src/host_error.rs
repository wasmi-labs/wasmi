use core::fmt::{Debug, Display};
use downcast_rs::{impl_downcast, DowncastSync};

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
pub trait HostError: 'static + Display + Debug + DowncastSync {}
impl_downcast!(HostError);

#[derive(Debug)]
pub enum HostErrType {
    WithReason(String),
    I32Exit(i32),
}

impl Display for HostErrType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HostErrType::WithReason(r) => f.debug_tuple("HostErrorWithReason").field(r).finish(),
            HostErrType::I32Exit(i) => f
                .debug_tuple("HostErrorI32Exit")
                .field(&format!("{i}"))
                .finish(),
        }
    }
}

impl HostErrType {
    pub fn new_with_reason(reason: String) -> Self {
        Self::WithReason(reason)
    }

    pub fn new_132_exit(i: i32) -> Self {
        Self::I32Exit(i)
    }
}

impl HostError for HostErrType {}
