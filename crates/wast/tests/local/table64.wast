(module
    ;; The maximum does not fit into `usize` on 32-bit hosts. It only limits
    ;; growth, so instantiation and growing within system limits must work.
    (table i64 1 0x1_0000_0000 funcref)
    (func (export "grow") (result i64)
        (table.grow (ref.null func) (i64.const 1))
    )
    (func (export "size") (result i64)
        (table.size)
    )
)

(assert_return (invoke "grow") (i64.const 1))
(assert_return (invoke "size") (i64.const 2))
