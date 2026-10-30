//! Regression tests for the fusion of `select` with a zero test of its condition
//! (`i32.eqz`, `i32.eq` or `i32.ne` against `0`), see issue #2046.
//!
//! When the tested operand lives in a slot (for example a `local.get`), the fused
//! `select` must test that slot and not the result of the dropped comparison.

use wasmi::{Engine, Linker, Module, Store};

const SELECT_WASM: &str = r#"
    (module
        (func (export "eqz") (param $a i32) (param $b i32) (result i32)
            (select (i32.const 7) (local.get $a) (i32.eqz (local.get $b))))
        (func (export "eq_0") (param $a i32) (param $b i32) (result i32)
            (select (i32.const 7) (local.get $a) (i32.eq (local.get $b) (i32.const 0))))
        (func (export "ne_0") (param $a i32) (param $b i32) (result i32)
            (select (i32.const 7) (local.get $a) (i32.ne (local.get $b) (i32.const 0))))
        (func (export "ne_0_lhs") (param $a i32) (param $b i32) (result i32)
            (select (i32.const 7) (local.get $a) (i32.ne (i32.const 0) (local.get $b))))
        (func (export "eqz_eqz") (param $a i32) (param $b i32) (result i32)
            (select (i32.const 7) (local.get $a) (i32.eqz (i32.eqz (local.get $b)))))
        (func (export "eqz_computed") (param $a i32) (param $b i32) (result i32)
            (select (i32.const 7) (local.get $a) (i32.eqz (i32.add (local.get $b) (i32.const 1)))))
    )
"#;

/// Name of an exported function and the condition it tests on `b`.
type Case = (&'static str, fn(i32) -> bool);

/// Expected result of `select(7, a, condition(b))` with `a = 5`.
fn expected(condition: bool) -> i32 {
    if condition { 7 } else { 5 }
}

#[test]
fn select_fused_with_zero_test() -> Result<(), wasmi::Error> {
    let engine = Engine::default();
    let mut store = Store::new(&engine, ());
    let module = Module::new(&engine, SELECT_WASM)?;
    let instance = Linker::new(&engine).instantiate_and_start(&mut store, &module)?;
    let cases: [Case; 6] = [
        ("eqz", |b| b == 0),
        ("eq_0", |b| b == 0),
        ("ne_0", |b| b != 0),
        ("ne_0_lhs", |b| b != 0),
        ("eqz_eqz", |b| b != 0),
        ("eqz_computed", |b| b.wrapping_add(1) == 0),
    ];
    for (name, condition) in cases {
        let func = instance.get_typed_func::<(i32, i32), i32>(&store, name)?;
        for b in [0, 1, -1, 9] {
            let result = func.call(&mut store, (5, b))?;
            assert_eq!(result, expected(condition(b)), "{name}(5, {b})");
        }
    }
    Ok(())
}
