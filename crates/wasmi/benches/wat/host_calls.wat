;; The exported `run` function calls the imported `benchmark/host` function `n` times.
;; The `benchmark/host` function is supposed to be an identity function with arity 0.
;;
;; After successful execution the `run` function returns 0.
(module
    (import "benchmark" "host/0" (func $host/0))
    (import "benchmark" "host/1" (func $host/1 (param i64) (result i64)))
    (import "benchmark" "host/8" (func $host/8
        (param i64 i64 i64 i64 i64 i64 i64 i64)
        (result i64 i64 i64 i64 i64 i64 i64 i64)
    ))
    (import "benchmark" "host/16" (func $host/16
        (param
            i64 i64 i64 i64 i64 i64 i64 i64
            i64 i64 i64 i64 i64 i64 i64 i64
        )
        (result
            i64 i64 i64 i64 i64 i64 i64 i64
            i64 i64 i64 i64 i64 i64 i64 i64
        )
    ))

    (func (export "run/0") (param $n i64) (result i64)
        (loop $continue
            (if
                (i64.eqz (local.get $n))
                (then
                    (return (i64.const 0))
                )
            )
            (call $host/0)
            (local.set $n (i64.sub (local.get $n) (i64.const 1)))
            (br $continue)
        )
        (unreachable)
    )

    (func (export "run/1") (param $n i64) (result i64)
        (loop $continue
            (if
                (i64.eqz (local.get $n))
                (then
                    (return (i64.const 0))
                )
            )
            (drop (call $host/1 (local.get $n)))
            (local.set $n (i64.sub (local.get $n) (i64.const 1)))
            (br $continue)
        )
        (unreachable)
    )

    (func (export "run/8") (param $n i64) (result i64)
        (loop $continue
            (if
                (i64.eqz (local.get $n))
                (then
                    (return (i64.const 0))
                )
            )
            (call $host/8 ;; takes 8 parameters
                (local.get $n) (local.get $n)
                (local.get $n) (local.get $n)
                (local.get $n) (local.get $n)
                (local.get $n) (local.get $n)
            )
            ;; drop all return values from the host function call
            (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop)
            (local.set $n (i64.sub (local.get $n) (i64.const 1)))
            (br $continue)
        )
        (unreachable)
    )

    (func (export "run/16") (param $n i64) (result i64)
        (loop $continue
            (if
                (i64.eqz (local.get $n))
                (then
                    (return (i64.const 0))
                )
            )
            (call $host/16 ;; takes 16 parameters
                (local.get $n) (local.get $n) (local.get $n) (local.get $n)
                (local.get $n) (local.get $n) (local.get $n) (local.get $n)
                (local.get $n) (local.get $n) (local.get $n) (local.get $n)
                (local.get $n) (local.get $n) (local.get $n) (local.get $n)
            )
            ;; drop all return values from the host function call
            (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop)
            (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop)
            (local.set $n (i64.sub (local.get $n) (i64.const 1)))
            (br $continue)
        )
        (unreachable)
    )
)
