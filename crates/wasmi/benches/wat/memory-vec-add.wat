;; Exports a function `vec_add` that computes the addition of 2 vectors
;; of length `len` starting at `ptr_a` and `ptr_b` and stores the result
;; into a buffer of the same length starting at `ptr_result`.
(module
    (memory (export "mem") 1)
    (func (export "vec_add")
        (param $ptr_result i32)
        (param $ptr_a i32)
        (param $ptr_b i32)
        (param $len i32)
        (local $n i32)
        (block $exit
            (loop $loop
                (br_if ;; exit loop if $n == $len
                    $exit
                    (i32.eq
                        (local.get $n)
                        (local.get $len)
                    )
                )
                (i64.store offset=0 ;; ptr_result[n] = ptr_a[n] + ptr_b[n]
                    (i32.add
                        (local.get $ptr_result)
                        (i32.mul
                            (local.get $n)
                            (i32.const 8)
                        )
                    )
                    (i64.add
                        (i64.load32_s offset=0 ;; load ptr_a[n]
                            (i32.add
                                (local.get $ptr_a)
                                (i32.mul
                                    (local.get $n)
                                    (i32.const 4)
                                )
                            )
                        )
                        (i64.load32_s offset=0 ;; load ptr_b[n]
                            (i32.add
                                (local.get $ptr_b)
                                (i32.mul
                                    (local.get $n)
                                    (i32.const 4)
                                )
                            )
                        )
                    )
                )
                (local.set $n ;; increment n
                    (i32.add (local.get $n) (i32.const 1))
                 )
                (br $loop) ;; continue loop
            )
        )
        (return)
    )
)
