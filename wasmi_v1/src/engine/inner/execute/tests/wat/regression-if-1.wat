(module
    (func (export "func") (param i32) (result i32)
        ;; (if (local.get 0) (then (nop)))
        ;; (if (local.get 0) (then (nop)) (else (nop)))
        (if (result i32) (local.get 0) (then (i32.const 7)) (else (i32.const 8)))
    )
)
