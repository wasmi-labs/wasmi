(assert_invalid
  (module
    (func (param i32) (result i32)
      local.get 0
      i32.extend8_s
    )
  )
  "sign extension operations support is not enabled"
)

(assert_invalid
  (module
    (func (param i32) (result i32)
      local.get 0
      i32.extend16_s
    )
  )
  "sign extension operations support is not enabled"
)

(assert_invalid
  (module
    (func (param i64) (result i64)
      local.get 0
      i64.extend8_s
    )
  )
  "sign extension operations support is not enabled"
)

(assert_invalid
  (module
    (func (param i64) (result i64)
      local.get 0
      i64.extend16_s
    )
  )
  "sign extension operations support is not enabled"
)

(assert_invalid
  (module
    (func (param i64) (result i64)
      local.get 0
      i64.extend32_s
    )
  )
  "sign extension operations support is not enabled"
)
