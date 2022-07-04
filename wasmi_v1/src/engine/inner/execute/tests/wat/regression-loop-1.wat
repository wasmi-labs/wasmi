;; Regression test testing returning values from a loop block.
(module
  (func (export "func") (result i32)
    (loop (result i32) (i32.const 7))
  )
)
