;; Exports a function `add` that computes the sum of its two operands.
(module
    (func $add (export "add") (param $lhs i32) (param $rhs i32) (result i32)
        (i32.add
            (local.get $lhs)
            (local.get $rhs)
        )
    )
)
