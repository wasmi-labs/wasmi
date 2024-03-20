(module
  (func (param i32)
    local.get 0
    local.tee 0
    i32.const 2
    i32.and
    local.get 0
    i32.eqz
    local.tee 0
    i32.and
    br_if 0
    unreachable
  )
)