(module
    (func (export "local.tee.same") (param i32) (result i32)
        (local.tee 0 (local.get 0))
    )
    (func (export "local.tee.same.drop") (param i32)
        (drop (local.tee 0 (local.get 0)))
    )
)

(assert_return
    (invoke "local.tee.same" (i32.const 0)) (i32.const 0)
)
(assert_return
    (invoke "local.tee.same" (i32.const 1)) (i32.const 1)
)

(assert_return
    (invoke "local.tee.same.drop" (i32.const 0))
)
(assert_return
    (invoke "local.tee.same.drop" (i32.const 1))
)
