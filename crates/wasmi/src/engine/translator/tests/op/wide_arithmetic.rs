use super::*;
use crate::core::UntypedVal;

/// A Wasm `wide-arithmetic` wide-mulitplication instruction.
#[derive(Copy, Clone)]
enum MulWideOp {
    /// Signed
    MulWideS,
    /// Unsigned
    MulWideU,
}

impl MulWideOp {
    /// Returns the `.wat` formatted instruction name.
    pub fn wat(self) -> &'static str {
        match self {
            MulWideOp::MulWideS => "i64.mul_wide_s",
            MulWideOp::MulWideU => "i64.mul_wide_u",
        }
    }

    /// Evaluates the inputs for the selected instruction.
    pub fn eval(self, lhs: i64, rhs: i64) -> (i64, i64) {
        let (res0, res1) = match self {
            MulWideOp::MulWideS => UntypedVal::i64_mul_wide_s(lhs.into(), rhs.into()),
            MulWideOp::MulWideU => UntypedVal::i64_mul_wide_u(lhs.into(), rhs.into()),
        };
        (res0.into(), res1.into())
    }
}

fn const_eval_for(op: MulWideOp, lhs: i64, rhs: i64) {
    let wat = op.wat();
    let wasm = format!(
        r"
        (module
            (func (result i64 i64)
                i64.const {lhs}
                i64.const {rhs}
                {wat}
            )
        )
    "
    );
    let (result_lo, result_hi) = op.eval(lhs, rhs);
    TranslationTest::new(wasm)
        .expect_func(
            ExpectedFunc::new([Instruction::return_reg2_ext(-1, -2)])
                .consts([result_lo, result_hi]),
        )
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn const_eval() {
    for op in [MulWideOp::MulWideS, MulWideOp::MulWideU] {
        const_eval_for(op, 288230376151711744, 288230376151711744);
    }
}
