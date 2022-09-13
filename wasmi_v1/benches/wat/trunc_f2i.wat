(module
    (func (export "trunc_f2i") (param $n i32) (param $input32 f32) (param $input64 f64) (result)
        (local $i i32)
        (block $exit
            (if
                (i32.le_u
                    (local.get $n)
                    (i32.const 0)
                )
                (unreachable) ;; trap if $n <= 0
            )
            (local.set $i (local.get $n)) ;; i = n
            (loop $continue
                (drop
                    (i32.trunc_f32_s (local.get $input32)) ;; <- under test
                )
                (drop
                    (i32.trunc_f32_u (local.get $input32)) ;; <- under test
                )
                (drop
                    (i64.trunc_f32_s (local.get $input32)) ;; <- under test
                )
                (drop
                    (i64.trunc_f64_u (local.get $input64)) ;; <- under test
                )
                (drop
                    (i32.trunc_f64_s (local.get $input64)) ;; <- under test
                )
                (drop
                    (i32.trunc_f64_u (local.get $input64)) ;; <- under test
                )
                (drop
                    (i64.trunc_f64_s (local.get $input64)) ;; <- under test
                )
                (drop
                    (i64.trunc_f64_u (local.get $input64)) ;; <- under test
                )
                (local.set $i ;; i -= 1
                    (i32.sub (local.get $i) (i32.const 1))
                )
                (br_if $continue (local.get $i)) ;; continue if i != 0
            )
        )
    )
)
