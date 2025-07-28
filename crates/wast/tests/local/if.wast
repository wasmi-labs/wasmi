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
