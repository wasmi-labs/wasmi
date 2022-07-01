(module
  (func $fib_recursive (export "fibonacci_recursive") (param $N i32) (result i32)
    (if
      (i32.eq (local.get $N) (i32.const 0))
      (then (return (i32.const 0)))
    )
    (if
      (i32.eq (local.get $N) (i32.const 1))
      (then (return (i32.const 1)))
    )
    (return
      (i32.add
        (call $fib_recursive
          (i32.sub (local.get $N) (i32.const 1))
        )
        (call $fib_recursive
          (i32.sub (local.get $N) (i32.const 2))
        )
      )
    )
  )
)
