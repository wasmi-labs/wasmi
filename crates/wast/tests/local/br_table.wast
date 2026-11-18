(module
    ;; Regression tests for `br_table` with equal-height targets whose
    ;; branch values sit above an operand that the branch drops.
    ;;
    ;; `$clobber` writes the caller's temporary slots so that a missing copy
    ;; reads a known wrong value instead of whatever the stack held before.

    (func $clobber (param i64 i64 i64))

    (func (export "br_table.dropped-operand.i64") (param $i i32) (result i64 i64)
        (call $clobber (i64.const -1) (i64.const -1) (i64.const -1))
        (block (result i64 i64)
            (i32.const 7)
            (i64.const 0x1111)
            (i64.const 0x2222)
            (br_table 0 1 0 (local.get $i))
        )
    )

    (func (export "br_table.dropped-operand.i64.reg-index") (param $i i32) (result i64 i64)
        (call $clobber (i64.const -1) (i64.const -1) (i64.const -1))
        (block (result i64 i64)
            (i32.const 7)
            (i64.const 0x3333)
            (i64.const 0x4444)
            (br_table 0 1 0 (i32.add (local.get $i) (i32.const 0)))
        )
    )

    (func (export "br_table.dropped-operand.v128") (param $i i32) (result v128)
        (call $clobber (i64.const -1) (i64.const -1) (i64.const -1))
        (block (result v128)
            (i32.const 7)
            (v128.const i64x2 0x5555 0x6666)
            (br_table 0 1 0 (local.get $i))
        )
    )
)

(assert_return (invoke "br_table.dropped-operand.i64" (i32.const 0)) (i64.const 0x1111) (i64.const 0x2222))
(assert_return (invoke "br_table.dropped-operand.i64" (i32.const 1)) (i64.const 0x1111) (i64.const 0x2222))
(assert_return (invoke "br_table.dropped-operand.i64" (i32.const 2)) (i64.const 0x1111) (i64.const 0x2222))
(assert_return (invoke "br_table.dropped-operand.i64.reg-index" (i32.const 0)) (i64.const 0x3333) (i64.const 0x4444))
(assert_return (invoke "br_table.dropped-operand.i64.reg-index" (i32.const 1)) (i64.const 0x3333) (i64.const 0x4444))
(assert_return (invoke "br_table.dropped-operand.i64.reg-index" (i32.const 2)) (i64.const 0x3333) (i64.const 0x4444))
(assert_return (invoke "br_table.dropped-operand.v128" (i32.const 0)) (v128.const i64x2 0x5555 0x6666))
(assert_return (invoke "br_table.dropped-operand.v128" (i32.const 1)) (v128.const i64x2 0x5555 0x6666))
(assert_return (invoke "br_table.dropped-operand.v128" (i32.const 2)) (v128.const i64x2 0x5555 0x6666))
