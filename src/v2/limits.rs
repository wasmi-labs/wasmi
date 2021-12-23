use parity_wasm as pwasm;
use core::fmt;
use core::fmt::Display;

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
pub struct ResizableLimits {
    initial: usize,
    maximum: Option<usize>,
}

impl TryFrom<pwasm::elements::ResizableLimits> for ResizableLimits {
    type Error = LimitsError;

    fn try_from(limits: pwasm::elements::ResizableLimits) -> Result<Self, Self::Error> {
        let initial = limits.initial() as usize;
        let maximum = limits.maximum().map(|maximum| maximum as usize);
        Self::new(initial, maximum)
    }
}

impl ResizableLimits {
    /// Creates a new resizable limit.
    pub fn new(initial: usize, maximum: Option<usize>) -> Result<Self, LimitsError> {
        if let Some(maximum) = maximum {
            if initial > maximum {
                return Err(LimitsError);
            }
        }
        Ok(Self { initial, maximum })
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
