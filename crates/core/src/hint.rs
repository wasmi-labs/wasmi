/// Indicates that the calling scope is unlikely to be executed.
#[cold]
#[inline]
pub fn cold() {}

/// Indicates that the condition is likely `true`.
#[inline]
pub fn likely(condition: bool) -> bool {
    if !condition {
        cold()
    }
    condition
}

/// Indicates that the condition is unlikely `true`.
#[inline]
pub fn unlikely(condition: bool) -> bool {
    if condition {
        cold()
    }
    condition
}
