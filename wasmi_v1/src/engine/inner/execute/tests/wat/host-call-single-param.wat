;; Exports a function `wasm` that calls a
;; host function and returns its return value.
(module
    (func $host (import "test" "host") (param i32) (result i32))
    (func (export "wasm") (param $input i32) (result i32)
        (call $host (local.get $input))
    )
)
