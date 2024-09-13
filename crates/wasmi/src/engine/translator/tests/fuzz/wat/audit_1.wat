(module ;; hangs on main branch
    (func (export "") (result i32 i32 i32)
        (local i32 i32 i32)
        i32.const 0
        (block (result i32 i32 i32) ;; label = @1
            local.get 0
            local.get 1
            local.get 2
            (block
                ;; The next two instructions together cause an integer-overflow trap.
                f64.const 0x1.b1ddf4040cd22p+901
                i32.trunc_f64_u
                drop
            )
        )
        drop
    )
)
