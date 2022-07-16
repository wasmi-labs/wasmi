;; Exports a function `call` that takes an input `n`.
;; The exported function calls itself `n` times and traps afterwards.
(module
    (func $call (export "call") (param $n i32) (result i32)
        (if (result i32)
            (local.get $n)
            (then
                (return
                    (call $call
                        (i32.sub
                            (local.get $n)
                            (i32.const 1)
                        )
                    )
                )
            )
            (else (unreachable))
        )
    )
)
