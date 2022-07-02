;; Exports a function `wasm` that calls a
;; host function and returns its return value.
(module
    (func $host (import "test" "host") (param i32) (param i32) (result i32))
    (func (export "wasm") (param $a i32) (param $b i32) (result i32)
        (call $host
            (local.get $a)
            (local.get $b)
        )
    )
)
