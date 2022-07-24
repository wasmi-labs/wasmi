(module
    (func $fib_recursive (export "fib_recursive") (param $N i64) (result i64)
        (if
            (i64.le_s (local.get $N) (i64.const 1))
            (then (return (local.get $N)))
        )
        (return
            (i64.add
                (call $fib_recursive
                  (i64.sub (local.get $N) (i64.const 1))
                )
                (call $fib_recursive
                  (i64.sub (local.get $N) (i64.const 2))
                )
            )
        )
    )

    (func $fib_iterative (export "fib_iterative") (param $N i64) (result i64)
        (local $n1 i64)
        (local $n2 i64)
        (local $tmp i64)
        (local $i i64)
        ;; return $N for N <= 1
        (if
            (i64.le_s (local.get $N) (i64.const 1))
            (then (return (local.get $N)))
        )
        (local.set $n1 (i64.const 1))
        (local.set $n2 (i64.const 1))
        (local.set $i (i64.const 2))
        ;;since we normally return n2, handle n=1 case specially
        (loop $again
            (if
                (i64.lt_s (local.get $i) (local.get $N))
                (then
                    (local.set $tmp (i64.add (local.get $n1) (local.get $n2)))
                    (local.set $n1 (local.get $n2))
                    (local.set $n2 (local.get $tmp))
                    (local.set $i (i64.add (local.get $i) (i64.const 1)))
                    (br $again)
                )
            )
        )
        (local.get $n2)
    )
)
