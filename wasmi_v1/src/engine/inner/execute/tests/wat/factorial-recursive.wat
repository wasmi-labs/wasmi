;; Trivial recursive factorial function.
(func $fac (export "factorial_rec") (param $input i64) (result i64)
    (if (result i64) (i64.eq (local.get $input) (i64.const 0))
        (i64.const 1)
        (else
            (i64.mul
                (local.get $input)
                (call $fac
                    (i64.sub (local.get $input) (i64.const 1))
                )
            )
        )
    )
)
