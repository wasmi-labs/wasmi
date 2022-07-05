;; Regression test for `drop` with unexpected empty providers.
;;
;; From Wasm Spec Test Suite: 'break-br_if-num'
(module
  (func (export "func") (param i32) (result i32)
    (drop
        (br_if 0
            (i32.const 50)
            (local.get 0)
        )
    )
    (i32.const 51)
  )
)
