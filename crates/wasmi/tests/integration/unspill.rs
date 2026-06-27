//! Regression tests for the "avoid unnecessary register spill" translation optimization.
//!
//! When a value `A` resides in the accumulator register and the next operator `B` (a compare)
//! is fused into a `br_if`, the spill of `A` that was emitted to make room for `B` becomes
//! unnecessary. The translator reverts that spill and keeps `A` in its register. These tests
//! make sure the reverted value is preserved correctly on both edges of the fused branch.

use wasmi::{Config, Engine, Module, Store};

/// Compiles `wat`, runs the exported `test(a, b, c, d) -> i32` function and returns its result.
fn run(wat: &str, fuel: bool, params: (i32, i32, i32, i32)) -> i32 {
    let mut config = Config::default();
    config.consume_fuel(fuel);
    let engine = Engine::new(&config);
    let mut store = Store::new(&engine, ());
    if fuel {
        // Plenty of fuel: we only care that execution succeeds and yields the right value.
        store.set_fuel(1_000_000).unwrap();
    }
    let module = Module::new(&engine, wat.as_bytes()).unwrap();
    let instance = wasmi::Linker::new(&engine)
        .instantiate_and_start(&mut store, &module)
        .unwrap();
    let func = instance
        .get_typed_func::<(i32, i32, i32, i32), i32>(&store, "test")
        .unwrap();
    func.call(&mut store, params).unwrap()
}

/// Asserts that `test(a, b, c, d)` equals `expected(a, b, c, d)` for a matrix of inputs that
/// exercises both the taken and not-taken edges of the fused `br_if`, with and without fuel.
fn assert_matches_oracle(wat: &str, expected: impl Fn(i32, i32, i32, i32) -> i32) {
    let inputs = [-3i32, -1, 0, 1, 5, 7];
    for &a in &inputs {
        for &b in &inputs {
            // `c, d` drive whether the fused `c < d` branch is taken (true) or not (false).
            for (c, d) in [(0, 1), (1, 0), (2, 2), (-1, 1)] {
                let want = expected(a, b, c, d);
                for fuel in [false, true] {
                    let got = run(wat, fuel, (a, b, c, d));
                    assert_eq!(
                        got, want,
                        "test({a}, {b}, {c}, {d}) with fuel={fuel} returned {got}, expected {want}",
                    );
                }
            }
        }
    }
}

/// `A = a & b` is produced into the accumulator register by a non-`add` producer, so the spill
/// caused by the following `c < d` compare is a standalone `copy_sr` operator. The `br_if` fuses
/// the compare and the spill is reverted. The result differs between the taken edge (returns `A`,
/// which is only correct if the un-spilled register still holds `a & b`) and the not-taken edge
/// (returns `-1`), so a corrupted un-spill is observable.
#[test]
#[cfg_attr(not(feature = "wat"), ignore)]
fn unspill_standalone_copy() {
    let wat = r#"
    (module
        (func (export "test") (param i32 i32 i32 i32) (result i32)
            (block (result i32)
                (i32.and (local.get 0) (local.get 1)) ;; A = a & b, into `ireg`
                (i32.lt_s (local.get 2) (local.get 3)) ;; cond = c < d, spills A via `copy_sr`
                (br_if 0)         ;; fuses `lt_s`; taken => block result = A
                (drop)            ;; not taken => drop A ...
                (i32.const -1)    ;; ... and return -1 instead
            )
        )
    )"#;
    assert_matches_oracle(wat, |a, b, c, d| if c < d { a & b } else { -1 });
}

/// Control case: `A = a + b` uses the `fuse_copy_sr` producer path (the producer writes both
/// `slot` and register, no standalone `copy_sr`), so no spill is reverted. Execution must still
/// be correct on both edges.
#[test]
#[cfg_attr(not(feature = "wat"), ignore)]
fn unspill_fused_producer_control() {
    let wat = r#"
    (module
        (func (export "test") (param i32 i32 i32 i32) (result i32)
            (block (result i32)
                (i32.add (local.get 0) (local.get 1)) ;; A = a + b, fused producer
                (i32.lt_s (local.get 2) (local.get 3)) ;; cond = c < d
                (br_if 0)
                (drop)
                (i32.const -1)
            )
        )
    )"#;
    assert_matches_oracle(wat, |a, b, c, d| if c < d { a.wrapping_add(b) } else { -1 });
}

/// Exercises the negated (`br_if` via `i32.eqz`) fusion: `(i32.eqz (i32.lt_s ..))` is fused into
/// an inverted compare-branch, again after spilling `A`. Confirms the reverted value survives the
/// negated path too.
#[test]
#[cfg_attr(not(feature = "wat"), ignore)]
fn unspill_negated_branch() {
    let wat = r#"
    (module
        (func (export "test") (param i32 i32 i32 i32) (result i32)
            (block (result i32)
                (i32.and (local.get 0) (local.get 1))         ;; A = a & b, into `ireg`
                (i32.eqz (i32.lt_s (local.get 2) (local.get 3))) ;; cond = !(c < d), spills A
                (br_if 0)
                (drop)
                (i32.const -1)
            )
        )
    )"#;
    assert_matches_oracle(wat, |a, b, c, d| if c < d { -1 } else { a & b });
}
