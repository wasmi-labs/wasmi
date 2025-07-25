(module
    ;; Regression test asserting that `select` does not push twice in case of
    ;; an `i32.const 0` conditional and temporary operands (via `i32.wrap_i64`).
    (func (export "select.consteval.result") (param i64) (result f32)
        f32.const 0
        f32.const 0
        (drop
            (select
                (i32.wrap_i64 (local.get 0)) ;; case: true
                (i32.wrap_i64 (local.get 0)) ;; case: fase
                i32.const 0                  ;; condition (false)
            )
        )
        f32.add
    )
)

(assert_return
    (invoke "select.consteval.result" (i64.const 42))
    (f32.const 0)
)
