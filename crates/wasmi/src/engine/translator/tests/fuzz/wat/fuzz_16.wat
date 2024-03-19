(module
  (func (export "")
    (local f64)
    i32.const 0x7FFFFFFF
    local.get 0
    i64.reinterpret_f64
    global.get 0
    local.tee 0
    global.set 0
    i64.store
    unreachable
  )
  (memory 0 10)
  (global (mut f64) f64.const 1.0)
)