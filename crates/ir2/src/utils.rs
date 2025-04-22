use crate::{Address, Memory, Offset, Op, Reg, Stack};
use core::ops::Deref;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct RefAccess<T>(T);

impl<T> RefAccess<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    pub unsafe fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for RefAccess<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Trait to query the [`OpCode`][`crate::OpCode`] of operators.
///
/// Implemented by [`Op`][crate::Op] and all operators found in the [`crate::Op`] module.
pub trait OperatorCode {
    /// Returns the [`OpCode`][crate::OpCode] associated to `self`.
    fn op_code(&self) -> crate::OpCode;
}

/// Trait implemented by all operator types.
pub trait Operator: Copy + OperatorCode + Into<Op> {}
impl<T> Operator for T where T: Copy + OperatorCode + Into<Op> {}

/// Class of unary operators.
pub trait UnaryOperator {
    /// The name of the operator class.
    const NAME: &'static str;

    /// The operator variant that takes a [`Reg`] and returns a [`Reg`].
    type OpRr: Operator;

    /// The operator variant that takes a [`Stack`] and returns a [`Reg`].
    type OpRs: Operator;

    /// The operator variant that takes a [`Reg`] and returns a [`Stack`].
    type OpSr: Operator;

    /// The operator variant that takes a [`Stack`] and returns a [`Stack`].
    type OpSs: Operator;

    /// Creates the operator variant that takes a [`Reg`] and returns a [`Reg`].
    fn make_rr(result: Reg, input: Reg) -> Self::OpRr;
    /// Creates the operator variant that takes a [`Stack`] and returns a [`Reg`].
    fn make_rs(result: Reg, input: Stack) -> Self::OpRs;
    /// Creates the operator variant that takes a [`Reg`] and returns a [`Stack`].
    fn make_sr(result: Stack, input: Reg) -> Self::OpSr;
    /// Creates the operator variant that takes a [`Stack`] and returns a [`Stack`].
    fn make_ss(result: Stack, input: Stack) -> Self::OpSs;
}

/// Class of commutative binary operators.
pub trait BinaryCommutativeOperator {
    /// The name of the operator class.
    const NAME: &'static str;

    /// The type of immediate input operands.
    type Imm;

    /// The operator variant with signature: `fn(Reg, Imm) -> Reg`
    type OpRri: Operator;
    /// The operator variant with signature: `fn(Reg, Stack) -> Reg`
    type OpRrs: Operator;
    /// The operator variant with signature: `fn(Stack, Imm) -> Reg`
    type OpRsi: Operator;
    /// The operator variant with signature: `fn(Stack, Stack) -> Reg`
    type OpRss: Operator;
    /// The operator variant with signature: `fn(Reg, Imm) -> Stack`
    type OpSri: Operator;
    /// The operator variant with signature: `fn(Reg, Stack) -> Stack`
    type OpSrs: Operator;
    /// The operator variant with signature: `fn(Stack, Imm) -> Stack`
    type OpSsi: Operator;
    /// The operator variant with signature: `fn(Stack, Stack) -> Stack`
    type OpSss: Operator;

    /// Creates the operator variant with signature: `fn(Reg, Imm) -> Reg`
    fn make_rri(result: Reg, lhs: Reg, rhs: Self::Imm) -> Self::OpRri;
    /// Creates the operator variant with signature: `fn(Reg, Stack) -> Reg`
    fn make_rrs(result: Reg, lhs: Reg, rhs: Stack) -> Self::OpRrs;
    /// Creates the operator variant with signature: `fn(Stack, Imm) -> Reg`
    fn make_rsi(result: Reg, lhs: Stack, rhs: Self::Imm) -> Self::OpRsi;
    /// Creates the operator variant with signature: `fn(Stack, Stack) -> Reg`
    fn make_rss(result: Reg, lhs: Stack, rhs: Stack) -> Self::OpRss;
    /// Creates the operator variant with signature: `fn(Reg, Imm) -> Stack`
    fn make_sri(result: Stack, lhs: Reg, rhs: Self::Imm) -> Self::OpSri;
    /// Creates the operator variant with signature: `fn(Reg, Stack) -> Stack`
    fn make_srs(result: Stack, lhs: Reg, rhs: Stack) -> Self::OpSrs;
    /// Creates the operator variant with signature: `fn(Stack, Imm) -> Stack`
    fn make_ssi(result: Stack, lhs: Stack, rhs: Self::Imm) -> Self::OpSsi;
    /// Creates the operator variant with signature: `fn(Stack, Stack) -> Stack`
    fn make_sss(result: Stack, lhs: Stack, rhs: Stack) -> Self::OpSss;
}

/// Class of non-commutative binary operators.
pub trait BinaryOperator {
    /// The name of the operator class.
    const NAME: &'static str;

    /// The type of immediate input operands.
    type Imm;

    /// The operator variant with signature: `fn(Imm, Reg) -> Reg`
    type OpRir: Operator;
    /// The operator variant with signature: `fn(Imm, Stack) -> Reg`
    type OpRis: Operator;
    /// The operator variant with signature: `fn(Reg, Imm) -> Reg`
    type OpRri: Operator;
    /// The operator variant with signature: `fn(Reg, Stack) -> Reg`
    type OpRrs: Operator;
    /// The operator variant with signature: `fn(Stack, Imm) -> Reg`
    type OpRsi: Operator;
    /// The operator variant with signature: `fn(Stack, Reg) -> Reg`
    type OpRsr: Operator;
    /// The operator variant with signature: `fn(Stack, Stack) -> Reg`
    type OpRss: Operator;

    /// Creates the operator variant with signature: `fn(Imm, Reg) -> Reg`
    fn make_rir(result: Reg, lhs: Self::Imm, rhs: Reg) -> Self::OpRir;
    /// Creates the operator variant with signature: `fn(Imm, Stack) -> Reg`
    fn make_ris(result: Reg, lhs: Self::Imm, rhs: Stack) -> Self::OpRis;
    /// Creates the operator variant with signature: `fn(Reg, Imm) -> Reg`
    fn make_rri(result: Reg, lhs: Reg, rhs: Self::Imm) -> Self::OpRri;
    /// Creates the operator variant with signature: `fn(Reg, Stack) -> Reg`
    fn make_rrs(result: Reg, lhs: Reg, rhs: Stack) -> Self::OpRrs;
    /// Creates the operator variant with signature: `fn(Stack, Imm) -> Reg`
    fn make_rsi(result: Reg, lhs: Stack, rhs: Self::Imm) -> Self::OpRsi;
    /// Creates the operator variant with signature: `fn(Stack, Reg) -> Reg`
    fn make_rsr(result: Reg, lhs: Stack, rhs: Reg) -> Self::OpRsr;
    /// Creates the operator variant with signature: `fn(Stack, Stack) -> Reg`
    fn make_rss(result: Reg, lhs: Stack, rhs: Stack) -> Self::OpRss;
}

/// Class of load operators.
pub trait LoadOperator {
    /// The name of the operator class.
    const NAME: &'static str;

    /// The operator variant for `(memory 0)` with signature: `fn(Imm) -> Reg`
    type OpMem0Ri: Operator;
    /// The operator variant for `(memory 0)` with signature: `fn(Reg) -> Reg`
    type OpMem0Rr: Operator;
    /// The operator variant for `(memory 0)` with signature: `fn(Stack) -> Reg`
    type OpMem0Rs: Operator;
    /// The operator variant for `(memory 0)` with signature: `fn(Imm) -> Stack`
    type OpMem0Si: Operator;
    /// The operator variant for `(memory 0)` with signature: `fn(Reg) -> Stack`
    type OpMem0Sr: Operator;
    /// The operator variant for `(memory 0)` with signature: `fn(Stack) -> Stack`
    type OpMem0Ss: Operator;
    /// The operator variant for with signature: `fn(Imm) -> Reg`
    type OpRi: Operator;
    /// The operator variant for with signature: `fn(Reg) -> Reg`
    type OpRr: Operator;
    /// The operator variant for with signature: `fn(Stack) -> Reg`
    type OpRs: Operator;

    /// Creates the operator variant for `(memory 0)` with signature: `fn(Imm) -> Reg`
    fn make_mem0_ri(result: Reg, address: Address) -> Self::OpMem0Ri;
    /// Creates the operator variant for `(memory 0)` with signature: `fn(Reg) -> Reg`
    fn make_mem0_rr(result: Reg, ptr: Reg, offset: Offset) -> Self::OpMem0Rr;
    /// Creates the operator variant for `(memory 0)` with signature: `fn(Stack) -> Reg`
    fn make_mem0_rs(result: Reg, ptr: Stack, offset: Offset) -> Self::OpMem0Rs;
    /// Creates the operator variant for `(memory 0)` with signature: `fn(Imm) -> Stack`
    fn make_mem0_si(result: Stack, address: Address) -> Self::OpMem0Si;
    /// Creates the operator variant for `(memory 0)` with signature: `fn(Reg) -> Stack`
    fn make_mem0_sr(result: Stack, ptr: Reg, offset: Offset) -> Self::OpMem0Sr;
    /// Creates the operator variant for `(memory 0)` with signature: `fn(Stack) -> Stack`
    fn make_mem0_ss(result: Stack, ptr: Stack, offset: Offset) -> Self::OpMem0Ss;
    /// Creates the operator variant for with signature: `fn(Imm) -> Reg`
    fn make_ri(result: Reg, address: Address, memory: Memory) -> Self::OpRi;
    /// Creates the operator variant for with signature: `fn(Reg) -> Reg`
    fn make_rr(result: Reg, ptr: Reg, offset: Offset, memory: Memory) -> Self::OpRr;
    /// Creates the operator variant for with signature: `fn(Stack) -> Reg`
    fn make_rs(result: Reg, ptr: Stack, offset: Offset, memory: Memory) -> Self::OpRs;
}

/// Class of store operators.
pub trait StoreOperator {
    /// The name of the operator class.
    const NAME: &'static str;

    /// The type of immediate value operand.
    type Imm;

    /// The operator variant for `(memory 0)` with signature: `fn(Reg, Reg)`
    /// 
    /// # Note
    /// 
    /// This only exists for some of the store classes.
    type OpMem0Rr: Operator;
    /// The operator variant for `(memory 0)` with signature: `fn(Reg, Stack)`
    type OpMem0Rs: Operator;
    /// The operator variant for `(memory 0)` with signature: `fn(Reg, Imm)`
    type OpMem0Ri: Operator;
    /// The operator variant for `(memory 0)` with signature: `fn(Stack, Reg)`
    type OpMem0Sr: Operator;
    /// The operator variant for `(memory 0)` with signature: `fn(Stack, Stack)`
    type OpMem0Ss: Operator;
    /// The operator variant for `(memory 0)` with signature: `fn(Stack, Imm)`
    type OpMem0Si: Operator;
    /// The operator variant for `(memory 0)` with signature: `fn(Imm, Reg)`
    type OpMem0Ir: Operator;
    /// The operator variant for `(memory 0)` with signature: `fn(Imm, Stack)`
    type OpMem0Is: Operator;
    /// The operator variant for `(memory 0)` with signature: `fn(Imm, Imm)`
    type OpMem0Ii: Operator;
    /// The operator variant with signature: `fn(Stack, Stack)`
    type OpSs: Operator;
    /// The operator variant with signature: `fn(Stack, Imm)`
    type OpSi: Operator;
    /// The operator variant with signature: `fn(Imm, Stack)`
    type OpIs: Operator;
    /// The operator variant with signature: `fn(Imm, Imm)`
    type OpIi: Operator;

    /// Creates the operator variant for `(memory 0)` with signature: `fn(Reg, Reg)`
    /// 
    /// # Note
    /// 
    /// This only exists for some of the store classes.
    fn make_mem0_rr(ptr: Reg, value: Reg) -> Option<Self::OpMem0Rr>;
    /// Creates the operator variant for `(memory 0)` with signature: `fn(Reg, Stack)`
    fn make_mem0_rs(ptr: Reg, value: Stack) -> Self::OpMem0Rs;
    /// Creates the operator variant for `(memory 0)` with signature: `fn(Reg, Imm)`
    fn make_mem0_ri(ptr: Reg, value: Self::Imm) -> Self::OpMem0Ri;
    /// Creates the operator variant for `(memory 0)` with signature: `fn(Stack, Reg)`
    fn make_mem0_sr(ptr: Stack, value: Reg) -> Self::OpMem0Sr;
    /// Creates the operator variant for `(memory 0)` with signature: `fn(Stack, Stack)`
    fn make_mem0_ss(ptr: Stack, value: Stack) -> Self::OpMem0Ss;
    /// Creates the operator variant for `(memory 0)` with signature: `fn(Stack, Imm)`
    fn make_mem0_si(ptr: Stack, value: Self::Imm) -> Self::OpMem0Si;
    /// Creates the operator variant for `(memory 0)` with signature: `fn(Imm, Reg)`
    fn make_mem0_ir(address: Address, value: Reg) -> Self::OpMem0Ir;
    /// Creates the operator variant for `(memory 0)` with signature: `fn(Imm, Stack)`
    fn make_mem0_is(address: Address, value: Stack) -> Self::OpMem0Is;
    /// Creates the operator variant for `(memory 0)` with signature: `fn(Imm, Imm)`
    fn make_mem0_ii(address: Address, value: Self::Imm) -> Self::OpMem0Ii;
    /// Creates the operator variant with signature: `fn(Stack, Stack)`
    fn make_ss() -> Self::OpSs;
    /// Creates the operator variant with signature: `fn(Stack, Imm)`
    fn make_si() -> Self::OpSi;
    /// Creates the operator variant with signature: `fn(Imm, Stack)`
    fn make_is() -> Self::OpIs;
    /// Creates the operator variant with signature: `fn(Imm, Imm)`
    fn make_ii() -> Self::OpIi;
}
