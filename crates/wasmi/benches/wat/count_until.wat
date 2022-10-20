;; Exports a function `count_until` that takes an input `n`.
;; The exported function counts an integer `n` times and then returns 0.
(module
  (func (export "count_until") (param $limit i32) (result i32)
    (local $counter i32)
    (block
        (loop
            (br_if
                1
                (i32.eq
                    (local.tee $counter
                        (i32.add
                            (local.get $counter)
                            (i32.const 1)
                        )
                    )
                    (local.get $limit)
                )
            )
            (br 0)
        )
    )
    (return (local.get $counter))
  )
)
