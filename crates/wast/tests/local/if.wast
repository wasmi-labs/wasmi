(module
    (func (export "if.only-then.diverging")
        (if
            (i32.const 0) ;; false
            (then
                (br 0)
            )
        )
    )
)
