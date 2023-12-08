(module
  (func (;0;) (param i32) (result i32 i32 i32)
    local.get 0
    call 0
    if (param i32) (result i32 i32) ;; label = @1
      call 0
      if ;; label = @2
      end
    else
      i32.const 0
    end
    unreachable
  )
)
