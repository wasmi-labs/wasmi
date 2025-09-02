use super::{Executor, InstructionPtr};
use crate::{
    core::wasm,
    engine::utils::unreachable_unchecked,
    ir::{FixedRegSpan, Op, Reg},
};

/// Parameters for the `i64.add128` and `i64.sub128` instructions.
struct Params128 {
    /// The register storing the high 64-bit part of the `lhs` parameter value.
    lhs_hi: Reg,
    /// The register storing the low 64-bit part of the `rhs` parameter value.
    rhs_lo: Reg,
    /// The register storing the low 64-bit part of the `rhs` parameter value.
    rhs_hi: Reg,
}

/// Function signature for `i64.binop128` handlers.
type BinOp128Fn = fn(lhs_lo: i64, lhs_hi: i64, rhs_lo: i64, rhs_hi: i64) -> (i64, i64);

/// Function signature for `i64.mul_wide_sx` handlers.
type I64MulWideFn = fn(lhs: i64, rhs: i64) -> (i64, i64);

impl Executor<'_> {
    /// Fetches the parameters required by the `i64.add128` and `i64.sub128` instructions.
    fn fetch_params128(&self) -> Params128 {
        let mut addr: InstructionPtr = self.ip;
        addr.add(1);
        match *addr.get() {
            Op::Register3 { regs } => Params128 {
                lhs_hi: regs[0],
                rhs_lo: regs[1],
                rhs_hi: regs[2],
            },
            unexpected => {
                // Safety: Wasmi translation guarantees that [`Op::MemoryIndex`] exists.
                unsafe {
                    unreachable_unchecked!("expected `Op::Register3` but found: {unexpected:?}")
                }
            }
        }
    }

    /// Executes a generic Wasm `i64.binop128` instruction.
    fn execute_i64_binop128(&mut self, results: [Reg; 2], lhs_lo: Reg, binop: BinOp128Fn) {
        let Params128 {
            lhs_hi,
            rhs_lo,
            rhs_hi,
        } = self.fetch_params128();
        let lhs_lo: i64 = self.get_register_as(lhs_lo);
        let lhs_hi: i64 = self.get_register_as(lhs_hi);
        let rhs_lo: i64 = self.get_register_as(rhs_lo);
        let rhs_hi: i64 = self.get_register_as(rhs_hi);
        let (result_lo, result_hi) = binop(lhs_lo, lhs_hi, rhs_lo, rhs_hi);
        self.set_register(results[0], result_lo);
        self.set_register(results[1], result_hi);
        self.next_instr_at(2)
    }

    /// Executes an [`Op::I64Add128`].
    pub fn execute_i64_add128(&mut self, results: [Reg; 2], lhs_lo: Reg) {
        self.execute_i64_binop128(results, lhs_lo, wasm::i64_add128)
    }

    /// Executes an [`Op::I64Sub128`].
    pub fn execute_i64_sub128(&mut self, results: [Reg; 2], lhs_lo: Reg) {
        self.execute_i64_binop128(results, lhs_lo, wasm::i64_sub128)
    }

    /// Executes a generic Wasm `i64.mul_wide_sx` instruction.
    fn execute_i64_mul_wide_sx(
        &mut self,
        results: FixedRegSpan<2>,
        lhs: Reg,
        rhs: Reg,
        mul_wide: I64MulWideFn,
    ) {
        let lhs: i64 = self.get_register_as(lhs);
        let rhs: i64 = self.get_register_as(rhs);
        let (result_lo, result_hi) = mul_wide(lhs, rhs);
        let results = results.to_array();
        self.set_register(results[0], result_lo);
        self.set_register(results[1], result_hi);
        self.next_instr()
    }

    /// Executes an [`Op::I64MulWideS`].
    pub fn execute_i64_mul_wide_s(&mut self, results: FixedRegSpan<2>, lhs: Reg, rhs: Reg) {
        self.execute_i64_mul_wide_sx(results, lhs, rhs, wasm::i64_mul_wide_s)
    }

    /// Executes an [`Op::I64MulWideU`].
    pub fn execute_i64_mul_wide_u(&mut self, results: FixedRegSpan<2>, lhs: Reg, rhs: Reg) {
        self.execute_i64_mul_wide_sx(results, lhs, rhs, wasm::i64_mul_wide_u)
    }
}
