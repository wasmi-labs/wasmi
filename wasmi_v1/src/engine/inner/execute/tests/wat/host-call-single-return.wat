;; Exports a function `wasm` that calls a
;; host function and returns its return value.
(module
    (func $host (import "test" "host") (result i32))
    (func (export "wasm") (result i32)
        (call $host)
    )
)
