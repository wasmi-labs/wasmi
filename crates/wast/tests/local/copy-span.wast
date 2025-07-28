(module
    (func (export "copy-many.local") (param $c i32) (param i32 i32 i32 i32) (result i32 i32 i32 i32)
        (block (result i32 i32 i32 i32)
            (local.get 1)
            (local.get 2)
            (local.get 3)
            (local.get 4)
            (br_if 0 (local.get $c)) ;; triggers a copy
        )
    )
)

(assert_return
    (invoke "copy-many.local" (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0))
    (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
)
(assert_return
    (invoke "copy-many.local" (i32.const 1) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0))
    (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
)
(assert_return
    (invoke "copy-many.local" (i32.const 0) (i32.const 1) (i32.const 2) (i32.const 3) (i32.const 4))
    (i32.const 1) (i32.const 2) (i32.const 3) (i32.const 4)
)
(assert_return
    (invoke "copy-many.local" (i32.const 1) (i32.const 1) (i32.const 2) (i32.const 3) (i32.const 4))
    (i32.const 1) (i32.const 2) (i32.const 3) (i32.const 4)
)

(module
    (func (export "copy-many.temp") (param $c i32) (param i64 i64 i64 i64) (result i32 i32 i32 i32)
        (block (result i32 i32 i32 i32)
            (i32.wrap_i64 (local.get 1))
            (i32.wrap_i64 (local.get 2))
            (i32.wrap_i64 (local.get 3))
            (i32.wrap_i64 (local.get 4))
            (br_if 0 (local.get $c)) ;; triggers a copy
        )
    )
)

(assert_return
    (invoke "copy-many.temp" (i32.const 0) (i64.const 0) (i64.const 0) (i64.const 0) (i64.const 0))
    (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
)
(assert_return
    (invoke "copy-many.temp" (i32.const 1) (i64.const 0) (i64.const 0) (i64.const 0) (i64.const 0))
    (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
)
(assert_return
    (invoke "copy-many.temp" (i32.const 0) (i64.const 1) (i64.const 2) (i64.const 3) (i64.const 4))
    (i32.const 1) (i32.const 2) (i32.const 3) (i32.const 4)
)
(assert_return
    (invoke "copy-many.temp" (i32.const 1) (i64.const 1) (i64.const 2) (i64.const 3) (i64.const 4))
    (i32.const 1) (i32.const 2) (i32.const 3) (i32.const 4)
)

(module
    (func (export "copy-many.across-stack-spaces") (param $c i32) (param i64 i64) (result i32 i64 i64 i32)
        (block (result i32 i64 i64 i32)
            (i32.wrap_i64 (local.get 1))
            (local.get 1)
            (local.get 2)
            (i32.wrap_i64 (local.get 2))
            (br_if 0 (local.get $c)) ;; triggers a copy
        )
    )
)

(assert_return
    (invoke "copy-many.across-stack-spaces" (i32.const 0) (i64.const 0) (i64.const 0))
    (i32.const 0) (i64.const 0) (i64.const 0) (i32.const 0)
)
(assert_return
    (invoke "copy-many.across-stack-spaces" (i32.const 0) (i64.const 0) (i64.const 1))
    (i32.const 0) (i64.const 0) (i64.const 1) (i32.const 1)
)
(assert_return
    (invoke "copy-many.across-stack-spaces" (i32.const 0) (i64.const 1) (i64.const 0))
    (i32.const 1) (i64.const 1) (i64.const 0) (i32.const 0)
)
(assert_return
    (invoke "copy-many.across-stack-spaces" (i32.const 0) (i64.const 1) (i64.const 2))
    (i32.const 1) (i64.const 1) (i64.const 2) (i32.const 2)
)
(assert_return
    (invoke "copy-many.across-stack-spaces" (i32.const 1) (i64.const 0) (i64.const 0))
    (i32.const 0) (i64.const 0) (i64.const 0) (i32.const 0)
)
(assert_return
    (invoke "copy-many.across-stack-spaces" (i32.const 1) (i64.const 0) (i64.const 1))
    (i32.const 0) (i64.const 0) (i64.const 1) (i32.const 1)
)
(assert_return
    (invoke "copy-many.across-stack-spaces" (i32.const 1) (i64.const 1) (i64.const 0))
    (i32.const 1) (i64.const 1) (i64.const 0) (i32.const 0)
)
(assert_return
    (invoke "copy-many.across-stack-spaces" (i32.const 1) (i64.const 1) (i64.const 2))
    (i32.const 1) (i64.const 1) (i64.const 2) (i32.const 2)
)

(module
    (func (export "copy-many.across-stack-spaces.2") (param $c i32) (param i64 i64) (result i64 i32 i32 i64)
        (block (result i64 i32 i32 i64)
            (local.get 1)
            (i32.wrap_i64 (local.get 1))
            (i32.wrap_i64 (local.get 2))
            (local.get 2)
            (br_if 0 (local.get $c)) ;; triggers a copy
        )
    )
)

(assert_return
    (invoke "copy-many.across-stack-spaces.2" (i32.const 0) (i64.const 0) (i64.const 0))
    (i64.const 0) (i32.const 0) (i32.const 0) (i64.const 0)
)
(assert_return
    (invoke "copy-many.across-stack-spaces.2" (i32.const 0) (i64.const 0) (i64.const 1))
    (i64.const 0) (i32.const 0) (i32.const 1) (i64.const 1)
)
(assert_return
    (invoke "copy-many.across-stack-spaces.2" (i32.const 0) (i64.const 1) (i64.const 0))
    (i64.const 1) (i32.const 1) (i32.const 0) (i64.const 0)
)
(assert_return
    (invoke "copy-many.across-stack-spaces.2" (i32.const 0) (i64.const 1) (i64.const 2))
    (i64.const 1) (i32.const 1) (i32.const 2) (i64.const 2)
)
(assert_return
    (invoke "copy-many.across-stack-spaces.2" (i32.const 1) (i64.const 0) (i64.const 0))
    (i64.const 0) (i32.const 0) (i32.const 0) (i64.const 0)
)
(assert_return
    (invoke "copy-many.across-stack-spaces.2" (i32.const 1) (i64.const 0) (i64.const 1))
    (i64.const 0) (i32.const 0) (i32.const 1) (i64.const 1)
)
(assert_return
    (invoke "copy-many.across-stack-spaces.2" (i32.const 1) (i64.const 1) (i64.const 0))
    (i64.const 1) (i32.const 1) (i32.const 0) (i64.const 0)
)
(assert_return
    (invoke "copy-many.across-stack-spaces.2" (i32.const 1) (i64.const 1) (i64.const 2))
    (i64.const 1) (i32.const 1) (i32.const 2) (i64.const 2)
)
