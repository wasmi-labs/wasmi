(module
    (func (result i32 i32 i32 i32)
        i32.const 1
        i32.const 0
        i32.const 1
        i32.const 0
    )
    (func (export "") (result i32 i32 i32 i32)
        (block (result i32 i32 i32 i32) ;; label = @1
            i32.const 0
            call 0
            br_table 0 1 1
        )
    )
)
