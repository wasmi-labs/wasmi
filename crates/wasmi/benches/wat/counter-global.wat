(module
    (global $count (mut i32) (i32.const 0))
    (func (export "count-via-globals") (param $n i32) (result i32)
        (global.set $count (local.get $n))
        (loop $continue
            (br_if
                $continue
                (global.set $count
                    (i32.sub
                        (global.get $count)
                        (i32.const 1)
                    )
                )
                (global.get $count)
            )
        )
        (return (global.get $count))
    )
)
