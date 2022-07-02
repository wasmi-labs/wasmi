;; Exports a function `wasm` that calls a
;; host function and returns its return value.
(module
    (func $host (import "test" "host") (result i64) (result i64) (result i32))
    (func (export "wasm") (result i64)
        (select
            (call $host)
        )
    )
)
