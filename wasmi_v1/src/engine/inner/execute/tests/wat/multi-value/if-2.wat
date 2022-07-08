;; Regression test for `if` writing back block results for
;; empty `then` block and non-existing `else` block.
;;
;; From Wasm Spec Test Suite: 'multi-value/if.wat/params-id'
(module
  (func (export "func") (param i32) (result i32)
    (i32.const 1)
    (i32.const 2)
    (if (param i32 i32) (result i32 i32) (local.get 0) (then))
    (i32.add)
  )
)
