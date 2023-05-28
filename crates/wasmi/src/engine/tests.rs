use super::{
    bytecode::{AddressOffset, BranchTableTargets, GlobalIdx},
    *,
};
use crate::{
    engine::{
        bytecode::{BranchOffset, Instruction},
        config::FuelCosts,
        DropKeep,
    },
    Engine,
    Module,
};

/// Converts the `wat` string source into `wasm` encoded byte.
fn wat2wasm(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).unwrap()
}

/// Compiles the `wasm` encoded bytes into a [`Module`].
///
/// # Panics
///
/// If an error occurred upon module compilation, validation or translation.
fn create_module(config: &Config, bytes: &[u8]) -> Module {
    let engine = Engine::new(config);
    Module::new(&engine, bytes).unwrap()
}

/// Contains some utility methods to construct instructions simpler.
mod instr {
    use super::Instruction;
    use crate::engine::bytecode::BlockFuel;

    /// Creates a new [`Instruction::LocalGet`] with the `local_depth`.
    ///
    /// # Panics
    ///
    /// If the `local_depth` is out of bounds.
    pub fn local_get(local_depth: u32) -> Instruction {
        Instruction::local_get(local_depth).unwrap()
    }

    /// Creates a new [`Instruction::LocalSet`] with the `local_depth`.
    ///
    /// # Panics
    ///
    /// If the `local_depth` is out of bounds.
    pub fn local_set(local_depth: u32) -> Instruction {
        Instruction::local_set(local_depth).unwrap()
    }

    /// Creates a new [`Instruction::ConsumeFuel`] with the `block_fuel`.
    ///
    /// # Panics
    ///
    /// If the `block_fuel` is out of bounds.
    pub fn consume_fuel(block_fuel: u64) -> Instruction {
        Instruction::ConsumeFuel(BlockFuel::try_from(block_fuel).unwrap())
    }

    /// Creates a new [`Instruction::Const32`] instruction for the `value`.
    ///
    /// # Panics
    ///
    /// If the `value` cannot be encoded into a 24-bit value.
    pub fn i32_const(value: i32) -> Instruction {
        Instruction::Const32(value.to_ne_bytes())
    }
}

/// Creates a [`CompiledFunc`] from the given `u32` index value.
///
/// # Panics
///
/// If the `u32` index value is out of bounds for the [`CompiledFunc`].
fn compiled_func(index: u32) -> CompiledFunc {
    CompiledFunc::from_usize(index as usize)
}

/// Creates a [`GlobalIdx`] from the given `u32` index value.
///
/// # Panics
///
/// If the `u32` index value is out of bounds for the [`GlobalIdx`].
fn global_idx(index: u32) -> GlobalIdx {
    GlobalIdx::try_from(index).unwrap()
}

/// Returns the [`BranchTableTargets`] for the given amount.
///
/// # Panics
///
/// If `len_targets` is out of bounds as [`BranchTableTargets`].
fn br_targets(len_targets: usize) -> BranchTableTargets {
    BranchTableTargets::try_from(len_targets).unwrap()
}

/// Asserts that the given `func_body` consists of the expected instructions.
///
/// # Panics
///
/// If there is an instruction mismatch between the actual instructions in
/// `func_body` and the `expected_instructions`.
fn assert_func_body<E>(
    engine: &Engine,
    func_type: DedupFuncType,
    func_body: CompiledFunc,
    expected_instructions: E,
) where
    E: IntoIterator<Item = Instruction>,
    <E as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let expected_instructions = expected_instructions.into_iter();
    let len_expected = expected_instructions.len();
    for (index, actual, expected) in
        expected_instructions
            .into_iter()
            .enumerate()
            .map(|(index, expected)| {
                (
                    index,
                    engine.resolve_instr(func_body, index).unwrap_or_else(|| {
                        panic!("encountered missing instruction at position {index}")
                    }),
                    expected,
                )
            })
    {
        assert_eq!(
            actual,
            expected,
            "encountered instruction mismatch for {:?} at position {index}",
            engine.resolve_func_type(&func_type, Clone::clone),
        );
    }
    if let Some(unexpected) = engine.resolve_instr(func_body, len_expected) {
        panic!("encountered unexpected instruction at position {len_expected}: {unexpected:?}",);
    }
}

/// Asserts that the given `wasm` bytes yield functions with expected instructions.
///
/// Uses the given [`Config`] to configure the [`Engine`] that the tests are run on.
///
/// # Panics
///
/// If any of the yielded functions consists of instruction different from the
/// expected instructions for that function.
fn assert_func_bodies_with_config<E, T>(config: &Config, wasm_bytes: impl AsRef<[u8]>, expected: E)
where
    E: IntoIterator<Item = T>,
    T: IntoIterator<Item = Instruction>,
    <T as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let wasm_bytes = wasm_bytes.as_ref();
    let module = create_module(config, wasm_bytes);
    let engine = module.engine();
    for ((func_type, func_body), expected) in module.internal_funcs().zip(expected) {
        assert_func_body(engine, func_type, func_body, expected);
    }
}

/// Asserts that the given `wasm` bytes yield functions with expected instructions.
///
/// Uses a default [`Config`] for the tests.
///
/// # Panics
///
/// If any of the yielded functions consists of instruction different from the
/// expected instructions for that function.
fn assert_func_bodies<E, T>(wasm_bytes: impl AsRef<[u8]>, expected: E)
where
    E: IntoIterator<Item = T>,
    T: IntoIterator<Item = Instruction>,
    <T as IntoIterator>::IntoIter: ExactSizeIterator,
{
    assert_func_bodies_with_config(&Config::default(), wasm_bytes, expected)
}

/// Asserts that the given `wasm` bytes yield functions with expected instructions.
///
/// Uses a [`Config`] for the tests where fuel metering is enabled.
///
/// # Panics
///
/// If any of the yielded functions consists of instruction different from the
/// expected instructions for that function.
fn assert_func_bodies_metered<E, T>(wasm_bytes: impl AsRef<[u8]>, expected: E)
where
    E: IntoIterator<Item = T>,
    T: IntoIterator<Item = Instruction>,
    <T as IntoIterator>::IntoIter: ExactSizeIterator,
{
    let mut config = Config::default();
    config.consume_fuel(true);
    assert_func_bodies_with_config(&config, wasm_bytes, expected)
}

fn drop_keep(drop: usize, keep: usize) -> DropKeep {
    DropKeep::new(drop, keep).unwrap()
}

#[test]
fn implicit_return_no_value() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
            )
        )
    "#,
    );
    let expected = [Instruction::Return(drop_keep(0, 0))];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn implicit_return_with_value() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (result i32)
                i32.const 0
            )
        )
    "#,
    );
    let expected = [instr::i32_const(0), Instruction::Return(drop_keep(0, 1))];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn implicit_return_param() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32)
            )
        )
    "#,
    );
    let expected = [Instruction::Return(drop_keep(1, 0))];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn get_local() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (result i32)
                local.get 0
            )
        )
    "#,
    );
    let expected = [instr::local_get(1), Instruction::Return(drop_keep(1, 1))];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn get_local_2() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (param i32) (result i32)
                local.get 0
                local.get 1
                drop
            )
        )
    "#,
    );
    let expected = [
        instr::local_get(2),
        instr::local_get(2),
        Instruction::Drop,
        Instruction::Return(drop_keep(2, 1)),
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn get_local_3() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (param i32)
                local.get 0
                local.get 1
                drop
                drop
            )
        )
    "#,
    );
    let expected = [
        instr::local_get(2),
        instr::local_get(2),
        Instruction::Drop,
        Instruction::Drop,
        Instruction::Return(drop_keep(2, 0)),
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn explicit_return() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (result i32)
                local.get 0
                return
            )
        )
    "#,
    );
    let expected = [instr::local_get(1), Instruction::Return(drop_keep(1, 1))];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn simple_add() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (param i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
        )
    "#,
    );
    let expected = [
        instr::local_get(2),
        instr::local_get(2),
        Instruction::I32Add,
        Instruction::Return(drop_keep(2, 1)),
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn simple_mul_add() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (param i32) (result i32)
                local.get 0
                local.get 1
                local.get 0
                local.get 1
                i32.add
                i32.add
                i32.mul
            )
        )
    "#,
    );
    let expected = [
        instr::local_get(2),
        instr::local_get(2),
        instr::local_get(4),
        instr::local_get(4),
        Instruction::I32Add,
        Instruction::I32Add,
        Instruction::I32Mul,
        Instruction::Return(drop_keep(2, 1)),
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn drop_locals() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32)
                (local i32)
                local.get 0
                local.set 1
            )
        )
    "#,
    );
    let expected = [
        instr::local_get(2),
        instr::local_set(1),
        Instruction::Return(drop_keep(2, 0)),
    ];
    assert_func_bodies(wasm, [expected]);
}

macro_rules! offset {
    ( $src:expr => $dst:expr ) => {
        BranchOffset::try_from($dst - $src).unwrap()
    };
}

#[test]
fn if_without_else() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (result i32)
                i32.const 1
                if
                    i32.const 2
                    return
                end
                i32.const 3
            )
        )
    "#,
    );
    let expected = [
        /* 0 */ instr::i32_const(1),
        /* 1 */ Instruction::BrIfEqz(offset!(1 => 4)),
        /* 2 */ instr::i32_const(2),
        /* 3 */ Instruction::Return(drop_keep(1, 1)),
        /* 4 */ instr::i32_const(3),
        /* 5 */ Instruction::Return(drop_keep(1, 1)),
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn if_else() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                (local i32)
                i32.const 1
                if
                    i32.const 2
                    local.set 0
                else
                    i32.const 3
                    local.set 0
                end
            )
        )
    "#,
    );
    let expected = [
        /* 0 */ instr::i32_const(1),
        /* 1 */ Instruction::BrIfEqz(offset!(1 => 5)),
        /* 2 */ instr::i32_const(2),
        /* 3 */ instr::local_set(1),
        /* 4 */ Instruction::Br(offset!(5 => 8)),
        /* 5 */ instr::i32_const(3),
        /* 6 */ instr::local_set(1),
        /* 7 */ Instruction::Return(drop_keep(1, 0)),
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn if_else_returns_result() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                i32.const 1
                if (result i32)
                    i32.const 2
                else
                    i32.const 3
                end
                drop
            )
        )
    "#,
    );
    let expected = [
        /* 0 */ instr::i32_const(1),
        /* 1 */ Instruction::BrIfEqz(offset!(1 => 4)),
        /* 2 */ instr::i32_const(2),
        /* 3 */ Instruction::Br(offset!(4 => 6)),
        /* 4 */ instr::i32_const(3),
        /* 5 */ Instruction::Drop,
        /* 6 */ Instruction::Return(drop_keep(0, 0)),
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn if_else_branch_from_true_branch() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                i32.const 1
                if (result i32)
                    i32.const 1
                    i32.const 1
                    br_if 0
                    drop
                    i32.const 2
                else
                    i32.const 3
                end
                drop
            )
        )
    "#,
    );
    let expected = [
        /*  0 */ instr::i32_const(1),
        /*  1 */ Instruction::BrIfEqz(offset!(1 => 8)),
        /*  2 */ instr::i32_const(1),
        /*  3 */ instr::i32_const(1),
        /*  4 */ Instruction::BrIfNez(offset!(4 => 9)),
        /*  5 */ Instruction::Drop,
        /*  6 */ instr::i32_const(2),
        /*  7 */ Instruction::Br(offset!(7 => 9)),
        /*  8 */ instr::i32_const(3),
        /*  9 */ Instruction::Drop,
        /* 10 */ Instruction::Return(drop_keep(0, 0)),
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn if_else_branch_from_false_branch() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                i32.const 1
                if (result i32)
                    i32.const 1
                else
                    i32.const 2
                    i32.const 1
                    br_if 0
                    drop
                    i32.const 3
                end
                drop
            )
        )
    "#,
    );
    let expected = [
        /*  0 */ instr::i32_const(1),
        /*  1 */ Instruction::BrIfEqz(offset!(1 => 4)),
        /*  2 */ instr::i32_const(1),
        /*  3 */ Instruction::Br(offset!(3 => 9)),
        /*  4 */ instr::i32_const(2),
        /*  5 */ instr::i32_const(1),
        /*  6 */ Instruction::BrIfNez(offset!(6 => 9)),
        /*  7 */ Instruction::Drop,
        /*  8 */ instr::i32_const(3),
        /*  9 */ Instruction::Drop,
        /* 10 */ Instruction::Return(drop_keep(0, 0)),
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn if_else_both_unreachable_before_end() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (result i32)
                local.get 0
                if (result i32)
                    i32.const 1
                    return
                    i32.const 100 ;; unreachable
                else
                    i32.const 2
                    return
                    i32.const 200 ;; unreachable
                end
                drop
                i32.const 3
            )
        )
    "#,
    );
    let expected = [
        /* 0 */ instr::local_get(1),
        /* 1 */ Instruction::BrIfEqz(offset!(1 => 4)),
        /* 2 */ instr::i32_const(1),
        /* 3 */ Instruction::Return(drop_keep(1, 1)),
        /* 4 */ instr::i32_const(2),
        /* 5 */ Instruction::Return(drop_keep(1, 1)),
        /* 6 */ Instruction::Drop,
        /* 7 */ instr::i32_const(3),
        /* 8 */ Instruction::Return(drop_keep(1, 1)),
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn loop_() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                loop (result i32)
                    i32.const 1
                    br_if 0
                    i32.const 2
                end
                drop
            )
        )
    "#,
    );
    let expected = [
        /* 0 */ instr::i32_const(1),
        /* 1 */ Instruction::BrIfNez(offset!(1 => 0)),
        /* 3 */ instr::i32_const(2),
        /* 4 */ Instruction::Drop,
        /* 5 */ Instruction::Return(drop_keep(0, 0)),
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn loop_empty() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                loop
                end
            )
        )
    "#,
    );
    let expected = [Instruction::Return(drop_keep(0, 0))];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn spec_as_br_if_value_cond() {
    let wasm = wat2wasm(
        r#"
            (func (export "as-br_if-value-cond") (result i32)
                (block (result i32)
                    (drop
                        (br_if 0
                            (i32.const 6)
                            (br_table 0 0
                                (i32.const 9)
                                (i32.const 0)
                            )
                        )
                    )
                    (i32.const 7)
                )
            )
    "#,
    );
    let expected = [
        /* 0 */ instr::i32_const(6),
        /* 1 */ instr::i32_const(9),
        /* 2 */ instr::i32_const(0),
        /* 3 */ Instruction::BrTable(br_targets(2)),
        /* 4 */ Instruction::BrAdjust(offset!(4 => 8)),
        /* 5 */ Instruction::Return(drop_keep(1, 1)),
        /* 6 */ Instruction::BrAdjust(offset!(6 => 8)),
        /* 7 */ Instruction::Return(drop_keep(1, 1)),
        /* 8 */ Instruction::Return(drop_keep(0, 1)),
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn br_table() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                block $1
                    loop $2
                        i32.const 0
                        br_table $2 $1
                    end
                end
            )
        )
    "#,
    );
    let expected = [
        /* 0 */ instr::i32_const(0),
        /* 1 */ Instruction::BrTable(br_targets(2)),
        /* 2 */ Instruction::BrAdjust(offset!(2 => 0)),
        /* 3 */ Instruction::Return(drop_keep(0, 0)),
        /* 4 */ Instruction::BrAdjust(offset!(4 => 6)),
        /* 5 */ Instruction::Return(drop_keep(0, 0)),
        /* 6 */ Instruction::Return(drop_keep(0, 0)),
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn br_table_returns_result() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                block $1 (result i32)
                    block $2 (result i32)
                        i32.const 0
                        i32.const 1
                        br_table $2 $1
                    end
                    unreachable
                end
                drop
            )
        )
    "#,
    );
    let expected = [
        /* 0 */ instr::i32_const(0),
        /* 1 */ instr::i32_const(1),
        /* 2 */ Instruction::BrTable(br_targets(2)),
        /* 3 */ Instruction::BrAdjust(offset!(3 => 7)),
        /* 4 */ Instruction::Return(drop_keep(0, 1)),
        /* 5 */ Instruction::BrAdjust(offset!(5 => 8)),
        /* 6 */ Instruction::Return(drop_keep(0, 1)),
        /* 7 */ Instruction::Unreachable,
        /* 8 */ Instruction::Drop,
        /* 9 */ Instruction::Return(drop_keep(0, 0)),
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn wabt_example() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32) (result i32)
                block $exit
                    get_local 0
                    br_if $exit
                    i32.const 1
                    return
                end
                i32.const 2
                return
            )
        )
    "#,
    );
    let expected = [
        /* 0 */ instr::local_get(1),
        /* 1 */ Instruction::BrIfNez(offset!(1 => 4)),
        /* 2 */ instr::i32_const(1),
        /* 3 */ Instruction::Return(drop_keep(1, 1)),
        /* 4 */ instr::i32_const(2),
        /* 5 */ Instruction::Return(drop_keep(1, 1)),
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn br_return() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call")
                br 0
                return
            )
        )
    "#,
    );
    let expected = [Instruction::Return(drop_keep(0, 0))];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn br_if_return() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32)
                local.get 0
                br_if 0
                return
            )
        )
    "#,
    );
    let expected = [
        instr::local_get(1),
        Instruction::ReturnIfNez(drop_keep(1, 0)),
        Instruction::Return(drop_keep(1, 0)),
    ];
    assert_func_bodies(wasm, [expected]);
}

#[test]
fn br_table_return() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (export "call") (param i32)
                block $1
                    block $0
                        local.get 0
                        br_table $0 $1 2
                    end
                end
                return
            )
        )
    "#,
    );
    let expected = [
        /* 0 */ instr::local_get(1),
        /* 1 */ Instruction::BrTable(br_targets(3)),
        /* 2 */ Instruction::BrAdjust(offset!(2 => 8)),
        /* 3 */ Instruction::Return(drop_keep(0, 0)),
        /* 4 */ Instruction::BrAdjust(offset!(4 => 8)),
        /* 5 */ Instruction::Return(drop_keep(0, 0)),
        /* 6 */ Instruction::Return(drop_keep(1, 0)),
        /* 7 */ Instruction::Return(drop_keep(1, 0)),
        /* 8 */ Instruction::Return(drop_keep(1, 0)),
    ];
    assert_func_bodies(wasm, [expected]);
}

/// Returns the default [`FuelCosts`].
pub fn fuel_costs() -> FuelCosts {
    *Config::default().fuel_costs()
}

#[test]
fn metered_simple_01() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param $p0 i32) (result i32)
                local.get $p0
            )
        )
    "#,
    );
    let costs = fuel_costs();
    let expected_fuel =
        3 * costs.base + costs.fuel_for_locals(1) + costs.fuel_for_drop_keep(drop_keep(1, 1));
    let expected = [
        instr::consume_fuel(expected_fuel),
        instr::local_get(1),
        Instruction::Return(drop_keep(1, 1)),
    ];
    assert_func_bodies_metered(wasm, [expected]);
}

#[test]
fn metered_simple_02() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param $p0 i32) (result i32)
                local.get $p0
                (block (result i32)
                    local.get $p0
                )
                drop
            )
        )
    "#,
    );
    let costs = fuel_costs();
    let expected_fuel =
        5 * costs.base + costs.fuel_for_locals(1) + costs.fuel_for_drop_keep(drop_keep(1, 1));
    let expected = [
        instr::consume_fuel(expected_fuel),
        instr::local_get(1),
        instr::local_get(2),
        Instruction::Drop,
        Instruction::Return(drop_keep(1, 1)),
    ];
    assert_func_bodies_metered(wasm, [expected]);
}

#[test]
fn metered_simple_03() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param $a i32) (param $b i32) (result i32)
                (local.set $a ;; c = a + b
                    (i32.add
                        (local.get $a)
                        (local.get $b)
                    )
                )
                (i32.mul (local.get $a) (local.get $a))
            )
        )
    "#,
    );
    let costs = fuel_costs();
    let expected_fuel =
        9 * costs.base + costs.fuel_for_locals(2) + costs.fuel_for_drop_keep(drop_keep(2, 1));
    let expected = [
        instr::consume_fuel(expected_fuel),
        instr::local_get(2),
        instr::local_get(2),
        Instruction::I32Add,
        instr::local_set(2),
        instr::local_get(2),
        instr::local_get(3),
        Instruction::I32Mul,
        Instruction::Return(drop_keep(2, 1)),
    ];
    assert_func_bodies_metered(wasm, [expected]);
}

#[test]
fn metered_if_01() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param $condition i32) (param $then i32) (param $else i32) (result i32)
                (if (result i32) (local.get $condition)
                    (then
                        (return (local.get $then))
                    )
                    (else 
                        (return (local.get $else))
                    )
                )
            )
        )
    "#,
    );
    let costs = fuel_costs();
    let expected_fuel_fn =
        4 * costs.base + costs.fuel_for_locals(3) + costs.fuel_for_drop_keep(drop_keep(3, 1));
    let expected_fuel_then = 3 * costs.base + costs.fuel_for_drop_keep(drop_keep(3, 1));
    let expected_fuel_else = expected_fuel_then;
    let expected = [
        /*  0 */ instr::consume_fuel(expected_fuel_fn), // function body
        /*  1 */ instr::local_get(3), // if condition
        /*  2 */ Instruction::BrIfEqz(offset!(2 => 6)),
        /*  3 */ instr::consume_fuel(expected_fuel_then), // then
        /*  4 */ instr::local_get(2),
        /*  5 */ Instruction::Return(drop_keep(3, 1)),
        /*  6 */ instr::consume_fuel(expected_fuel_else), // else
        /*  7 */ instr::local_get(1),
        /*  8 */ Instruction::Return(drop_keep(3, 1)), // end if
        /*  9 */ Instruction::Return(drop_keep(3, 1)),
    ];
    assert_func_bodies_metered(wasm, [expected]);
}

#[test]
fn metered_block_in_if_01() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param $condition i32) (param $then i32) (param $else i32) (result i32)
                (if (result i32) (local.get $condition)
                    (then
                        (block (result i32)
                            (return (local.get $then))
                        )
                    )
                    (else
                        (block (result i32)
                            (return (local.get $else))
                        )
                    )
                )
            )
        )
    "#,
    );
    let costs = fuel_costs();
    let expected_fuel_fn =
        5 * costs.base + costs.fuel_for_locals(3) + costs.fuel_for_drop_keep(drop_keep(3, 1));
    let expected_fuel_then = 3 * costs.base + costs.fuel_for_drop_keep(drop_keep(3, 1));
    let expected_fuel_else = expected_fuel_then;
    #[rustfmt::skip]
    let expected = [
        /*  0 */ instr::consume_fuel(expected_fuel_fn), // function body
        /*  1 */ instr::local_get(3), // if condition
        /*  2 */ Instruction::BrIfEqz(offset!(2 => 7)),
        /*  3 */ instr::consume_fuel(expected_fuel_then), // then
        /*  4 */ instr::local_get(2),
        /*  5 */ Instruction::Return(drop_keep(3, 1)),
        /*  6 */ Instruction::Br(offset!(6 => 10)), // This deadcode Br is created because
                                                    // `wasmi`'s dead code analysis does not
                                                    // properly detect dead code in blocks
                                                    // (and loops) that have an unreachable end.
        /*  7 */ instr::consume_fuel(expected_fuel_else), // else
        /*  8 */ instr::local_get(1),
        /*  9 */ Instruction::Return(drop_keep(3, 1)), // end if
        /* 10 */ Instruction::Return(drop_keep(3, 1)),
    ];
    assert_func_bodies_metered(wasm, [expected]);
}

#[test]
fn metered_block_in_if_02() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param $condition i32) (param $then i32) (param $else i32) (result i32)
                (if (result i32) (local.get $condition)
                    (then
                        (block (result i32)
                            (local.get $then)
                        )
                    )
                    (else
                        (block (result i32)
                            (local.get $else)
                        )
                    )
                )
            )
        )
    "#,
    );
    let costs = fuel_costs();
    let expected_fuel_fn =
        5 * costs.base + costs.fuel_for_locals(3) + costs.fuel_for_drop_keep(drop_keep(3, 1));
    let expected_fuel_then = 2 * costs.base;
    let expected_fuel_else = expected_fuel_then;
    let expected = [
        /*  0 */ instr::consume_fuel(expected_fuel_fn), // function body
        /*  1 */ instr::local_get(3), // if condition
        /*  2 */ Instruction::BrIfEqz(offset!(2 => 6)),
        /*  3 */ instr::consume_fuel(expected_fuel_then), // then
        /*  4 */ instr::local_get(2),
        /*  5 */ Instruction::Br(offset!(5 => 8)),
        /*  6 */ instr::consume_fuel(expected_fuel_else), // else
        /*  7 */ instr::local_get(1),
        /*  8 */ Instruction::Return(drop_keep(3, 1)), // end if
    ];
    assert_func_bodies_metered(wasm, [expected]);
}

#[test]
fn metered_loop_in_if() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param $condition i32) (param $then i32) (param $else i32) (result i32)
                (if (result i32) (local.get $condition)
                    (then
                        (loop (result i32)
                            (local.get $then)
                        )
                    )
                    (else
                        (loop (result i32)
                            (local.get $else)
                        )
                    )
                )
            )
        )
    "#,
    );
    let costs = fuel_costs();
    let expected_fuel_fn =
        5 * costs.base + costs.fuel_for_locals(3) + costs.fuel_for_drop_keep(drop_keep(3, 1));
    let expected_fuel_then = costs.base;
    let expected_fuel_else = expected_fuel_then;
    let expected_fuel_loop = 2 * costs.base;
    let expected = [
        /*  0 */ instr::consume_fuel(expected_fuel_fn), // function body
        /*  1 */ instr::local_get(3), // if condition
        /*  2 */ Instruction::BrIfEqz(offset!(2 => 7)),
        /*  3 */ instr::consume_fuel(expected_fuel_then), // then
        /*  4 */ instr::consume_fuel(expected_fuel_loop), // loop
        /*  5 */ instr::local_get(2),
        /*  6 */ Instruction::Br(offset!(6 => 10)),
        /*  7 */ instr::consume_fuel(expected_fuel_else), // else
        /*  8 */ instr::consume_fuel(expected_fuel_loop), // loop
        /*  9 */ instr::local_get(1),
        /* 10 */ Instruction::Return(drop_keep(3, 1)),
    ];
    assert_func_bodies_metered(wasm, [expected]);
}

#[test]
fn metered_nested_blocks() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param $p0 i32) (result i32)
                local.get $p0
                (block
                    local.get $p0
                    (block
                        local.get $p0
                        (block
                            local.get $p0
                            (block
                                local.get $p0
                                drop
                            )
                            drop
                        )
                        drop
                    )
                    drop
                )
            )
        )
    "#,
    );
    let costs = fuel_costs();
    let expected_fuel =
        11 * costs.base + costs.fuel_for_locals(1) + costs.fuel_for_drop_keep(drop_keep(1, 1));
    let expected = [
        instr::consume_fuel(expected_fuel),
        instr::local_get(1),
        instr::local_get(2),
        instr::local_get(3),
        instr::local_get(4),
        instr::local_get(5),
        Instruction::Drop,
        Instruction::Drop,
        Instruction::Drop,
        Instruction::Drop,
        Instruction::Return(drop_keep(1, 1)),
    ];
    assert_func_bodies_metered(wasm, [expected]);
}

#[test]
fn metered_nested_loops() {
    let wasm = wat2wasm(
        r#"
        (module
            (func (param $p0 i32) (result i32)
                local.get $p0
                (loop
                    local.get $p0
                    (loop
                        local.get $p0
                        (loop
                            local.get $p0
                            (loop
                                local.get $p0
                                drop
                            )
                            drop
                        )
                        drop
                    )
                    drop
                )
            )
        )
    "#,
    );
    let costs = fuel_costs();
    let expected_fuel_outer =
        3 * costs.base + costs.fuel_for_locals(1) + costs.fuel_for_drop_keep(drop_keep(1, 1));
    let expected_fuel_inner = 3 * costs.base;
    let expected = [
        instr::consume_fuel(expected_fuel_outer),
        instr::local_get(1),
        instr::consume_fuel(expected_fuel_inner),
        instr::local_get(2),
        instr::consume_fuel(expected_fuel_inner),
        instr::local_get(3),
        instr::consume_fuel(expected_fuel_inner),
        instr::local_get(4),
        instr::consume_fuel(expected_fuel_inner),
        instr::local_get(5),
        Instruction::Drop,
        Instruction::Drop,
        Instruction::Drop,
        Instruction::Drop,
        Instruction::Return(drop_keep(1, 1)),
    ];
    assert_func_bodies_metered(wasm, [expected]);
}

#[test]
fn metered_global_bump() {
    let wasm = wat2wasm(
        r#"
        (module
            (global $g (mut i32) (i32.const 0))
            ;; Increases $g by $delta and returns the new value.
            (func (param $delta i32) (result i32)
                (global.set $g
                    (i32.add
                        (global.get $g)
                        (local.get $delta)
                    )
                )
                (global.get $g)
            )
        )
    "#,
    );
    let costs = fuel_costs();
    let expected_fuel = 3 * costs.entity
        + 4 * costs.base
        + costs.fuel_for_locals(1)
        + costs.fuel_for_drop_keep(drop_keep(1, 1));
    let expected = [
        instr::consume_fuel(expected_fuel),
        Instruction::GlobalGet(global_idx(0)),
        instr::local_get(2),
        Instruction::I32Add,
        Instruction::GlobalSet(global_idx(0)),
        Instruction::GlobalGet(global_idx(0)),
        Instruction::Return(drop_keep(1, 1)),
    ];
    assert_func_bodies_metered(wasm, [expected]);
}

#[test]
fn metered_calls_01() {
    let wasm = wat2wasm(
        r#"
        (module
            (func $f0 (result i32)
                (i32.const 0)
            )
            (func $f1 (result i32)
                (call $f0)
            )
        )
    "#,
    );
    let costs = fuel_costs();
    let expected_fuel_f0 = 3 * costs.base;
    let expected_f0 = [
        instr::consume_fuel(expected_fuel_f0),
        instr::i32_const(0),
        Instruction::Return(drop_keep(0, 1)),
    ];
    let expected_fuel_f1 = 2 * costs.base + costs.call;
    let expected_f1 = [
        instr::consume_fuel(expected_fuel_f1),
        Instruction::CallInternal(compiled_func(0)),
        Instruction::Return(drop_keep(0, 1)),
    ];
    assert_func_bodies_metered(wasm, [expected_f0, expected_f1]);
}

#[test]
fn metered_calls_02() {
    let wasm = wat2wasm(
        r#"
        (module
            (func $f0 (param $a i32) (param $b i32) (result i32)
                (i32.add
                    (local.get $a)
                    (local.get $b)
                )
            )
            (func $f1 (param $a i32) (param $b i32) (result i32)
                (call $f0
                    (local.get $a)
                    (local.get $b)
                )
            )
        )
    "#,
    );
    let costs = fuel_costs();
    let expected_fuel_f0 =
        5 * costs.base + costs.fuel_for_locals(2) + costs.fuel_for_drop_keep(drop_keep(2, 1));
    let expected_f0 = [
        instr::consume_fuel(expected_fuel_f0),
        instr::local_get(2),
        instr::local_get(2),
        Instruction::I32Add,
        Instruction::Return(drop_keep(2, 1)),
    ];
    let expected_fuel_f1 = 4 * costs.base
        + costs.call
        + costs.fuel_for_locals(2)
        + costs.fuel_for_drop_keep(drop_keep(2, 1));
    let expected_f1 = [
        instr::consume_fuel(expected_fuel_f1),
        instr::local_get(2),
        instr::local_get(2),
        Instruction::CallInternal(compiled_func(0)),
        Instruction::Return(drop_keep(2, 1)),
    ];
    assert_func_bodies_metered(wasm, [expected_f0, expected_f1]);
}

#[test]
fn metered_calls_03() {
    let wasm = wat2wasm(
        r#"
        (module
            (func $f0 (param $a i32) (result i32)
                (local $b i32) ;; index 1
                (local.set $b (local.get $a))
                (i32.add
                    (local.get $a)
                    (local.get $b)
                )
            )
            (func $f1 (param $a i32) (result i32)
                (call $f0
                    (local.get $a)
                )
            )
        )
    "#,
    );
    let costs = fuel_costs();
    let expected_fuel_f0 =
        7 * costs.base + costs.fuel_for_locals(2) + costs.fuel_for_drop_keep(drop_keep(2, 1));
    let expected_f0 = [
        instr::consume_fuel(expected_fuel_f0),
        instr::local_get(2),
        instr::local_set(1),
        instr::local_get(2),
        instr::local_get(2),
        Instruction::I32Add,
        Instruction::Return(drop_keep(2, 1)),
    ];
    let expected_fuel_f1 = 3 * costs.base
        + costs.call
        + costs.fuel_for_locals(1)
        + costs.fuel_for_drop_keep(drop_keep(1, 1));
    let expected_f1 = [
        instr::consume_fuel(expected_fuel_f1),
        instr::local_get(1),
        Instruction::CallInternal(compiled_func(0)),
        Instruction::Return(drop_keep(1, 1)),
    ];
    assert_func_bodies_metered(
        &wasm,
        [expected_f0.iter().copied(), expected_f1.iter().copied()],
    );
}

#[test]
fn metered_load_01() {
    let wasm = wat2wasm(
        r#"
        (module
            (memory 1)
            (func (param $src i32) (result i32)
                (i32.load (local.get $src))
            )
        )
    "#,
    );
    let costs = fuel_costs();
    let expected_fuel = 3 * costs.base
        + costs.load
        + costs.fuel_for_locals(1)
        + costs.fuel_for_drop_keep(drop_keep(1, 1));
    let expected = [
        instr::consume_fuel(expected_fuel),
        instr::local_get(1),
        Instruction::I32Load(AddressOffset::from(0)),
        Instruction::Return(drop_keep(1, 1)),
    ];
    assert_func_bodies_metered(wasm, [expected]);
}

#[test]
fn metered_store_01() {
    let wasm = wat2wasm(
        r#"
        (module
            (memory 1)
            (func (param $dst i32) (param $value i32)
                (i32.store
                    (local.get $dst) (local.get $value)
                )
            )
        )
    "#,
    );
    let costs = fuel_costs();
    let expected_fuel = 4 * costs.base + costs.store + costs.fuel_for_locals(2);
    let expected = [
        instr::consume_fuel(expected_fuel),
        instr::local_get(2),
        instr::local_get(2),
        Instruction::I32Store(AddressOffset::from(0)),
        Instruction::Return(drop_keep(2, 0)),
    ];
    assert_func_bodies_metered(wasm, [expected]);
}
