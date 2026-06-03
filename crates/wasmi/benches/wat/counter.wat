(module
  (func (export "run") (param $n i32) (result i32)
    (loop $continue
        (br_if
            $continue
            (local.tee $n
                (i32.sub
                    (local.get $n)
                    (i32.const 1)
                )
            )
        )
    )
    (return (local.get $n))
  )
)
