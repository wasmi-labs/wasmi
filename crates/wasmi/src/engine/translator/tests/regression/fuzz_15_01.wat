(module
  (func (;0;) (param i64) (result f32)
    block (result f32) ;; label = @1
      block (result f32) ;; label = @2
        block (result f32) ;; label = @3
          f32.const 10
          local.get 0
          i32.wrap_i64
          br_table 0 (;@3;) 3 (;@0;) 0 (;@3;)
        end
        unreachable
      end
    end
    unreachable
  )
  (export "" (func 0))
)