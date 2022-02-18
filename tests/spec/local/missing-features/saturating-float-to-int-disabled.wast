(assert_invalid
  (module
    (func (param f32) (result i32)
      local.get 0
      i32.trunc_sat_f32_s
    )
  )
  "saturating float to int conversions support is not enabled"
)

(assert_invalid
  (module
    (func (param f32) (result i32)
      local.get 0
      i32.trunc_sat_f32_u
    )
  )
  "saturating float to int conversions support is not enabled"
)

(assert_invalid
  (module
    (func (param f64) (result i32)
      local.get 0
      i32.trunc_sat_f64_s
    )
  )
  "saturating float to int conversions support is not enabled"
)

(assert_invalid
  (module
    (func (param f64) (result i32)
      local.get 0
      i32.trunc_sat_f64_u
    )
  )
  "saturating float to int conversions support is not enabled"
)

(assert_invalid
  (module
    (func (param f32) (result i64)
      local.get 0
      i64.trunc_sat_f32_s
    )
  )
  "saturating float to int conversions support is not enabled"
)

(assert_invalid
  (module
    (func (param f32) (result i64)
      local.get 0
      i64.trunc_sat_f32_u
    )
  )
  "saturating float to int conversions support is not enabled"
)

(assert_invalid
  (module
    (func (param f64) (result i64)
      local.get 0
      i64.trunc_sat_f64_s
    )
  )
  "saturating float to int conversions support is not enabled"
)

(assert_invalid
  (module
    (func (param f64) (result i64)
      local.get 0
      i64.trunc_sat_f64_u
    )
  )
  "saturating float to int conversions support is not enabled"
)
