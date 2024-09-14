(module ;; different result on main than on Wasmtime
  (func (export "") (param i32) (result i32 i32 i32 i32)
    local.get 0
    local.get 0
    block (param i32 i32)
      local.tee 0
      block (param i32 i32)
        local.get 0
        local.get 0
        br 2 ;; returns
      end
    end
    unreachable
  )
)
