(module
    (type $sig/0 (func))
    (type $sig/1 (func (param i64) (result i64)))
    (type $sig/8 (func
        (param i64 i64 i64 i64 i64 i64 i64 i64)
        (result i64 i64 i64 i64 i64 i64 i64 i64)
    ))
    (type $sig/16 (func
        (param
            i64 i64 i64 i64 i64 i64 i64 i64
            i64 i64 i64 i64 i64 i64 i64 i64
        )
        (result
            i64 i64 i64 i64 i64 i64 i64 i64
            i64 i64 i64 i64 i64 i64 i64 i64
        )
    ))

    (func $identity/0 (type $sig/0))
    (func $identity/1 (type $sig/1)
        (local.get 0)
    )
    (func $identity/8 (type $sig/8)
        (local.get 0) (local.get 1) (local.get 2) (local.get 3)
        (local.get 4) (local.get 5) (local.get 6) (local.get 7)
    )
    (func $identity/16 (type $sig/16)
        (local.get  0) (local.get  1) (local.get  2) (local.get  3)
        (local.get  4) (local.get  5) (local.get  6) (local.get  7)
        (local.get  8) (local.get  9) (local.get 10) (local.get 11)
        (local.get 12) (local.get 13) (local.get 14) (local.get 15)
    )

    ;; The callees are placed in the table at these indices:
    ;;   0 => $identity/0, 1 => $identity/1, 2 => $identity/8, 3 => $identity/16
    (table $t 4 4 funcref)
    (elem (i32.const 0) $identity/0 $identity/1 $identity/8 $identity/16)

    (func (export "run/0") (param $n i64) (result i64)
        (loop $continue
            (if
                (i64.eqz (local.get $n))
                (then
                    (return (i64.const 0))
                )
            )
            (call_indirect (type $sig/0) (i32.const 0))
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
            (drop (call_indirect (type $sig/1) (local.get $n) (i32.const 1)))
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
            (call_indirect (type $sig/8) ;; takes 8 parameters
                (local.get $n) (local.get $n)
                (local.get $n) (local.get $n)
                (local.get $n) (local.get $n)
                (local.get $n) (local.get $n)
                (i32.const 2)
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
            (call_indirect (type $sig/16) ;; takes 16 parameters
                (local.get $n) (local.get $n) (local.get $n) (local.get $n)
                (local.get $n) (local.get $n) (local.get $n) (local.get $n)
                (local.get $n) (local.get $n) (local.get $n) (local.get $n)
                (local.get $n) (local.get $n) (local.get $n) (local.get $n)
                (i32.const 3)
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
