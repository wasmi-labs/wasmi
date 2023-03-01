(module
    (func $fib_recursive (export "fibonacci_rec") (param $N i64) (result i64)
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

    (func $fib_tail_recursive (param $N i64) (param $a i64) (param $b i64) (result i64)
        (if (i64.eqz (local.get $N))
            (then
                (return (local.get $a))
            )
        )
        (if (i64.eq (local.get $N) (i64.const 1))
            (then
                (return (local.get $b))
            )
        )
        (return_call $fib_tail_recursive
            (i64.sub (local.get $N) (i64.const 1))
            (local.get $b)
            (i64.add (local.get $a) (local.get $b))
        )
    )

    (func (export "fibonacci_tail") (param $N i64) (result i64)
        (return_call $fib_tail_recursive (local.get $N) (i64.const 0) (i64.const 1))
    )

    (func $fib_iterative (export "fibonacci_iter") (param $N i64) (result i64)
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
