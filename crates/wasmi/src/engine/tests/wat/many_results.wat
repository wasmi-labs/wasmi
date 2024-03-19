(module
    (func (export "test")
        (result
            ;; Define 150 i32 results
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
        )
        (call $return10) (call $return10) (call $return10) (call $return10) (call $return10)
        (call $return10) (call $return10) (call $return10) (call $return10) (call $return10)
        (call $return10) (call $return10) (call $return10) (call $return10) (call $return10)
    )

    (func $return10
        (result i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
        (i32.const 0) (i32.const 1) (i32.const 2) (i32.const 3) (i32.const 4)
        (i32.const 5) (i32.const 6) (i32.const 7) (i32.const 8) (i32.const 9)
    )
)
