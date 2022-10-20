(module
    (func (export "bare_call_0") (param) (result))
    (func (export "bare_call_1") (param i32) (result i32) local.get 0)
    (func (export "bare_call_4") (param i32 i64 f32 f64) (result i32 i64 f32 f64)
        local.get 0
        local.get 1
        local.get 2
        local.get 3
    )
    (func (export "bare_call_16")
        (param i32 i64 f32 f64 i32 i64 f32 f64 i32 i64 f32 f64 i32 i64 f32 f64)
        (result i32 i64 f32 f64 i32 i64 f32 f64 i32 i64 f32 f64 i32 i64 f32 f64)
        local.get 0
        local.get 1
        local.get 2
        local.get 3
        local.get 4
        local.get 5
        local.get 6
        local.get 7
        local.get 8
        local.get 9
        local.get 10
        local.get 11
        local.get 12
        local.get 13
        local.get 14
        local.get 15
    )
)
