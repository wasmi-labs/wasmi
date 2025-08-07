use super::*;
use crate::{engine::translator::tests::wasm_type::WasmTy, ValType};
use core::{
    fmt::{self, Display},
    mem,
};

/// Tells which kind of `select` instruction to test.
#[derive(Debug, Copy, Clone)]
enum SelectKind {
    /// The untyped Wasm `select` instruction.
    Select,
    /// The typed Wasm `select (result <ty>)` instruction
    /// introduced in the `reference-types` Wasm proposal.
    TypedSelect,
}

/// Display a `select` or typed `select (result <ty>)` instruction as demanded by Wasm.
struct DisplaySelect {
    /// The kind of the `select` instruction.
    kind: SelectKind,
    /// The `result` type of the `select` instruction.
    ty: ValType,
}

impl DisplaySelect {
    /// Creates a new [`DisplaySelect`].
    fn new(kind: SelectKind, ty: ValType) -> Self {
        Self { kind, ty }
    }
}

impl Display for DisplaySelect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            SelectKind::Select => write!(f, "select"),
            SelectKind::TypedSelect => {
                write!(f, "select (result {})", DisplayValueType::from(self.ty))
            }
        }
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg() {
    fn test_reg(kind: SelectKind, result_ty: ValType) {
        let display_ty = DisplayValueType::from(result_ty);
        let display_select = DisplaySelect::new(kind, result_ty);
        let wasm = format!(
            r#"
            (module
                (func (param $condition i32)
                      (param $true_val {display_ty})
                      (param $false_val {display_ty})
                      (result {display_ty})
                    local.get $true_val
                    local.get $false_val
                    local.get $condition
                    {display_select}
                )
            )
        "#,
        );
        let condition = Reg::from(0);
        let true_val = Reg::from(1);
        let false_val = Reg::from(2);
        let result = Reg::from(3);
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                Instruction::select_i32_eq_imm16(result, condition, 0_i16),
                Instruction::register2_ext(false_val, true_val),
                Instruction::return_reg(result),
            ])
            .run();
    }
    fn test_for(kind: SelectKind) {
        test_reg(kind, ValType::I32);
        test_reg(kind, ValType::I64);
        test_reg(kind, ValType::F32);
        test_reg(kind, ValType::F64);
    }
    test_for(SelectKind::Select);
    test_for(SelectKind::TypedSelect);
    test_reg(SelectKind::TypedSelect, ValType::FuncRef);
    test_reg(SelectKind::TypedSelect, ValType::ExternRef);
}

#[test]
#[cfg_attr(miri, ignore)]
fn same_reg() {
    fn test_same_reg(kind: SelectKind, result_ty: ValType) {
        let display_ty = DisplayValueType::from(result_ty);
        let display_select = DisplaySelect::new(kind, result_ty);
        let wasm = format!(
            r#"
            (module
                (func (param $condition i32) (param $input {display_ty}) (result {display_ty})
                    local.get $input
                    local.get $input
                    local.get $condition
                    {display_select}
                )
            )
        "#,
        );
        TranslationTest::new(&wasm)
            .expect_func_instrs([Instruction::return_reg(Reg::from(1))])
            .run();
    }
    fn test_for(kind: SelectKind) {
        test_same_reg(kind, ValType::I32);
        test_same_reg(kind, ValType::I64);
        test_same_reg(kind, ValType::F32);
        test_same_reg(kind, ValType::F64);
    }
    test_for(SelectKind::Select);
    test_for(SelectKind::TypedSelect);
    test_same_reg(SelectKind::TypedSelect, ValType::FuncRef);
    test_same_reg(SelectKind::TypedSelect, ValType::ExternRef);
}

fn test_same_imm<T>(kind: SelectKind, input: T) -> TranslationTest
where
    T: WasmTy,
    DisplayWasm<T>: Display,
{
    let ty = T::VALUE_TYPE;
    let display_ty = DisplayValueType::from(ty);
    let display_input = DisplayWasm::from(input);
    let display_select = DisplaySelect::new(kind, ty);
    let wasm = format!(
        r#"
        (module
            (func (param $condition i32) (param $input {display_ty}) (result {display_ty})
                {display_ty}.const {display_input}
                {display_ty}.const {display_input}
                local.get $condition
                {display_select}
            )
        )
    "#,
    );
    TranslationTest::new(&wasm)
}

#[test]
#[cfg_attr(miri, ignore)]
fn same_imm32() {
    fn test_for_kind<T>(kind: SelectKind, value: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
        AnyConst32: From<T>,
    {
        let expected = [Instruction::return_imm32(AnyConst32::from(value))];
        test_same_imm(kind, value)
            .expect_func_instrs(expected)
            .run();
    }

    fn test_for<T>(value: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
        AnyConst32: From<T>,
    {
        test_for_kind(SelectKind::Select, value);
        test_for_kind(SelectKind::TypedSelect, value);
    }

    test_for::<i32>(0);
    test_for::<i32>(1);
    test_for::<i32>(-1);
    test_for::<i32>(i32::MIN);
    test_for::<i32>(i32::MAX);

    test_for::<f32>(0.0);
    test_for::<f32>(0.25);
    test_for::<f32>(-0.25);
    test_for::<f32>(1.0);
    test_for::<f32>(-1.0);
    test_for::<f32>(f32::NEG_INFINITY);
    test_for::<f32>(f32::INFINITY);
    test_for::<f32>(f32::NAN);
    test_for::<f32>(f32::EPSILON);
}

#[test]
#[cfg_attr(miri, ignore)]
fn same_i64imm32() {
    fn test_for(value: i64) {
        let expected = [return_i64imm32_instr(value)];
        test_same_imm(SelectKind::Select, value)
            .expect_func_instrs(expected)
            .run();
        test_same_imm(SelectKind::TypedSelect, value)
            .expect_func_instrs(expected)
            .run();
    }

    test_for(0);
    test_for(1);
    test_for(-1);
    test_for(i64::from(i32::MIN) + 1);
    test_for(i64::from(i32::MIN));
    test_for(i64::from(i32::MAX) - 1);
    test_for(i64::from(i32::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn same_f64imm32() {
    fn test_for(value: f64) {
        let expected = [return_f64imm32_instr(value)];
        test_same_imm(SelectKind::Select, value)
            .expect_func_instrs(expected)
            .run();
        test_same_imm(SelectKind::TypedSelect, value)
            .expect_func_instrs(expected)
            .run();
    }

    test_for(0.0);
    test_for(0.25);
    test_for(-0.25);
    test_for(1.0);
    test_for(-1.0);
    test_for(f64::NEG_INFINITY);
    test_for(f64::INFINITY);
    test_for(f64::NAN);
    test_for(f64::EPSILON);
}

#[test]
#[cfg_attr(miri, ignore)]
fn same_imm() {
    fn test_for<T>(value: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
    {
        let instrs = [Instruction::return_reg(Reg::from(-1))];
        let expected = ExpectedFunc::new(instrs).consts([value]);
        test_same_imm(SelectKind::Select, value)
            .expect_func(expected.clone())
            .run();
        test_same_imm(SelectKind::TypedSelect, value)
            .expect_func(expected)
            .run();
    }

    test_for::<i64>(i64::from(i32::MIN) - 1);
    test_for::<i64>(i64::from(i32::MAX) + 1);
    test_for::<i64>(i64::MIN + 1);
    test_for::<i64>(i64::MIN);
    test_for::<i64>(i64::MAX - 1);
    test_for::<i64>(i64::MAX);

    test_for::<f64>(0.3);
    test_for::<f64>(-0.3);
    test_for::<f64>(0.123456789);
    test_for::<f64>(-0.123456789);
    test_for::<f64>(9.87654321);
    test_for::<f64>(-9.87654321);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cmp_select() {
    fn run_test(
        op: CmpOp,
        kind: SelectKind,
        result_ty: ValType,
        expected: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        swap_operands: bool,
    ) {
        let cmp_input_ty = op.param_ty();
        let cmp_result_ty = op.result_ty();
        let cmp_input_ty = DisplayValueType::from(cmp_input_ty);
        let cmp_result_ty = DisplayValueType::from(cmp_result_ty);
        let cmp_op = op.op_str();

        let select_op = DisplaySelect::new(kind, result_ty);
        let result_ty = DisplayValueType::from(result_ty);
        let wasm = format!(
            r#"
            (module
                (func
                    (param $lhs {cmp_input_ty}) (param $rhs {cmp_input_ty})
                    (param $true_val {result_ty}) (param $false_val {result_ty})
                    (result {result_ty})
                    ({select_op}
                        (local.get $true_val)
                        (local.get $false_val)
                        ({cmp_result_ty}.ne
                            ({cmp_input_ty}.{cmp_op} (local.get $lhs) (local.get $rhs))
                            ({cmp_result_ty}.const 0)
                        )
                    )
                )
            )
        "#,
        );
        let mut true_val = Reg::from(2);
        let mut false_val = Reg::from(3);
        if swap_operands {
            mem::swap(&mut true_val, &mut false_val);
        }
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                expected(Reg::from(4), Reg::from(0), Reg::from(1)),
                Instruction::register2_ext(true_val, false_val),
                Instruction::return_reg(4),
            ])
            .run();
    }

    #[rustfmt::skip]
    fn test_for_each_cmp(kind: SelectKind, ty: ValType) {
        for (op, expected, swap_operands) in [
            (CmpOp::I32And, Instruction::select_i32_and as fn(Reg, Reg, Reg) -> Instruction, false),
            (CmpOp::I32Or, Instruction::select_i32_or, false),
            (CmpOp::I32Xor, Instruction::select_i32_xor, false),
            (CmpOp::I32Eq, Instruction::select_i32_eq, false),
            (CmpOp::I32Ne, Instruction::select_i32_eq, true),
            (CmpOp::I32LtS, Instruction::select_i32_lt_s, false),
            (CmpOp::I32LtU, Instruction::select_i32_lt_u, false),
            (CmpOp::I32LeS, Instruction::select_i32_le_s, false),
            (CmpOp::I32LeU, Instruction::select_i32_le_u, false),
            (CmpOp::I32GtS, swap_cmp_select_ops!(Instruction::select_i32_lt_s), false),
            (CmpOp::I32GtU, swap_cmp_select_ops!(Instruction::select_i32_lt_u), false),
            (CmpOp::I32GeS, swap_cmp_select_ops!(Instruction::select_i32_le_s), false),
            (CmpOp::I32GeU, swap_cmp_select_ops!(Instruction::select_i32_le_u), false),
            (CmpOp::I64And, Instruction::select_i64_and, false),
            (CmpOp::I64Or, Instruction::select_i64_or, false),
            (CmpOp::I64Xor, Instruction::select_i64_xor, false),
            (CmpOp::I64Eq, Instruction::select_i64_eq, false),
            (CmpOp::I64Ne, Instruction::select_i64_eq, true),
            (CmpOp::I64LtS, Instruction::select_i64_lt_s, false),
            (CmpOp::I64LtU, Instruction::select_i64_lt_u, false),
            (CmpOp::I64LeS, Instruction::select_i64_le_s, false),
            (CmpOp::I64LeU, Instruction::select_i64_le_u, false),
            (CmpOp::I64GtS, swap_cmp_select_ops!(Instruction::select_i64_lt_s), false),
            (CmpOp::I64GtU, swap_cmp_select_ops!(Instruction::select_i64_lt_u), false),
            (CmpOp::I64GeS, swap_cmp_select_ops!(Instruction::select_i64_le_s), false),
            (CmpOp::I64GeU, swap_cmp_select_ops!(Instruction::select_i64_le_u), false),
            (CmpOp::F32Eq, Instruction::select_f32_eq, false),
            (CmpOp::F32Ne, Instruction::select_f32_eq, true),
            (CmpOp::F32Lt, Instruction::select_f32_lt, false),
            (CmpOp::F32Le, Instruction::select_f32_le, false),
            (CmpOp::F32Gt, swap_cmp_select_ops!(Instruction::select_f32_lt), false),
            (CmpOp::F32Ge, swap_cmp_select_ops!(Instruction::select_f32_le), false),
            (CmpOp::F64Eq, Instruction::select_f64_eq, false),
            (CmpOp::F64Ne, Instruction::select_f64_eq, true),
            (CmpOp::F64Lt, Instruction::select_f64_lt, false),
            (CmpOp::F64Le, Instruction::select_f64_le, false),
            (CmpOp::F64Gt, swap_cmp_select_ops!(Instruction::select_f64_lt), false),
            (CmpOp::F64Ge, swap_cmp_select_ops!(Instruction::select_f64_le), false),
        ] {
            run_test(op, kind, ty, expected, swap_operands)
        }
    }

    fn test_for(kind: SelectKind) {
        for ty in [ValType::I32, ValType::I64, ValType::F32, ValType::F64] {
            test_for_each_cmp(kind, ty);
        }
    }

    test_for(SelectKind::Select);
    test_for(SelectKind::TypedSelect);
    test_for_each_cmp(SelectKind::TypedSelect, ValType::FuncRef);
    test_for_each_cmp(SelectKind::TypedSelect, ValType::ExternRef);
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_cmp_select_eqz() {
    fn run_test(
        op: CmpOp,
        kind: SelectKind,
        result_ty: ValType,
        expected: fn(result: Reg, lhs: Reg, rhs: Reg) -> Instruction,
        swap_operands: bool,
    ) {
        let cmp_input_ty = op.param_ty();
        let cmp_result_ty = op.result_ty();
        let cmp_input_ty = DisplayValueType::from(cmp_input_ty);
        let cmp_result_ty = DisplayValueType::from(cmp_result_ty);
        let cmp_op = op.op_str();

        let select_op = DisplaySelect::new(kind, result_ty);
        let result_ty = DisplayValueType::from(result_ty);
        let wasm = format!(
            r#"
            (module
                (func
                    (param $lhs {cmp_input_ty}) (param $rhs {cmp_input_ty})
                    (param $true_val {result_ty}) (param $false_val {result_ty})
                    (result {result_ty})
                    ({select_op}
                        (local.get $true_val)
                        (local.get $false_val)
                        ({cmp_result_ty}.eqz
                            ({cmp_input_ty}.{cmp_op} (local.get $lhs) (local.get $rhs))
                        )
                    )
                )
            )
        "#,
        );
        let mut true_val = Reg::from(2);
        let mut false_val = Reg::from(3);
        if swap_operands {
            mem::swap(&mut true_val, &mut false_val);
        }
        TranslationTest::new(&wasm)
            .expect_func_instrs([
                expected(Reg::from(4), Reg::from(0), Reg::from(1)),
                Instruction::register2_ext(true_val, false_val),
                Instruction::return_reg(4),
            ])
            .run();
    }

    #[rustfmt::skip]
    fn test_for_each_cmp(kind: SelectKind, ty: ValType) {
        for (op, expected, swap_operands) in [
            (CmpOp::I32And, Instruction::select_i32_and as fn(Reg, Reg, Reg) -> Instruction, true),
            (CmpOp::I32Or, Instruction::select_i32_or, true),
            (CmpOp::I32Xor, Instruction::select_i32_xor, true),
            (CmpOp::I32Eq, Instruction::select_i32_eq, true),
            (CmpOp::I32Ne, Instruction::select_i32_eq, false),
            (CmpOp::I32LtS, swap_cmp_select_ops!(Instruction::select_i32_le_s), false),
            (CmpOp::I32LtU, swap_cmp_select_ops!(Instruction::select_i32_le_u), false),
            (CmpOp::I32LeS, swap_cmp_select_ops!(Instruction::select_i32_lt_s), false),
            (CmpOp::I32LeU, swap_cmp_select_ops!(Instruction::select_i32_lt_u), false),
            (CmpOp::I32GtS, Instruction::select_i32_le_s, false),
            (CmpOp::I32GtU, Instruction::select_i32_le_u, false),
            (CmpOp::I32GeS, Instruction::select_i32_lt_s, false),
            (CmpOp::I32GeU, Instruction::select_i32_lt_u, false),
            (CmpOp::I64And, Instruction::select_i64_and, true),
            (CmpOp::I64Or, Instruction::select_i64_or, true),
            (CmpOp::I64Xor, Instruction::select_i64_xor, true),
            (CmpOp::I64Eq, Instruction::select_i64_eq, true),
            (CmpOp::I64Ne, Instruction::select_i64_eq, false),
            (CmpOp::I64LtS, swap_cmp_select_ops!(Instruction::select_i64_le_s), false),
            (CmpOp::I64LtU, swap_cmp_select_ops!(Instruction::select_i64_le_u), false),
            (CmpOp::I64LeS, swap_cmp_select_ops!(Instruction::select_i64_lt_s), false),
            (CmpOp::I64LeU, swap_cmp_select_ops!(Instruction::select_i64_lt_u), false),
            (CmpOp::I64GtS, Instruction::select_i64_le_s, false),
            (CmpOp::I64GtU, Instruction::select_i64_le_u, false),
            (CmpOp::I64GeS, Instruction::select_i64_lt_s, false),
            (CmpOp::I64GeU, Instruction::select_i64_lt_u, false),
            (CmpOp::F32Eq, Instruction::select_f32_eq, true),
            (CmpOp::F32Ne, Instruction::select_f32_eq, false),
            (CmpOp::F32Lt, swap_cmp_select_ops!(Instruction::select_f32_lt), true),
            (CmpOp::F32Le, swap_cmp_select_ops!(Instruction::select_f32_le), true),
            (CmpOp::F32Gt, Instruction::select_f32_lt, true),
            (CmpOp::F32Ge, Instruction::select_f32_le, true),
            (CmpOp::F64Eq, Instruction::select_f64_eq, true),
            (CmpOp::F64Ne, Instruction::select_f64_eq, false),
            (CmpOp::F64Lt, swap_cmp_select_ops!(Instruction::select_f64_lt), true),
            (CmpOp::F64Le, swap_cmp_select_ops!(Instruction::select_f64_le), true),
            (CmpOp::F64Gt, Instruction::select_f64_lt, true),
            (CmpOp::F64Ge, Instruction::select_f64_le, true),
        ] {
            run_test(op, kind, ty, expected, swap_operands)
        }
    }

    fn test_for(kind: SelectKind) {
        for ty in [ValType::I32, ValType::I64, ValType::F32, ValType::F64] {
            test_for_each_cmp(kind, ty);
        }
    }

    test_for(SelectKind::Select);
    test_for(SelectKind::TypedSelect);
    test_for_each_cmp(SelectKind::TypedSelect, ValType::FuncRef);
    test_for_each_cmp(SelectKind::TypedSelect, ValType::ExternRef);
}
