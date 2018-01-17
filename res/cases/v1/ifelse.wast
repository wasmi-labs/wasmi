(module
  (type (;0;) (func (result i32)))
  (func (;0;) (type 0) (result i32)
    (local i32)
    i32.const 0
    set_local 0
    i32.const 0
    if i32
        i32.const 5
    else
        i32.const 7
    end
    set_local 0
    get_local 0
    return)
)
