;; The below `.wat` file exports a function `call` that takes a `n` of type `i64`.
;; It will iterate `n` times and call the imported function `host_call` every time.
;;
;; This benchmarks tests the performance of host calls.
;;
;; After successful execution the `call` function will return `0`.
(module
    (import "benchmark" "host_call" (func $host_sub1 (param i64) (result i64)))
    (func $call (export "call") (param $n i64) (result i64)
        (if (result i64)
            (i64.eqz (local.get $n))
            (then
                ;; bail out early if n == 0
                (return (local.get $n))
            )
            (else
                (loop
                    ;; continue if n != 0, otherwise break out of loop
                    ;; the $host_call is expected to decrease its input by 1
                    (br_if 0
                        (i32.wrap_i64
                            (local.tee $n (call $host_sub1 (local.get $n)))
                        )
                    )
                )
                (return (local.get $n))
            )
        )
    )
)
