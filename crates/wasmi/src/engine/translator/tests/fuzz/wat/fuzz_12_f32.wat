(module
  (func (param f32)
    f32.const -nan:0x7fffff (;=NaN;)
    local.tee 0
    local.get 0
    local.get 0
    f32.le
    br_if 0
    unreachable
  )
  (func (param f32)
    f32.const -nan:0x7fffff (;=NaN;)
    local.tee 0
    local.get 0
    local.get 0
    f32.ge
    br_if 0
    unreachable
  )
)