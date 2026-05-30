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
        (return_call $fib_tail_recursive
            (i64.sub (local.get $N) (i64.const 1))
            (local.get $b)
            (i64.add (local.get $a) (local.get $b))
        )
    )

    (func (export "fibonacci_tail") (param $N i64) (result i64)
        (return_call $fib_tail_recursive (local.get $N) (i64.const 0) (i64.const 1))
    )

    (func (export "fibonacci_iter") (param $n i64) (result i64)
        (local $a i64)
        (local $b i64)
        (local $i i64)
        (local.set $a (i64.const 0))
        (local.set $b (i64.const 1))
        (local.set $i (local.get $n))
        (block $break
            (br_if $break (i64.eqz (local.get $i)))
            (loop $continue
                (i64.add (local.get $a) (local.get $b))
                (local.set $a (local.get $b))
                (local.set $b)
                (local.set $i (i64.sub (local.get $i) (i64.const 1)))
                (br_if $continue (i64.ne (local.get $i) (i64.const 0)))
            )
        )
        (local.get $a)
    )
)
