/// Bail out early in case the current code is unreachable.
///
/// # Note
///
/// - This should be prepended to most Wasm operator translation procedures.
/// - If we are in unreachable code most Wasm translation is skipped. Only
///   certain control flow operators such as `End` are going through the
///   translation process. In particular the `End` operator may end unreachable
///   code blocks.
macro_rules! bail_unreachable {
    ($this:ident) => {{
        if !$this.reachable {
            return Ok(());
        }
    }};
}

/// Implemented by types that can be reset for reuse.
pub trait Reset: Sized {
    /// Resets `self` for reuse.
    fn reset(&mut self);

    /// Returns `self` in resetted state.
    #[must_use]
    fn into_reset(self) -> Self {
        let mut this = self;
        this.reset();
        this
    }
}
