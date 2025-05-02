use super::*;
use crate::{core::ValType, engine::translator::tests::wasm_type::WasmTy};
use core::{fmt, fmt::Display};

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
                Instruction::select_i32_ne_imm16(result, condition, 0_i16),
                Instruction::register2_ext(true_val, false_val),
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

fn test_reg_imm<T>(kind: SelectKind, rhs: T) -> TranslationTest
where
    T: WasmTy,
    DisplayWasm<T>: Display,
{
    let ty = T::VALUE_TYPE;
    let display_ty = DisplayValueType::from(ty);
    let display_rhs = DisplayWasm::from(rhs);
    let display_select = DisplaySelect::new(kind, ty);
    let wasm = format!(
        r#"
        (module
            (func (param $condition i32) (param $lhs {display_ty}) (result {display_ty})
                local.get $lhs
                {display_ty}.const {display_rhs}
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
fn reg_imm32() {
    fn test_for_kind<T>(kind: SelectKind, value: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
        AnyConst32: From<T>,
    {
        let result = Reg::from(2);
        let condition = Reg::from(0);
        let lhs = Reg::from(1);
        let expected = [
            Instruction::select_imm32_rhs(result, lhs),
            Instruction::register_and_imm32(condition, value),
            Instruction::return_reg(result),
        ];
        test_reg_imm(kind, value).expect_func_instrs(expected).run();
    }

    fn test_for<T>(value: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
        AnyConst32: From<T>,
    {
        test_for_kind::<T>(SelectKind::Select, value);
        test_for_kind::<T>(SelectKind::TypedSelect, value);
    }

    test_for::<i32>(0);
    test_for::<i32>(1);
    test_for::<i32>(-1);
    test_for::<i32>(i32::MIN + 1);
    test_for::<i32>(i32::MIN);
    test_for::<i32>(i32::MAX - 1);
    test_for::<i32>(i32::MAX);

    test_for::<f32>(0.0);
    test_for::<f32>(0.25);
    test_for::<f32>(-0.25);
    test_for::<f32>(0.3);
    test_for::<f32>(-0.3);
    test_for::<f32>(1.0);
    test_for::<f32>(-1.0);
    test_for::<f32>(f32::NEG_INFINITY);
    test_for::<f32>(f32::INFINITY);
    test_for::<f32>(f32::NAN);
    test_for::<f32>(f32::EPSILON);
}

#[test]
#[cfg_attr(miri, ignore)]
fn reg_imm() {
    fn test_for_kind<T>(kind: SelectKind, value: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
    {
        let result = Reg::from(2);
        let condition = Reg::from(0);
        let lhs = Reg::from(1);
        let instrs = [
            Instruction::select(result, lhs),
            Instruction::register2_ext(condition, Reg::from(-1)),
            Instruction::return_reg(result),
        ];
        let expected = ExpectedFunc::new(instrs).consts([value]);
        test_reg_imm(kind, value).expect_func(expected).run();
    }

    fn test_for<T>(value: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
    {
        test_for_kind(SelectKind::Select, value);
        test_for_kind(SelectKind::TypedSelect, value);
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
fn reg_i64imm32() {
    fn test_for_kind(kind: SelectKind, value: i64) {
        let result = Reg::from(2);
        let condition = Reg::from(0);
        let lhs = Reg::from(1);
        let expected = [
            Instruction::select_i64imm32_rhs(result, lhs),
            Instruction::register_and_imm32(condition, i32::try_from(value).unwrap()),
            Instruction::return_reg(result),
        ];
        test_reg_imm(kind, value).expect_func_instrs(expected).run();
    }

    fn test_for(value: i64) {
        test_for_kind(SelectKind::Select, value);
        test_for_kind(SelectKind::TypedSelect, value);
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
fn reg_f64imm32() {
    fn test_for_kind(kind: SelectKind, value: f64) {
        let result = Reg::from(2);
        let condition = Reg::from(0);
        let lhs = Reg::from(1);
        let expected = [
            Instruction::select_f64imm32_rhs(result, lhs),
            Instruction::register_and_imm32(condition, value as f32),
            Instruction::return_reg(result),
        ];
        test_reg_imm(kind, value).expect_func_instrs(expected).run();
    }

    fn test_for(value: f64) {
        test_for_kind(SelectKind::Select, value);
        test_for_kind(SelectKind::TypedSelect, value);
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

fn test_imm_reg<T>(kind: SelectKind, lhs: T) -> TranslationTest
where
    T: WasmTy,
    DisplayWasm<T>: Display,
{
    let ty = T::VALUE_TYPE;
    let display_ty = DisplayValueType::from(ty);
    let display_lhs = DisplayWasm::from(lhs);
    let display_select = DisplaySelect::new(kind, ty);
    let wasm = format!(
        r#"
        (module
            (func (param $condition i32) (param $rhs {display_ty}) (result {display_ty})
                {display_ty}.const {display_lhs}
                local.get $rhs
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
fn imm32_reg() {
    fn test_for_kind<T>(kind: SelectKind, value: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
        AnyConst32: From<T>,
    {
        let result = Reg::from(2);
        let condition = Reg::from(0);
        let lhs = Reg::from(1);
        let expected = [
            Instruction::select_imm32_lhs(result, value),
            Instruction::register2_ext(condition, lhs),
            Instruction::return_reg(result),
        ];
        test_imm_reg(kind, value).expect_func_instrs(expected).run();
    }

    fn test_for<T>(value: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
        AnyConst32: From<T>,
    {
        test_for_kind::<T>(SelectKind::Select, value);
        test_for_kind::<T>(SelectKind::TypedSelect, value);
    }

    test_for::<i32>(0);
    test_for::<i32>(1);
    test_for::<i32>(-1);
    test_for::<i32>(i32::MIN + 1);
    test_for::<i32>(i32::MIN);
    test_for::<i32>(i32::MAX - 1);
    test_for::<i32>(i32::MAX);

    test_for::<f32>(0.0);
    test_for::<f32>(0.25);
    test_for::<f32>(-0.25);
    test_for::<f32>(0.3);
    test_for::<f32>(-0.3);
    test_for::<f32>(1.0);
    test_for::<f32>(-1.0);
    test_for::<f32>(f32::NEG_INFINITY);
    test_for::<f32>(f32::INFINITY);
    test_for::<f32>(f32::NAN);
    test_for::<f32>(f32::EPSILON);
}

#[test]
#[cfg_attr(miri, ignore)]
fn imm_reg() {
    fn test_for_kind<T>(kind: SelectKind, value: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
    {
        let result = Reg::from(2);
        let condition = Reg::from(0);
        let lhs = Reg::from(1);
        let instrs = [
            Instruction::select(result, Reg::from(-1)),
            Instruction::register2_ext(condition, lhs),
            Instruction::return_reg(result),
        ];
        let expected = ExpectedFunc::new(instrs).consts([value]);
        test_imm_reg(kind, value).expect_func(expected).run();
    }

    fn test_for<T>(value: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
    {
        test_for_kind(SelectKind::Select, value);
        test_for_kind(SelectKind::TypedSelect, value);
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
fn i64imm32_reg() {
    fn test_for_kind(kind: SelectKind, value: i64) {
        let result = Reg::from(2);
        let condition = Reg::from(0);
        let lhs = Reg::from(1);
        let expected = [
            Instruction::select_i64imm32_lhs(result, i32::try_from(value).unwrap()),
            Instruction::register2_ext(condition, lhs),
            Instruction::return_reg(result),
        ];
        test_imm_reg(kind, value).expect_func_instrs(expected).run();
    }

    fn test_for(value: i64) {
        test_for_kind(SelectKind::Select, value);
        test_for_kind(SelectKind::TypedSelect, value);
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
fn f64imm32_reg() {
    fn test_for_kind(kind: SelectKind, value: f64) {
        let result = Reg::from(2);
        let condition = Reg::from(0);
        let lhs = Reg::from(1);
        let expected = [
            Instruction::select_f64imm32_lhs(result, value as f32),
            Instruction::register2_ext(condition, lhs),
            Instruction::return_reg(result),
        ];
        test_imm_reg(kind, value).expect_func_instrs(expected).run();
    }

    fn test_for(value: f64) {
        test_for_kind(SelectKind::Select, value);
        test_for_kind(SelectKind::TypedSelect, value);
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

fn test_both_imm<T>(kind: SelectKind, lhs: T, rhs: T) -> TranslationTest
where
    T: WasmTy,
    DisplayWasm<T>: Display,
{
    let ty = T::VALUE_TYPE;
    let display_ty = DisplayValueType::from(ty);
    let display_lhs = DisplayWasm::from(lhs);
    let display_rhs = DisplayWasm::from(rhs);
    let display_select = DisplaySelect::new(kind, ty);
    let wasm = format!(
        r#"
        (module
            (func (param $condition i32) (result {display_ty})
                {display_ty}.const {display_lhs}
                {display_ty}.const {display_rhs}
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
fn both_imm32() {
    fn test_for_kind<T>(kind: SelectKind, lhs: T, rhs: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
        AnyConst32: From<T>,
    {
        let result = Reg::from(1);
        let condition = Reg::from(0);
        let lhs32 = AnyConst32::from(lhs);
        let rhs32 = AnyConst32::from(rhs);
        let expected = [
            Instruction::select_imm32(result, lhs32),
            Instruction::register_and_imm32(condition, rhs32),
            Instruction::return_reg(result),
        ];
        test_both_imm(kind, lhs, rhs)
            .expect_func_instrs(expected)
            .run();
    }

    fn test_for<T>(lhs: T, rhs: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
        AnyConst32: From<T>,
    {
        test_for_kind(SelectKind::Select, lhs, rhs);
        test_for_kind(SelectKind::Select, rhs, lhs);
        test_for_kind(SelectKind::TypedSelect, lhs, rhs);
        test_for_kind(SelectKind::TypedSelect, rhs, lhs);
    }

    test_for::<i32>(0, 1);
    test_for::<i32>(-5, 42);
    test_for::<i32>(i32::MIN + 1, i32::MAX - 1);
    test_for::<i32>(i32::MIN, i32::MAX);

    test_for::<f32>(0.0, 1.0);
    test_for::<f32>(0.3, -0.3);
    test_for::<f32>(f32::NEG_INFINITY, f32::INFINITY);
    test_for::<f32>(f32::NAN, f32::EPSILON);
}

#[test]
#[cfg_attr(miri, ignore)]
fn both_imm() {
    fn test_for_kind<T>(kind: SelectKind, lhs: T, rhs: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
    {
        let result = Reg::from(1);
        let condition = Reg::from(0);
        let lhs_reg = Reg::from(-1);
        let rhs_reg = Reg::from(-2);
        let instrs = [
            Instruction::select(result, lhs_reg),
            Instruction::register2_ext(condition, rhs_reg),
            Instruction::return_reg(result),
        ];
        test_both_imm(kind, lhs, rhs)
            .expect_func(ExpectedFunc::new(instrs).consts([lhs, rhs]))
            .run();
    }

    fn test_for<T>(lhs: T, rhs: T)
    where
        T: WasmTy,
        DisplayWasm<T>: Display,
    {
        test_for_kind(SelectKind::Select, lhs, rhs);
        test_for_kind(SelectKind::Select, rhs, lhs);
        test_for_kind(SelectKind::TypedSelect, lhs, rhs);
        test_for_kind(SelectKind::TypedSelect, rhs, lhs);
    }

    test_for::<i64>(i64::from(i32::MIN) - 1, i64::from(i32::MAX) + 1);
    test_for::<i64>(i64::MIN, i64::MAX);

    test_for::<f64>(0.3, -0.3);
    test_for::<f64>(0.123456789, -0.987654321);
}

#[test]
#[cfg_attr(miri, ignore)]
fn both_i64imm32() {
    fn test_for_kind(kind: SelectKind, lhs: i64, rhs: i64) {
        let result = Reg::from(1);
        let condition = Reg::from(0);
        let lhs32 = i64imm32(lhs);
        let rhs32 = i64imm32(rhs);
        let expected = [
            Instruction::select_i64imm32(result, lhs32),
            Instruction::register_and_imm32(condition, rhs32),
            Instruction::return_reg(result),
        ];
        test_both_imm(kind, lhs, rhs)
            .expect_func_instrs(expected)
            .run();
    }

    fn test_for(lhs: i64, rhs: i64) {
        test_for_kind(SelectKind::Select, lhs, rhs);
        test_for_kind(SelectKind::Select, rhs, lhs);
        test_for_kind(SelectKind::TypedSelect, lhs, rhs);
        test_for_kind(SelectKind::TypedSelect, rhs, lhs);
    }

    test_for(0, 1);
    test_for(-5, 42);
    test_for(i64::from(i32::MIN) + 1, i64::from(i32::MAX) - 1);
    test_for(i64::from(i32::MIN), i64::from(i32::MAX));
}

#[test]
#[cfg_attr(miri, ignore)]
fn both_f64imm32() {
    fn test_for_kind(kind: SelectKind, lhs: f64, rhs: f64) {
        let result = Reg::from(1);
        let condition = Reg::from(0);
        let lhs32 = f64imm32(lhs);
        let rhs32 = f64imm32(rhs);
        let expected = [
            Instruction::select_f64imm32(result, lhs32),
            Instruction::register_and_imm32(condition, rhs32),
            Instruction::return_reg(result),
        ];
        test_both_imm(kind, lhs, rhs)
            .expect_func_instrs(expected)
            .run();
    }

    fn test_for(lhs: f64, rhs: f64) {
        test_for_kind(SelectKind::Select, lhs, rhs);
        test_for_kind(SelectKind::Select, rhs, lhs);
        test_for_kind(SelectKind::TypedSelect, lhs, rhs);
        test_for_kind(SelectKind::TypedSelect, rhs, lhs);
    }

    test_for(0.0, 1.0);
    test_for(-5.5, 42.25);
    test_for(f64::NEG_INFINITY, f64::INFINITY);
    test_for(f64::NAN, f64::EPSILON);
}

#[test]
#[cfg_attr(miri, ignore)]
fn fuzz_fail_01() {
    let wasm = r#"
        (module
            (func (export "test") (param i32) (result i32)
                (i32.popcnt (local.get 0))        ;; case: true  (i32.const  0)
                (i32.clz (i32.eqz (local.get 0))) ;; case: false (i32.const 31)
                (i32.const 0)                     ;; condition   (i32.const  0)
                (select)                          ;; case: true  (i32.const 31)
                (i32.const 0)                     ;; case: false (i32.const  0)
                (i32.eqz (local.get 0))           ;; condition   (i32.const  1)
                (select)
            )
        )
    "#;
    TranslationTest::new(wasm)
        .expect_func_instrs([
            Instruction::i32_popcnt(1, 0),
            Instruction::i32_eq_imm16(2, 0, 0_i16),
            Instruction::i32_clz(2, 2),
            Instruction::copy(1, 2),
            Instruction::i32_eq_imm16(2, 0, 0_i16),
            Instruction::select_imm32_rhs(1, 1),
            Instruction::register_and_imm32(2, 0_i32),
            Instruction::return_reg(1),
        ])
        .run();
}
