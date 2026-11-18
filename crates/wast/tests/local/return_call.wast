(module
    ;; Regression test: tail calling a host function without params and results
    ;; must resume the caller's caller with a valid stack pointer, even if its
    ;; frame starts where the tail calling function's frame starts.

    (import "spectest" "print" (func $host))

    (func $tail_call_host
        (return_call $host)
    )

    (func $forty_two (result i32)
        (i32.const 42)
    )

    (func (export "return_call.host.no-params-no-results") (result i32)
        (call $tail_call_host)
        (call $forty_two)
    )
)

(assert_return (invoke "return_call.host.no-params-no-results") (i32.const 42))
