;; Wast tests that check if Wasmi does not replace results of previous instructions
;; as optimization in situations where this produces semantically incorrect code.

(module
    (func (export "block.conditional.overwrite") (param $condition i32) (param $lhs i32) (param $rhs i32) (result i32)
        (block $break (result i32)
            (drop
                (i32.const 0)
                (br_if $break (local.get $condition))
            )
            (i32.add
                (local.get $lhs)
                (local.get $rhs)
            )
        )
        ;; Wasmi must not fuse `i32.add`'s result because `block` has divergent control flow.
        (local.tee $lhs)
    )

    (func (export "loop.conditional.overwrite") (param $condition i32) (param $lhs i32) (param $rhs i32) (result i32)
        (local $temp i32)
        (i32.add
            (local.get $lhs)
            (local.get $rhs)
        )
        (loop $continue (param i32) (result i32)
            ;; Wasmi must not fuse `i32.add`'s result because `loop` header has divergent control flow.
            (local.tee $temp)
            (drop
                (i32.const 0)
                (local.get $condition)
                (local.set $condition (i32.const 0))
                (br_if $continue)
            )
        )
    )

    (func (export "if.conditional.overwrite") (param $condition i32) (param $lhs i32) (param $rhs i32) (result i32)
        (if $break (result i32) (local.get $condition)
            (then
                (i32.const 0)
            )
            (else
                (i32.add
                    (local.get $lhs)
                    (local.get $rhs)
                )
            )
        )
        ;; Wasmi must not fuse `i32.add`'s result because `if` has divergent control flow.
        (local.tee $lhs)
    )

    (func (export "if.conditional.overwrite.no-else") (param $condition i32) (param $lhs i32) (param $rhs i32) (result i32)
        (i32.const 0)
        (if $break (param i32) (result i32)
            (local.get $condition)
            (then
                (drop)
                (i32.add
                    (local.get $lhs)
                    (local.get $rhs)
                )
            )
        )
        ;; Wasmi must not fuse `i32.add`'s result because `if` has divergent control flow.
        (local.tee $lhs)
    )

    (func (export "if.conditional.overwrite.static-then") (param $condition i32) (param $lhs i32) (param $rhs i32) (result i32)
        (i32.const 0)
        (if $break (param i32) (result i32)
            (i32.const 1) ;; if condition: true
            (then
                (drop
                    (br_if $break (local.get $condition))
                )
                (i32.add
                    (local.get $lhs)
                    (local.get $rhs)
                )
            )
        )
        ;; Wasmi must not fuse `i32.add`'s result because `block` has divergent control flow.
        (local.tee $lhs)
    )

    (func (export "if.conditional.overwrite.static-else") (param $condition i32) (param $lhs i32) (param $rhs i32) (result i32)
        (i32.const 0)
        (if $break (param i32) (result i32)
            (i32.const 0) ;; if condition: false
            (then)
            (else
                (drop
                    (br_if $break (local.get $condition))
                )
                (i32.add
                    (local.get $lhs)
                    (local.get $rhs)
                )
            )
        )
        ;; Wasmi must not fuse `i32.add`'s result because `block` has divergent control flow.
        (local.tee $lhs)
    )
)

(assert_return
    (invoke "block.conditional.overwrite" (i32.const 1) (i32.const 1) (i32.const 2)) (i32.const 0)
)
(assert_return
    (invoke "block.conditional.overwrite" (i32.const 0) (i32.const 1) (i32.const 2)) (i32.const 3)
)

(assert_return
    (invoke "loop.conditional.overwrite" (i32.const 1) (i32.const 1) (i32.const 2)) (i32.const 0)
)
(assert_return
    (invoke "loop.conditional.overwrite" (i32.const 0) (i32.const 1) (i32.const 2)) (i32.const 3)
)

(assert_return
    (invoke "if.conditional.overwrite" (i32.const 1) (i32.const 1) (i32.const 2)) (i32.const 0)
)
(assert_return
    (invoke "if.conditional.overwrite" (i32.const 0) (i32.const 1) (i32.const 2)) (i32.const 3)
)

(assert_return
    (invoke "if.conditional.overwrite.no-else" (i32.const 1) (i32.const 1) (i32.const 2)) (i32.const 3)
)
(assert_return
    (invoke "if.conditional.overwrite.no-else" (i32.const 0) (i32.const 1) (i32.const 2)) (i32.const 0)
)

(assert_return
    (invoke "if.conditional.overwrite.static-then" (i32.const 1) (i32.const 1) (i32.const 2)) (i32.const 0)
)
(assert_return
    (invoke "if.conditional.overwrite.static-then" (i32.const 0) (i32.const 1) (i32.const 2)) (i32.const 3)
)

(assert_return
    (invoke "if.conditional.overwrite.static-else" (i32.const 1) (i32.const 1) (i32.const 2)) (i32.const 0)
)
(assert_return
    (invoke "if.conditional.overwrite.static-else" (i32.const 0) (i32.const 1) (i32.const 2)) (i32.const 3)
)
