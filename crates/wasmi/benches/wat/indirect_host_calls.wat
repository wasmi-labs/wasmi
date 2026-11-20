;; Same as `indirect_calls.wat` but the `call_indirect` callees are imported host functions.
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

    (import "benchmark" "host/0" (func $host/0 (type $sig/0)))
    (import "benchmark" "host/1" (func $host/1 (type $sig/1)))
    (import "benchmark" "host/8" (func $host/8 (type $sig/8)))
    (import "benchmark" "host/16" (func $host/16 (type $sig/16)))

    ;; The callees are placed in the table at these indices:
    ;;   0 => $host/0, 1 => $host/1, 2 => $host/8, 3 => $host/16
    (table $t 4 4 funcref)
    (elem (i32.const 0) $host/0 $host/1 $host/8 $host/16)

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

