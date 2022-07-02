;; Exports a function `linear_integral` that takes an input `n`.
;; It recursively calls itself with decreasing `n` and summing
;; up the chain of `n` values.
;; Therefore the exported function calls itself `n` times.
;;
;; Basically this function describes: f(n) := (n²+n)/2
(module
    (func $func (export "func") (param $n i32) (result i32)
        (if (result i32)
            (i32.eq (local.get $n) (i32.const 0))
            (then
                ;; return 0 if $n == 0
                (i32.const 0)
            )
            (else
                ;; return $n + (call $func($n - 1)) otherwise
                (i32.add
                    (call $func
                        (i32.sub
                            (local.get $n)
                            (i32.const 1)
                        )
                    )
                    (local.get $n)
                )
            )
        )
    )
)
