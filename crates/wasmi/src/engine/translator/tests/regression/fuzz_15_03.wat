(module
  (func (;0;) (param i64) (result i32)
    block (result i32 i32) ;; label = @1
      global.get 0
      block (result i32 i32) ;; label = @2
        global.get 0
        block (result i32 i32) ;; label = @3
          i32.const 10
          i32.const 20
          local.get 0
          i32.wrap_i64
          br_table 0 (;@3;) 1 (;@2;) 0 (;@3;) 2 (;@1;)
        end
        i32.add
        ;; drop
        return
      end
      i32.mul
      ;; drop
      return
    end
    i32.xor
    return
  )
  (global (mut i32) (i32.const 30))
  (export "" (func 0))
)