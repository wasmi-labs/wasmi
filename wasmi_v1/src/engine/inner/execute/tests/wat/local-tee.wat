;; Regression test for `local.tee`.
;;
;; From Wasm Spec Test Suite: 'local-tee.wat/result'
(module
    (func (export "func") (param i64 f32 f64 i32 i32) (result f64)
        (local f32 i64 i64 f64)
        (f64.add
            (f64.convert_i64_u (local.tee 0 (i64.const 1)))
            (f64.add
                (f64.promote_f32 (local.tee 1 (f32.const 2)))
                (f64.add
                    (local.tee 2 (f64.const 3.3))
                    (f64.add
                        (f64.convert_i32_u (local.tee 3 (i32.const 4)))
                        (f64.add
                        (f64.convert_i32_s (local.tee 4 (i32.const 5)))
                            (f64.add
                                (f64.promote_f32 (local.tee 5 (f32.const 5.5)))
                                (f64.add
                                    (f64.convert_i64_u (local.tee 6 (i64.const 6)))
                                    (f64.add
                                        (f64.convert_i64_u (local.tee 7 (i64.const 0)))
                                        (local.tee 8 (f64.const 8))
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    )
)
