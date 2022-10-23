(module
    (global $g0 i32 (i32.const 0)) ;; constant
    (global $g1 i32 (i32.const 1)) ;; constant
    (func (export "call") (param $limit i32) (result i32)
        (local $accumulator i32)
        (loop $continue
            (br_if
                $continue
                (i32.lt_s
                    (local.tee $accumulator
                        (i32.add
                            (local.get $accumulator)
                            (i32.add
                                (global.get $g0)
                                (global.get $g1)
                            )
                        )
                    )
                    (local.get $limit)
                )
            )
        )
        (return (local.get $accumulator))
    )
)
