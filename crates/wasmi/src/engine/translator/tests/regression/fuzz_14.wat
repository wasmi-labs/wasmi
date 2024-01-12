(module
  (func (param i32 i32) (result i32 i32)
    local.get 0
    local.get 1
    i32.and
    global.get 0
    i32.eqz
  )
  (global i32 (i32.const -2))
)