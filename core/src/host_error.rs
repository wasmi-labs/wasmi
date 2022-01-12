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
/// #[derive(Debug)]
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
///     let my_error = MyError { code: 1312 };
///     // Note how you can just convert your errors to `wasmi::Error`
///     Err(my_error.into())
/// }
///
/// // Get a reference to the concrete error
/// match failable_fn() {
///     Err(Trap::Host(host_error)) => {
///         let my_error: &MyError = host_error.downcast_ref::<MyError>().unwrap();
///         assert_eq!(my_error.code, 1312);
///     }
///     _ => panic!(),
/// }
///
/// // get the concrete error itself
/// match failable_fn() {
///     Err(err) => {
///         let my_error = match err {
///             Trap::Host(host_error) => host_error.downcast::<MyError>().unwrap(),
///             unexpected => panic!("expected host error but found: {}", unexpected),
///         };
///         assert_eq!(my_error.code, 1312);
///     }
///     _ => panic!(),
/// }
///
/// ```
pub trait HostError: 'static + Display + Debug + DowncastSync {}
impl_downcast!(HostError);
