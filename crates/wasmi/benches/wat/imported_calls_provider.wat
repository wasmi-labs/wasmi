;; Provider module for the `execute/call/imported` benchmark.
;;
;; It exports the `identity/{n}` Wasm functions that the caller module
;; (`imported_calls.wat`) imports and calls via `call_imported`.
(module
    (func (export "identity/0"))
    (func (export "identity/1") (param i64) (result i64)
        (local.get 0)
    )
    (func (export "identity/8")
        (param i64 i64 i64 i64 i64 i64 i64 i64)
        (result i64 i64 i64 i64 i64 i64 i64 i64)
        (local.get 0) (local.get 1) (local.get 2) (local.get 3)
        (local.get 4) (local.get 5) (local.get 6) (local.get 7)
    )
    (func (export "identity/16")
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
)
