/// Implemented by types that can be reset for reuse.
pub trait Reset {
    /// Resets `self` for reuse.
    fn reset(&mut self);
}
