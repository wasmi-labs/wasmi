;; Regression test testing returning values from a nested basic block.
(module
  (func (export "func") (result i32)
    (if (result i32) (i32.const 1)
        (then
            (i32.const 2)
        )
        (else
            (block (result i32)
                (i32.const 1)
            )
        )
    )
  )
)
