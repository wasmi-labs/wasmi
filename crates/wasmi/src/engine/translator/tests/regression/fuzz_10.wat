(module
  (func (param $a i32) (result i32)
    i32.const 20
    (if (param i32) (result i32)
      (local.get $a)
      (then
        drop
        i32.const 10
      )
    )
  )
)
