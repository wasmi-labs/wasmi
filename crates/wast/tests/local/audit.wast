;; Failing test cases found during the 2nd Wasmi audit.

(module
    (func $return-10-20-30-40 (result i32 i32 i32 i32)
        i32.const 10
        i32.const 20
        i32.const 30
        i32.const 40
    )
    ;; Audit found a function that returned an incorrect result.
    (func (export "audit.0") (result i32 i32 i32 i32)
        (block (result i32 i32 i32 i32)
            (i32.const 0)
            (call $return-10-20-30-40)
            (br_table 0 1 1)
        )
    )
)

(assert_return
    (invoke "audit.0")
    (i32.const 0) (i32.const 10) (i32.const 20) (i32.const 30)
)

(module
    ;; Audit found a hang on `main` branch instead of a trap.
    (func (export "audit.1") (result i32 i32)
        (local i32)
        (i32.const 0)
        (block (result i32)
            local.get 0
            (block
                ;; The next two instructions together cause an integer-overflow trap.
                (drop
                    (i32.trunc_f64_u (f64.const -inf))
                )
            )
        )
    )
)

(assert_trap
    (invoke "audit.1")
    "integer overflow"
)

(module
    ;; Audit found different results on Wasmi compared to Wasmtime.
    (func (export "audit.2") (param i32) (result i32 i32 i32 i32)
        (local.get 0)
        (local.get 0)
        (block (param i32 i32)
            local.tee 0
            (block (param i32 i32)
                (local.get 0)
                (local.get 0)
                (br 2) ;; returns
            )
        )
        (unreachable)
    )
)

(assert_return
    (invoke "audit.2" (i32.const 1))
    (i32.const 1) (i32.const 1) (i32.const 1) (i32.const 1)
)

(module
  (func (export "regression.0") (param i32) (result i32 i32 i32)
    local.get 0
    local.get 0
    local.get 0
    local.get 0
    br_if 0 ;; conditional return
  )
)

(assert_return
    (invoke "regression.0" (i32.const 0))
    (i32.const 0) (i32.const 0) (i32.const 0)
)
(assert_return
    (invoke "regression.0" (i32.const 1))
    (i32.const 1) (i32.const 1) (i32.const 1)
)
(assert_return
    (invoke "regression.0" (i32.const -1))
    (i32.const -1) (i32.const -1) (i32.const -1)
)

(module
  (func (export "regression.1") (param i64) (result f32)
    (block (result f32) ;; label = @1
      (block (result f32) ;; label = @2
        (block (result f32) ;; label = @3
          (f32.const 10)
          (local.get 0)
          (i32.wrap_i64)
          (br_table 0 3 0)
        )
        (unreachable)
      )
    )
    (unreachable)
  )
)

(assert_trap
    (invoke "regression.1" (i64.const 0))
    "unreachable"
)
(assert_return
    (invoke "regression.1" (i64.const 1))
    (f32.const 10.0)
)
(assert_trap
    (invoke "regression.1" (i64.const 2))
    "unreachable"
)
