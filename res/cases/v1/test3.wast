(module
  (type (;0;) (func (param i32) (result i32)))
  (global (;0;) (mut i32) (i32.const 0))
  (func (;55;) (type 0) (param i32) (result i32)
    (local i32)
    block i32  ;; label = @1
        get_global 0
        set_local 1
        get_global 0
        get_local 0
        i32.add
        set_global 0
        get_global 0
        i32.const 15
        i32.add
        i32.const -16
        i32.and
        set_global 0
        get_local 1
    end)
)
