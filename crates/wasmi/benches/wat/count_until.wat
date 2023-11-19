;; Exports a function `count_until` that takes an input `n`.
;; The exported function counts an integer `n` times and then returns 0.
(module
  (func (export "count_until") (param $limit i32) (result i32)
    (local $counter i32)
    (loop
        (br_if
            0
            (i32.ne
                (local.tee $counter
                    (i32.add
                        (local.get $counter)
                        (i32.const 1)
                    )
                )
                (local.get $limit)
            )
        )
    )
    (return (local.get $counter))
  )
)
