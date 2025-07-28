(module
    (func (export "if.only-then.diverging")
        (if
            (i32.const 0) ;; false
            (then
                (br 0)
            )
        )
    )
)

(assert_return (invoke "if.only-then.diverging"))

(module
    ;; Regression tests for PR #838.

    (func (export "local-preserve.if-then-drop-replace") (param i32) (result i32)
        (i32.const 20)
        (if (param i32) (result i32)
            (local.get 0)
            (then
                (drop)
                (i32.const 10)
            )
        )
    )
)

(assert_return (invoke "local-preserve.if-then-drop-replace" (i32.const 0)) (i32.const 20))
(assert_return (invoke "local-preserve.if-then-drop-replace" (i32.const 1)) (i32.const 10))

(module
    ;; Regression test from PR #838.
    (func (export "if.apply-and-conditionally-drop") (param $condition i32) (param i32 i32) (result i32)
        (local $tmp i32)
        (local.get 1)
        (local.get 2)
        (local.set 2 (i32.const -1)) ;; overwrite locals in reverse order
        (local.set 1 (i32.const -2))
        (if (param i32 i32) (result i32)
            (local.get $condition)
            (then
                ;; return 2nd param
                (local.set $tmp)
                (drop)
                (local.get $tmp)
            )
            (else
                ;; return 1st param
                (drop)
            )
        )
    )
)

(assert_return
    (invoke "if.apply-and-conditionally-drop" (i32.const 0) (i32.const 10) (i32.const 20))
    (i32.const 10)
)
(assert_return
    (invoke "if.apply-and-conditionally-drop" (i32.const 1) (i32.const 10) (i32.const 20))
    (i32.const 20)
)
