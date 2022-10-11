;; Exports a function `sum` that returns the sum of the linear memory
;; contents until the given `limit`.
(module
    (memory (export "mem") 1)
    (func (export "sum_bytes") (param $limit i32) (result i64)
        (local $n i32)
        (local $sum i64)
        (block $exit
            (loop $loop
                (br_if ;; exit loop if $n == $limit
                    $exit
                    (i32.eq
                        (local.get $n)
                        (local.get $limit)
                    )
                )
                (local.set $sum ;; load n-th value from memory and add to sum
                    (i64.add
                        (local.get $sum)
                        (i64.load8_u offset=0 (local.get $n))
                    )
                )
                (local.set $n ;; increment n
                    (i32.add (local.get $n) (i32.const 1))
                 )
                (br $loop) ;; continue loop
            )
        )
        (return (local.get $sum))
    )
)
