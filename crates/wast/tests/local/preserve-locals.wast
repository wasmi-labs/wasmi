(module
    (func (export "block.preserve-locals") (param $cond i32) (param $x i32) (result i32)
        (local.get $x)
        (block
            (local.get $cond)
            (br_if 0)
            (local.set $x (i32.const 1))
        )
    )

    (func (export "if.preserve-locals") (param $cond i32) (param $x i32) (result i32)
        (local.get $x)
        (if
            (i32.eqz (local.get $cond))
            (then
                (local.set $x (i32.const 1))
            )
        )
    )
)

;; block

(assert_return (invoke "block.preserve-locals" (i32.const 0) (i32.const 0)) (i32.const 0))
(assert_return (invoke "block.preserve-locals" (i32.const 0) (i32.const 1)) (i32.const 1))
(assert_return (invoke "block.preserve-locals" (i32.const 0) (i32.const -1)) (i32.const -1))
(assert_return (invoke "block.preserve-locals" (i32.const 0) (i32.const 42)) (i32.const 042))
(assert_return (invoke "block.preserve-locals" (i32.const 0) (i32.const 999)) (i32.const 999))
(assert_return (invoke "block.preserve-locals" (i32.const 1) (i32.const 0)) (i32.const 0))
(assert_return (invoke "block.preserve-locals" (i32.const 1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "block.preserve-locals" (i32.const 1) (i32.const -1)) (i32.const -1))
(assert_return (invoke "block.preserve-locals" (i32.const 1) (i32.const 42)) (i32.const 042))
(assert_return (invoke "block.preserve-locals" (i32.const 1) (i32.const 999)) (i32.const 999))

;; if

(assert_return (invoke "if.preserve-locals" (i32.const 0) (i32.const 0)) (i32.const 0))
(assert_return (invoke "if.preserve-locals" (i32.const 0) (i32.const 1)) (i32.const 1))
(assert_return (invoke "if.preserve-locals" (i32.const 0) (i32.const -1)) (i32.const -1))
(assert_return (invoke "if.preserve-locals" (i32.const 0) (i32.const 42)) (i32.const 042))
(assert_return (invoke "if.preserve-locals" (i32.const 0) (i32.const 999)) (i32.const 999))
(assert_return (invoke "if.preserve-locals" (i32.const 1) (i32.const 0)) (i32.const 0))
(assert_return (invoke "if.preserve-locals" (i32.const 1) (i32.const 1)) (i32.const 1))
(assert_return (invoke "if.preserve-locals" (i32.const 1) (i32.const -1)) (i32.const -1))
(assert_return (invoke "if.preserve-locals" (i32.const 1) (i32.const 42)) (i32.const 042))
(assert_return (invoke "if.preserve-locals" (i32.const 1) (i32.const 999)) (i32.const 999))
