(module
    (func $f/1 (param i64)
        (br_if 0 (i64.eqz (local.get 0)))
        (local.set 0 (i64.sub (local.get 0) (i64.const 1)))
        (call $f/1 (local.get 0))
    )

    (func $f/8 (param i64 i64 i64 i64 i64 i64 i64 i64)
        (br_if 0 (i64.eqz (local.get 0)))
        (local.set 0 (i64.sub (local.get 0) (i64.const 1)))
        (call $f/8
            (local.get 0) (local.get 1) (local.get 2) (local.get 3)
            (local.get 4) (local.get 5) (local.get 6) (local.get 7)
        )
    )

    (func $f/16
        (param
            i64 i64 i64 i64
            i64 i64 i64 i64
            i64 i64 i64 i64
            i64 i64 i64 i64
        )
        (br_if 0 (i64.eqz (local.get 0)))
        (local.set 0 (i64.sub (local.get 0) (i64.const 1)))
        (call $f/16
            (local.get  0) (local.get  1) (local.get  2) (local.get  3)
            (local.get  4) (local.get  5) (local.get  6) (local.get  7)
            (local.get  8) (local.get  9) (local.get 10) (local.get 11)
            (local.get 12) (local.get 13) (local.get 14) (local.get 15)
        )
    )

    (func (export "run/1") (param $n i64) (result i64)
        (call $f/1 (local.get $n))
        (i64.const 0)
    )

    (func (export "run/8") (param $n i64) (result i64)
        (call $f/8
            (local.get $n) (local.get $n) (local.get $n) (local.get $n)
            (local.get $n) (local.get $n) (local.get $n) (local.get $n)
        )
        (i64.const 0)
    )

    (func (export "run/16") (param $n i64) (result i64)
        (call $f/16
            (local.get $n) (local.get $n) (local.get $n) (local.get $n)
            (local.get $n) (local.get $n) (local.get $n) (local.get $n)
            (local.get $n) (local.get $n) (local.get $n) (local.get $n)
            (local.get $n) (local.get $n) (local.get $n) (local.get $n)
        )
        (i64.const 0)
    )
)
