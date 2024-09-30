/// Expands to
///
/// - [`core::unreachable`] if `debug_assertions` are enabled.
/// - [`core::hint::unreachable_unchecked`], otherwise.
macro_rules! unreachable_unchecked {
    ($($arg:tt)*) => {{
        match cfg!(debug_assertions) {
            true => ::core::unreachable!( $($arg)* ),
            false => ::core::hint::unreachable_unchecked(),
        }
    }};
}
pub(crate) use unreachable_unchecked;
