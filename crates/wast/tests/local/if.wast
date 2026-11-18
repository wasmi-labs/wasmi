(module
    ;; Regression tests for stale local register links on the `else` path of
    ;; an `if` with params where the `then` block writes a local.

    (func (export "if.params.then-local-set") (param $a i32) (param $b i32) (result i32)
        (local.get $a)
        (local.get $b)
        (if (param i32) (result i32)
            (then
                (local.set $a (i32.add (local.get $b) (i32.const 100)))
            )
        )
    )

    (func (export "if.params.then-local-tee") (param $a f64) (param $b i32) (result f64)
        (local.get $a)
        (local.get $b)
        (if (param f64) (result f64)
            (then
                (drop (local.tee $a (f64.add (local.get $a) (f64.const 100))))
            )
        )
    )

    (func (export "if.params.reg-temp.then-local-set") (param $a i32) (param $b i32) (result i32)
        (i32.add (local.get $a) (i32.const 1))
        (local.get $b)
        (if (param i32) (result i32)
            (then
                (local.set $a (i32.add (local.get $b) (i32.const 100)))
            )
        )
    )

    (func (export "if.params.reg-temp.then-local-set.else") (param $a i32) (param $b i32) (result i32)
        (i32.add (local.get $a) (i32.const 1))
        (local.get $b)
        (if (param i32) (result i32)
            (then
                (local.set $a (i32.add (local.get $b) (i32.const 100)))
            )
            (else)
        )
    )
)

(assert_return (invoke "if.params.then-local-set" (i32.const 42) (i32.const 0)) (i32.const 42))
(assert_return (invoke "if.params.then-local-set" (i32.const 7) (i32.const 1)) (i32.const 7))
(assert_return (invoke "if.params.then-local-tee" (f64.const 42) (i32.const 0)) (f64.const 42))
(assert_return (invoke "if.params.then-local-tee" (f64.const 7) (i32.const 1)) (f64.const 7))
(assert_return (invoke "if.params.reg-temp.then-local-set" (i32.const 42) (i32.const 0)) (i32.const 43))
(assert_return (invoke "if.params.reg-temp.then-local-set" (i32.const 7) (i32.const 1)) (i32.const 8))
(assert_return (invoke "if.params.reg-temp.then-local-set.else" (i32.const 42) (i32.const 0)) (i32.const 43))
(assert_return (invoke "if.params.reg-temp.then-local-set.else" (i32.const 7) (i32.const 1)) (i32.const 8))
