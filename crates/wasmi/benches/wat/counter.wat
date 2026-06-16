(module
    (func (export "count-via-locals") (param $n i32) (result i32)
        (loop $continue
            (br_if
                $continue
                (local.tee $n
                    (i32.sub
                        (local.get $n)
                        (i32.const 1)
                    )
                )
            )
        )
        (return (local.get $n))
    )

    (func (export "count-via-params") (param $n i32) (result i32)
        (local.get $n)
        (loop $continue (param i32) (result i32)
            (i32.const 1)
            (i32.sub)
            (local.tee $n)
            (local.get $n)
            (br_if $continue)
        )
    )
)
