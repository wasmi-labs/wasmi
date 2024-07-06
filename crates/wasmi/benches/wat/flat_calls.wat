(module
    (func $identity/0)
    (func $identity/1 (param i64) (result i64)
        (local.get 0)
    )
    (func $identity/8
        (param i64 i64 i64 i64 i64 i64 i64 i64)
        (result i64 i64 i64 i64 i64 i64 i64 i64)
        (local.get 0) (local.get 1) (local.get 2) (local.get 3)
        (local.get 4) (local.get 5) (local.get 6) (local.get 7)
    )
    (func $identity/16
        (param
            i64 i64 i64 i64 i64 i64 i64 i64
            i64 i64 i64 i64 i64 i64 i64 i64
        )
        (result
            i64 i64 i64 i64 i64 i64 i64 i64
            i64 i64 i64 i64 i64 i64 i64 i64
        )
        (local.get  0) (local.get  1) (local.get  2) (local.get  3)
        (local.get  4) (local.get  5) (local.get  6) (local.get  7)
        (local.get  8) (local.get  9) (local.get 10) (local.get 11)
        (local.get 12) (local.get 13) (local.get 14) (local.get 15)
    )

    (func (export "run/0") (param $n i64) (result i64)
        (loop $continue
            (if
                (i64.eqz (local.get $n))
                (then
                    (return (i64.const 0))
                )
            )
            (call $identity/0)
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
            (drop (call $identity/1 (local.get $n)))
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
            (call $identity/8 ;; takes 8 parameters
                (local.get $n) (local.get $n)
                (local.get $n) (local.get $n)
                (local.get $n) (local.get $n)
                (local.get $n) (local.get $n)
            )
            ;; drop all return values from the previous function call
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
            (call $identity/16 ;; takes 16 parameters
                (local.get $n) (local.get $n) (local.get $n) (local.get $n)
                (local.get $n) (local.get $n) (local.get $n) (local.get $n)
                (local.get $n) (local.get $n) (local.get $n) (local.get $n)
                (local.get $n) (local.get $n) (local.get $n) (local.get $n)
            )
            ;; drop all return values from the previous function call
            (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop)
            (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop)
            (local.set $n (i64.sub (local.get $n) (i64.const 1)))
            (br $continue)
        )
        (unreachable)
    )
)
