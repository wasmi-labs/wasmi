;; Regression test for `if` with const eval block parameters.
;;
;; From Wasm Spec Test Suite: 'multi-value/if.wat/params'
(module
    (func (export "func") (param i32) (result i32)
        (i32.const 1)
        (if (param i32) (result i32) (local.get 0)
            (then (i32.const 2) (i32.add))
            (else (i32.const -2) (i32.add))
        )
    )
)
