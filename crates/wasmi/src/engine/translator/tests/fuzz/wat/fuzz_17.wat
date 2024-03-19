(module
  (func (export "") (result i32 f64 f64)
    (local i64 f32)
    i32.const -1
    local.get 0
    f32.const -1.0
    i64.const 2
    i64.extend8_s
    local.set 0
    local.set 1
    i64.store
    unreachable
  )
  (memory 10)
)