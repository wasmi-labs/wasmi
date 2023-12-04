(module
  (func (;0;) (param f64) (result i64)
    local.get 0
    f64.const 13
    local.set 0
    i64.reinterpret_f64
  )
  (export "" (func 0))
)
