(module
  (func $fib_recursive (param $N i32) (result i32)
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

  (func $fib_iterative (param $N i32) (result i32)
    (local $n1 i32)
    (local $n2 i32)
    (local $tmp i32)
    (local $i i32)
    (local.set $n1 (i32.const 1))
    (local.set $n2 (i32.const 1))
    (local.set $i (i32.const 2))
    ;; return 0 for N <= 0
    (if
      (i32.le_s (local.get $N) (i32.const 0))
      (then (return (i32.const 0)))
    )
    ;;since we normally return n2, handle n=1 case specially
    (if
      (i32.le_s (local.get $N) (i32.const 2))
      (then (return (i32.const 1)))
    )
    (loop $again
      (if
        (i32.lt_s (local.get $i) (local.get $N))
        (then
          (local.set $tmp (i32.add (local.get $n1) (local.get $n2)))
          (local.set $n1 (local.get $n2))
          (local.set $n2 (local.get $tmp))
          (local.set $i (i32.add (local.get $i) (i32.const 1)))
          (br $again)
        )
      )
    )
    (local.get $n2)
  )

  (export "fib_iterative" (func $fib_iterative))
  (export "fib_recursive" (func $fib_recursive))
)
