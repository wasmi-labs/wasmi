(assert_invalid
  (module
    (import "m0" "g0" (global (mut i32)))
  )
  "mutable global support is not enabled"
)

(assert_invalid
  (module
    (global $g0 (mut i32) (i32.const 0))
    (export "g0" (global $g0))
  )
  "mutable global support is not enabled"
)
