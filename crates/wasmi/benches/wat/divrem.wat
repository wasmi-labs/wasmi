(module
  (func (export "test") (param $n i32) (result i32)
    (local $m i64)
    (local $tmp32 i32)
    (local $tmp64 i64)
    (loop $continue
        ;; n -= 1
        (local.set $n
            (i32.sub
                (local.get $n)
                (i32.const 1)
            )
        )
        ;; m = n
        (local.set $m (i64.extend_i32_u (local.get $n)))
        ;; execute a bunch of div and rem instructions with immediate `rhs` values
        (local.set $tmp32 (i32.div_s (local.get $n) (i32.const 3)))
        (local.set $tmp32 (i32.div_u (local.get $n) (i32.const 3)))
        (local.set $tmp32 (i32.rem_s (local.get $n) (i32.const 3)))
        (local.set $tmp32 (i32.rem_u (local.get $n) (i32.const 3)))
        (local.set $tmp64 (i64.div_s (local.get $m) (i64.const 3)))
        (local.set $tmp64 (i64.div_u (local.get $m) (i64.const 3)))
        (local.set $tmp64 (i64.rem_s (local.get $m) (i64.const 3)))
        (local.set $tmp64 (i64.rem_u (local.get $m) (i64.const 3)))
        ;; continue if $n != 0
        (br_if $continue (local.get $n))
    )
    (return (local.get $n))
  )
)
