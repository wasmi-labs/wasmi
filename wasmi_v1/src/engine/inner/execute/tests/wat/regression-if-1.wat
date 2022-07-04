;; Regression test testing returning values from a loop block.
(module
    (func (export "func") (param i32) (result i32)
        (if (result i32) (local.get 0)
            (then (i32.const 7))
            (else (i32.const 8))
        )
    )
)
