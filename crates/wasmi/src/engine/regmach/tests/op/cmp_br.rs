use super::*;
use crate::engine::regmach::tests::display_wasm::DisplayValueType;
use crate::engine::{
    bytecode::{BranchOffset, GlobalIdx},
    regmach::bytecode::BranchOffset16,
};
use crate::core::ValueType;

#[test]
#[cfg_attr(miri, ignore)]
fn loop_backward() {
    fn test_for(ty: ValueType, op: &str, expect_instr: fn(Register, Register, BranchOffset16) -> Instruction) {
        let ty = DisplayValueType::from(ty);
        let wasm = wat2wasm(&format!(
            r"
            (module
                (func (param {ty} {ty})
                    (loop
                        (local.get 0)
                        (local.get 1)
                        ({ty}.{op})
                        (br_if 0)
                    )
                )
            )",
        ));
        TranslationTest::new(wasm)
            .expect_func_instrs([
                expect_instr(
                    Register::from_i16(0),
                    Register::from_i16(1),
                    BranchOffset16::from(0),
                ),
                Instruction::Return,
            ])
            .run()
    }

    test_for(ValueType::I32, "eq", Instruction::branch_i32_eq);
    test_for(ValueType::I32, "ne", Instruction::branch_i32_ne);
    test_for(ValueType::I32, "lt_s", Instruction::branch_i32_lt_s);
    test_for(ValueType::I32, "lt_u", Instruction::branch_i32_lt_u);
    test_for(ValueType::I32, "le_s", Instruction::branch_i32_le_s);
    test_for(ValueType::I32, "le_u", Instruction::branch_i32_le_u);
    test_for(ValueType::I32, "gt_s", Instruction::branch_i32_gt_s);
    test_for(ValueType::I32, "gt_u", Instruction::branch_i32_gt_u);
    test_for(ValueType::I32, "ge_s", Instruction::branch_i32_ge_s);
    test_for(ValueType::I32, "ge_u", Instruction::branch_i32_ge_u);

    test_for(ValueType::I64, "eq", Instruction::branch_i64_eq);
    test_for(ValueType::I64, "ne", Instruction::branch_i64_ne);
    test_for(ValueType::I64, "lt_s", Instruction::branch_i64_lt_s);
    test_for(ValueType::I64, "lt_u", Instruction::branch_i64_lt_u);
    test_for(ValueType::I64, "le_s", Instruction::branch_i64_le_s);
    test_for(ValueType::I64, "le_u", Instruction::branch_i64_le_u);
    test_for(ValueType::I64, "gt_s", Instruction::branch_i64_gt_s);
    test_for(ValueType::I64, "gt_u", Instruction::branch_i64_gt_u);
    test_for(ValueType::I64, "ge_s", Instruction::branch_i64_ge_s);
    test_for(ValueType::I64, "ge_u", Instruction::branch_i64_ge_u);

    test_for(ValueType::F32, "eq", Instruction::branch_f32_eq);
    test_for(ValueType::F32, "ne", Instruction::branch_f32_ne);
    test_for(ValueType::F32, "lt", Instruction::branch_f32_lt);
    test_for(ValueType::F32, "le", Instruction::branch_f32_le);
    test_for(ValueType::F32, "gt", Instruction::branch_f32_gt);
    test_for(ValueType::F32, "ge", Instruction::branch_f32_ge);

    test_for(ValueType::F64, "eq", Instruction::branch_f64_eq);
    test_for(ValueType::F64, "ne", Instruction::branch_f64_ne);
    test_for(ValueType::F64, "lt", Instruction::branch_f64_lt);
    test_for(ValueType::F64, "le", Instruction::branch_f64_le);
    test_for(ValueType::F64, "gt", Instruction::branch_f64_gt);
    test_for(ValueType::F64, "ge", Instruction::branch_f64_ge);
}

#[test]
#[cfg_attr(miri, ignore)]
fn loop_backward_imm() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32)
                (loop
                    (local.get 0)
                    (i32.const 1)
                    (i32.eq)
                    (br_if 0)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq_imm(
                Register::from_i16(0),
                i32imm16(1_i32),
                BranchOffset16::from(0),
            ),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn loop_backward_imm_eqz() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32)
                (loop
                    (local.get 0)
                    (i32.const 0)
                    (i32.eq)
                    (br_if 0)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_eqz(Register::from_i16(0), BranchOffset::from(0_i32)),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn block_forward() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32)
                (block
                    (local.get 0)
                    (local.get 1)
                    (i32.eq)
                    (br_if 0)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_eq(
                Register::from_i16(0),
                Register::from_i16(1),
                BranchOffset16::from(1),
            ),
            Instruction::Return,
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn block_forward_nop_copy() {
    let wasm = wat2wasm(
        r"
        (module
            (global $g (mut i32) (i32.const 10))
            (func (param i32 i32) (result i32)
                (global.get $g)
                (block (param i32) (result i32)
                    (local.get 0)
                    (local.get 1)
                    (i32.eq)
                    (br_if 0)
                    (drop)
                    (local.get 0)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::global_get(Register::from_i16(2), GlobalIdx::from(0)),
            Instruction::branch_i32_eq(
                Register::from_i16(0),
                Register::from_i16(1),
                BranchOffset16::from(2),
            ),
            Instruction::copy(Register::from_i16(2), Register::from_i16(0)),
            Instruction::return_reg(2),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_forward_multi_value() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32) (result i32)
                (block (result i32)
                    (local.get 0) ;; returned from block if `local.get 0 != 0`
                    (local.get 0)
                    (local.get 1)
                    (i32.eq)
                    (br_if 0)
                    (drop)
                    (local.get 1) ;; returned from block if `local.get 0 == 0`
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_ne(
                Register::from_i16(0),
                Register::from_i16(1),
                BranchOffset16::from(3),
            ),
            Instruction::copy(Register::from_i16(2), Register::from_i16(0)),
            Instruction::branch(BranchOffset::from(2)),
            Instruction::copy(Register::from_i16(2), Register::from_i16(1)),
            Instruction::return_reg(2),
        ])
        .run()
}

#[test]
#[cfg_attr(miri, ignore)]
fn if_forward() {
    let wasm = wat2wasm(
        r"
        (module
            (func (param i32 i32)
                (if
                    (i32.eq
                        (local.get 0)
                        (local.get 1)
                    )
                    (then)
                )
            )
        )",
    );
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::branch_i32_ne(
                Register::from_i16(0),
                Register::from_i16(1),
                BranchOffset16::from(1),
            ),
            Instruction::Return,
        ])
        .run()
}
