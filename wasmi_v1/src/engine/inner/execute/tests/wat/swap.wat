;; Exports a function `swap` that returns its two operands swapped.
(module
  (func $swap (export "swap") (param $lhs i32) (param $rhs i32) (result i32) (result i32)
    local.get $rhs
    local.get $lhs
  )
)
