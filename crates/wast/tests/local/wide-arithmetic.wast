(module
    (func (export "i64.mul_wide_s(x,0)") (param $x i64) (result i64 i64)
        (i64.mul_wide_s (local.get $x) (i64.const 0))
    )
    (func (export "i64.mul_wide_s(0,x)") (param $x i64) (result i64 i64)
        (i64.mul_wide_s (i64.const 0) (local.get $x))
    )

    (func (export "i64.mul_wide_s(x,1)") (param $x i64) (result i64 i64)
        (i64.mul_wide_s (local.get $x) (i64.const 1))
    )
    (func (export "i64.mul_wide_s(1,x)") (param $x i64) (result i64 i64)
        (i64.mul_wide_s (i64.const 1) (local.get $x))
    )
)

(assert_return
    (invoke "i64.mul_wide_s(x,1)" (i64.const 0))
    (i64.const 0) (i64.const 0)
)
(assert_return
    (invoke "i64.mul_wide_s(x,1)" (i64.const 1))
    (i64.const 1) (i64.const 0)
)
(assert_return
    (invoke "i64.mul_wide_s(x,1)" (i64.const -1))
    (i64.const -1) (i64.const -1)
)

(assert_return
    (invoke "i64.mul_wide_s(1,x)" (i64.const 0))
    (i64.const 0) (i64.const 0)
)
(assert_return
    (invoke "i64.mul_wide_s(1,x)" (i64.const 1))
    (i64.const 1) (i64.const 0)
)
(assert_return
    (invoke "i64.mul_wide_s(1,x)" (i64.const -1))
    (i64.const -1) (i64.const -1)
)
