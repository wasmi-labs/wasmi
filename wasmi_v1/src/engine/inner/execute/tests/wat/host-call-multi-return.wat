;; Exports a function `wasm` that calls a host function.
;;
;; The host function returns 3 values:
;;
;; - `condition: i32`
;; - `a: i64`
;; - `b: i64`
;;
;; The wasm function either returns `a` or `b` if the `condition` is `true`
;; or `false` respectively.
(module
    (func $host (import "test" "host") (result i64) (result i64) (result i32))
    (func (export "wasm") (result i64)
        (select
            (call $host)
        )
    )
)
