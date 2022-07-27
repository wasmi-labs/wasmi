(module
    (func $is_even (export "is_even") (param $a i32) (result i32)
        (if (result i32)
            (i32.eqz (local.get $a))
            (then
                (i32.const 1)
            )
            (else
                (call $is_odd (i32.sub (local.get $a) (i32.const 1)))
            )
        )
    )
    (func $is_odd (param $a i32) (result i32)
        (if (result i32)
            (i32.eqz (local.get $a))
            (then
                (i32.const 0)
            )
            (else
                (call $is_even (i32.sub (local.get $a) (i32.const 1)))
            )
        )
    )
)
