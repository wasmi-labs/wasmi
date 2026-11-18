(module
    ;; Regression tests for `loop` params clobbering the register that holds
    ;; the result of a preceding `local.tee`.

    (func (export "loop.params.local-tee.i64") (param $x i64) (param $y i32) (result i64)
        (local $l i64)
        (local.tee $l (i64.add (local.get $x) (i64.const 1)))
        (local.get $y)
        (loop (param i32) (drop))
    )

    (func (export "loop.params.local-tee.f64") (param $x f64) (param $y f64) (result f64)
        (local $l f64)
        (local.tee $l (f64.add (local.get $x) (f64.const 1)))
        (local.get $y)
        (loop (param f64) (drop))
    )
)

(assert_return (invoke "loop.params.local-tee.i64" (i64.const 41) (i32.const 7)) (i64.const 42))
(assert_return (invoke "loop.params.local-tee.i64" (i64.const 9) (i32.const 100)) (i64.const 10))
(assert_return (invoke "loop.params.local-tee.f64" (f64.const 41) (f64.const 7)) (f64.const 42))
(assert_return (invoke "loop.params.local-tee.f64" (f64.const 9) (f64.const 100)) (f64.const 10))
