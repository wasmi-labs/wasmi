(module
  (func (;0;) (param f32) (result i32)
    local.get 0
    f32.const 13
    local.set 0
    i32.reinterpret_f32
  )
  (export "" (func 0))
)
