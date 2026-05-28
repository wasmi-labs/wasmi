;; Iterative fibonacci without return_call (for stitch compatibility)
(module
    (func $fib_iterative (export "fibonacci_iter") (param $N i64) (result i64)
        (local $n1 i64)
        (local $n2 i64)
        (local $tmp i64)
        (local $i i64)
        (if
            (i64.le_s (local.get $N) (i64.const 1))
            (then (return (local.get $N)))
        )
        (local.set $n1 (i64.const 1))
        (local.set $n2 (i64.const 1))
        (local.set $i (i64.const 2))
        (loop $continue
            (if
                (i64.lt_s (local.get $i) (local.get $N))
                (then
                    (local.set $tmp (i64.add (local.get $n1) (local.get $n2)))
                    (local.set $n1 (local.get $n2))
                    (local.set $n2 (local.get $tmp))
                    (local.set $i (i64.add (local.get $i) (i64.const 1)))
                    (br $continue)
                )
            )
        )
        (local.get $n2)
    )
)
