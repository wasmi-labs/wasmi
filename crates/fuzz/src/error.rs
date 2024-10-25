#[derive(Debug)]
pub enum FuzzError {
    Trap(TrapCode),
    Other,
}

#[derive(Debug, Copy, Clone)]
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
