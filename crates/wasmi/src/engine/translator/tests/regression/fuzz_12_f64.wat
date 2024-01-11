(module
  (func (param f64)
    f64.const -nan:0xfffffffffffff (;=NaN;)
    local.tee 0
    local.get 0
    local.get 0
    f64.le
    br_if 0
    unreachable
  )
  (func (param f64)
    f64.const -nan:0xfffffffffffff (;=NaN;)
    local.tee 0
    local.get 0
    local.get 0
    f64.ge
    br_if 0
    unreachable
  )
)