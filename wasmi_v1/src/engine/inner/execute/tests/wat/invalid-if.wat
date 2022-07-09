;; This module is expected to be invalid upon instantiation.
(module
    (func $type-else-value-unreached-select (result i32)
        (if (result i64) (i32.const 1)
            (then (select (unreachable) (unreachable) (unreachable)))
            (else (select (unreachable) (unreachable) (unreachable)))
        )
    )
)
