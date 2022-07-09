;; Regression test for `drop` with unexpected empty providers.
;;
;; From Wasm Spec Test Suite: 'if.wat/break-value'
;;
;; Input: i32(0)
;; Expected: i32(21)
(module
    (func (export "func") (param i32) (result i32)
        (if (result i32) (local.get 0)
            (then (br 0 (i32.const 18)) (i32.const 19))
            (else (br 0 (i32.const 21)) (i32.const 20))
        )
    )
)
