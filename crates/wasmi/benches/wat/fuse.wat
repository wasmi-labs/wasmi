(module
  (func (export "test") (param $n i32) (result i32)
    (local $i i32)
    (loop $continue
        ;; i += 1
        (local.set $i
            (i32.add
                (local.get $i)
                (i32.const 1)
            )
        )
        ;; if not((i >= n) and (i <= n)) then continue
        ;; Note: The above is equal to:
        ;; if i != n then continue
        (br_if
            $continue
            (i32.eqz
                (i32.and
                    (i32.ge_u (local.get $i) (local.get $n))
                    (i32.le_u (local.get $i) (local.get $n))
                )
            )
        )
    )
    (return (local.get $i))
  )
)
