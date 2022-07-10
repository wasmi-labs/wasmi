(module
    (func $add64_u_with_carry (export "add64_u_with_carry")
        (param $i i64) (param $j i64) (param $c i32) (result i64 i32)
        (local $k i64)
        (local.set $k
            (i64.add
            (i64.add (local.get $i) (local.get $j))
            (i64.extend_i32_u (local.get $c))
            )
        )
        (return (local.get $k) (i64.lt_u (local.get $k) (local.get $i)))
    )

    (func $add64_u_saturated (export "add64_u_saturated")
        (param i64 i64) (result i64)
        (call $add64_u_with_carry (local.get 0) (local.get 1) (i32.const 0))
        (if (param i64) (result i64)
            (then (drop) (i64.const -1))
        )
    )
)
