//! Tests for the translator's loop-rotation optimization.
//!
//! Loop rotation turns `loop { if cond { body; br } }` into a one-time guard plus a
//! fused conditional back-edge, removing one branch per iteration. These tests assert
//! that the transform preserves observable behavior across the relevant loop shapes,
//! including shapes where it must *not* fire (fuel metering, register-operand guards).

use wasmi::{Config, Engine, Linker, Module, Store};

/// Instantiates `wat` (WebAssembly text) and returns the `run` function, typed.
fn setup(consume_fuel: bool, wat: &str) -> (Store<()>, wasmi::Func) {
    let mut config = Config::default();
    config.consume_fuel(consume_fuel);
    let engine = Engine::new(&config);
    let mut store = Store::new(&engine, ());
    if consume_fuel {
        store.set_fuel(u64::MAX).unwrap();
    }
    let module = Module::new(store.engine(), wat.as_bytes()).unwrap();
    let linker = Linker::new(&engine);
    let instance = linker.instantiate_and_start(&mut store, &module).unwrap();
    let func = instance.get_func(&store, "run").unwrap();
    (store, func)
}

/// `loop { if (i < N) { body; br } }` — the canonical rotatable shape (iterative Fibonacci).
const FIB: &str = r#"
(module
  (func (export "run") (param $N i64) (result i64)
    (local $n1 i64) (local $n2 i64) (local $tmp i64) (local $i i64)
    (if (i64.le_s (local.get $N) (i64.const 1)) (then (return (local.get $N))))
    (local.set $n1 (i64.const 1))
    (local.set $n2 (i64.const 1))
    (local.set $i (i64.const 2))
    (loop $continue
      (if (i64.lt_s (local.get $i) (local.get $N))
        (then
          (local.set $tmp (i64.add (local.get $n1) (local.get $n2)))
          (local.set $n1 (local.get $n2))
          (local.set $n2 (local.get $tmp))
          (local.set $i (i64.add (local.get $i) (i64.const 1)))
          (br $continue))))
    (local.get $n2)))
"#;

fn fib_ref(n: i64) -> i64 {
    if n <= 1 {
        return n;
    }
    let (mut a, mut b) = (1i64, 1i64);
    let mut i = 2;
    while i < n {
        let t = a.wrapping_add(b);
        a = b;
        b = t;
        i += 1;
    }
    b
}

#[test]
fn fib_loop_rotation_is_correct() {
    let (mut store, func) = setup(false, FIB);
    let run = func.typed::<i64, i64>(&store).unwrap();
    for n in [0i64, 1, 2, 3, 4, 10, 20, 50, 90] {
        assert_eq!(run.call(&mut store, n).unwrap(), fib_ref(n), "fib({n})");
    }
}

#[test]
fn fib_with_fuel_is_correct() {
    // With fuel metering enabled the loop must NOT be rotated, but results must match.
    let (mut store, func) = setup(true, FIB);
    let run = func.typed::<i64, i64>(&store).unwrap();
    for n in [0i64, 1, 2, 10, 50] {
        assert_eq!(run.call(&mut store, n).unwrap(), fib_ref(n), "fib({n}) +fuel");
    }
}

/// `loop { if (i < n) { body; br } tail }` — code after the `if` (the guard-miss / back-edge
/// fall-through must both reach `tail`).
const LOOP_WITH_TAIL: &str = r#"
(module
  (func (export "run") (param $n i32) (result i32)
    (local $i i32) (local $acc i32)
    (loop $L
      (if (i32.lt_s (local.get $i) (local.get $n))
        (then
          (local.set $acc (i32.add (local.get $acc) (local.get $i)))
          (local.set $i (i32.add (local.get $i) (i32.const 1)))
          (br $L)))
      ;; tail: executed exactly once, when the loop condition is false
      (local.set $acc (i32.add (local.get $acc) (i32.const 1000))))
    (local.get $acc)))
"#;

#[test]
fn loop_with_tail_is_correct() {
    let (mut store, func) = setup(false, LOOP_WITH_TAIL);
    let run = func.typed::<i32, i32>(&store).unwrap();
    for n in [0i32, 1, 2, 5, 100] {
        let expected = (0..n).sum::<i32>() + 1000;
        assert_eq!(run.call(&mut store, n).unwrap(), expected, "tail({n})");
    }
}

/// `loop { if ((i + 0) < n) { .. ; br } }` — the guard is preceded by an `i32.add`, so the
/// comparison is not the first instruction of the loop body and rotation must bail. Verifies
/// correctness of the unrotated path.
const NON_ROTATABLE: &str = r#"
(module
  (func (export "run") (param $n i32) (result i32)
    (local $i i32)
    (loop $L
      (if (i32.lt_s (i32.add (local.get $i) (i32.const 0)) (local.get $n))
        (then
          (local.set $i (i32.add (local.get $i) (i32.const 1)))
          (br $L))))
    (local.get $i)))
"#;

#[test]
fn non_rotatable_loop_is_correct() {
    let (mut store, func) = setup(false, NON_ROTATABLE);
    let run = func.typed::<i32, i32>(&store).unwrap();
    for n in [0i32, 1, 5, 1000] {
        assert_eq!(run.call(&mut store, n).unwrap(), n.max(0), "nonrot({n})");
    }
}

/// `block $exit { loop { br_if $exit cond ; body ; br } }` — the forward `br_if`-exit shape
/// (what real toolchains emit). Rotation must fire here and preserve behavior.
const BRIF_EXIT: &str = r#"
(module
  (func (export "run") (param $n i32) (result i32)
    (local $i i32) (local $acc i32)
    (block $exit
      (loop $L
        (br_if $exit (i32.eq (local.get $i) (local.get $n)))
        (local.set $acc (i32.add (local.get $acc) (local.get $i)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $L)))
    (local.get $acc)))
"#;

#[test]
fn brif_exit_loop_rotation_is_correct() {
    let (mut store, func) = setup(false, BRIF_EXIT);
    let run = func.typed::<i32, i32>(&store).unwrap();
    for n in [0i32, 1, 2, 5, 100, 1000] {
        let expected = (0..n).sum::<i32>();
        assert_eq!(run.call(&mut store, n).unwrap(), expected, "brif_exit({n})");
    }
}

#[test]
fn brif_exit_with_fuel_is_correct() {
    // Fuel metering on: must not rotate, results must still match.
    let (mut store, func) = setup(true, BRIF_EXIT);
    let run = func.typed::<i32, i32>(&store).unwrap();
    for n in [0i32, 1, 5, 100] {
        let expected = (0..n).sum::<i32>();
        assert_eq!(run.call(&mut store, n).unwrap(), expected, "brif_exit({n}) +fuel");
    }
}

/// `block $exit { loop { br_if $exit (i >= n) ; body ; br } }` — `>=` exit (negation is `<`),
/// exercising a different comparison through the rotation.
const BRIF_EXIT_GE: &str = r#"
(module
  (func (export "run") (param $n i32) (result i32)
    (local $i i32) (local $acc i32)
    (block $exit
      (loop $L
        (br_if $exit (i32.ge_s (local.get $i) (local.get $n)))
        (local.set $acc (i32.add (local.get $acc) (local.get $i)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $L)))
    (local.get $acc)))
"#;

#[test]
fn brif_exit_ge_loop_rotation_is_correct() {
    let (mut store, func) = setup(false, BRIF_EXIT_GE);
    let run = func.typed::<i32, i32>(&store).unwrap();
    for n in [0i32, 1, 2, 5, 100, 1000] {
        let expected = (0..n.max(0)).sum::<i32>();
        assert_eq!(run.call(&mut store, n).unwrap(), expected, "brif_exit_ge({n})");
    }
}

/// A `loop { br_if }` shape (no inner `if`) — already optimal, must be untouched and correct.
const COUNTER: &str = r#"
(module
  (func (export "run") (param $n i32) (result i32)
    (local $i i32)
    (loop $continue
      (br_if $continue
        (i32.ne (local.tee $i (i32.add (local.get $i) (i32.const 1))) (local.get $n))))
    (local.get $i)))
"#;

/// Memory loops: rotation is gated off (loads/stores make the loop memory-bound), but the
/// computation must remain correct. Stores `i` to `mem[i*4]`, then sums it back.
const MEMORY_LOOP: &str = r#"
(module
  (memory 1)
  (func (export "run") (param $n i32) (result i32)
    (local $i i32) (local $acc i32)
    (block $w (loop $L
      (br_if $w (i32.eq (local.get $i) (local.get $n)))
      (i32.store (i32.mul (local.get $i) (i32.const 4)) (local.get $i))
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br $L)))
    (local.set $i (i32.const 0))
    (block $r (loop $L2
      (br_if $r (i32.eq (local.get $i) (local.get $n)))
      (local.set $acc
        (i32.add (local.get $acc)
          (i32.load (i32.mul (local.get $i) (i32.const 4)))))
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br $L2)))
    (local.get $acc)))
"#;

#[test]
fn memory_loop_is_correct() {
    let (mut store, func) = setup(false, MEMORY_LOOP);
    let run = func.typed::<i32, i32>(&store).unwrap();
    for n in [0i32, 1, 2, 100, 256] {
        let expected = (0..n).sum::<i32>();
        assert_eq!(run.call(&mut store, n).unwrap(), expected, "mem_loop({n})");
    }
}

#[test]
fn counter_loop_is_correct() {
    let (mut store, func) = setup(false, COUNTER);
    let run = func.typed::<i32, i32>(&store).unwrap();
    for n in [1i32, 2, 5, 1000] {
        assert_eq!(run.call(&mut store, n).unwrap(), n, "counter({n})");
    }
}
