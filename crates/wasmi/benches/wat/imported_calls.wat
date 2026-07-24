;; Caller module for the `execute/call/imported` benchmark.
;;
;; It imports the `identity/{n}` Wasm functions from another module
;; (`imported_calls_provider.wat`) and calls them in a loop. Because the
;; callees are imported, the translator emits `call_imported` instead of
;; `call_internal`, exercising the cross-instance imported-Wasm call path.
(module
    (import "provider" "identity/0" (func $identity/0))
    (import "provider" "identity/1" (func $identity/1 (param i64) (result i64)))
    (import "provider" "identity/8" (func $identity/8
        (param i64 i64 i64 i64 i64 i64 i64 i64)
        (result i64 i64 i64 i64 i64 i64 i64 i64)
    ))
    (import "provider" "identity/16" (func $identity/16
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
