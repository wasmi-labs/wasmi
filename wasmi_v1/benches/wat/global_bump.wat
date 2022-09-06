;; Exports a function `bump` that takes an input `n`.
;; The exported function bumps a global variable `n` times and then returns it.
(module
    (global $g (mut i32) (i32.const 0))
    (func $bump (export "bump") (param $n i32) (result i32)
        (global.set $g (i32.const 0))
        (block $break
            (loop $continue
                (br_if ;; if $g == $n then break
                    $break
                    (i32.eq
                        (global.get $g)
                        (local.get $n)
                    )
                )
                (global.set $g ;; $g += 1
                    (i32.add
                        (global.get $g)
                        (i32.const 1)
                    )
                )
                (br $continue)
            )
        )
        (return (global.get $g))
    )
)
