#[derive(Debug, PartialEq, Eq)]
pub enum FuzzError {
    Trap(TrapCode),
    Other,
}

impl FuzzError {
    /// Returns `true` if `self` may be of non-deterministic origin.
    pub fn is_non_deterministic(&self) -> bool {
        matches!(self, Self::Trap(TrapCode::StackOverflow) | Self::Other)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TrapCode {
    UnreachableCodeReached,
    MemoryOutOfBounds,
    TableOutOfBounds,
    IndirectCallToNull,
    IntegerDivisionByZero,
    IntegerOverflow,
    BadConversionToInteger,
    StackOverflow,
    BadSignature,
}
