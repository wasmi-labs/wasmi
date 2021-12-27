use core::{fmt, fmt::Display};

/// Errors that can occur upon operating with resizable limits.
#[derive(Debug)]
#[non_exhaustive]
pub struct LimitsError;

impl Display for LimitsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "encountered invalid resizable limit")
    }
}

/// Memory and table limits.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TableType {
    initial: usize,
    maximum: Option<usize>,
}

impl TableType {
    /// Creates a new resizable limit.
    ///
    /// # Panics
    ///
    /// - If the `initial` limit is greater than the `maximum` limit if any.
    pub fn new(initial: usize, maximum: Option<usize>) -> Self {
        if let Some(maximum) = maximum {
            assert!(initial <= maximum);
        }
        Self { initial, maximum }
    }

    /// Returns the initial limit.
    pub fn initial(self) -> usize {
        self.initial
    }

    /// Returns the maximum limit if any.
    pub fn maximum(self) -> Option<usize> {
        self.maximum
    }
}
