(module
  (func (export "run") (param $n i32) (result i32)
    (local $i i32)
    (loop $continue
        (br_if
            $continue
            (i32.ne
                (local.tee $i
                    (i32.add
                        (local.get $i)
                        (i32.const 1)
                    )
                )
                (local.get $n)
            )
        )
    )
    (return (local.get $i))
  )
)
