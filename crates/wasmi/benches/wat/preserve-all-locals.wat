(module
    (func (export "run") (param i32) (result i32)
        (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
        ;; Push 100 locals onto the stack.
        (local.get 1) (local.get 2) (local.get 3) (local.get 4) (local.get  5)
        (local.get 6) (local.get 7) (local.get 8) (local.get 9) (local.get 10) ;; 10
        (local.get 1) (local.get 2) (local.get 3) (local.get 4) (local.get  5)
        (local.get 6) (local.get 7) (local.get 8) (local.get 9) (local.get 10) ;; 20
        (local.get 1) (local.get 2) (local.get 3) (local.get 4) (local.get  5)
        (local.get 6) (local.get 7) (local.get 8) (local.get 9) (local.get 10) ;; 30
        (local.get 1) (local.get 2) (local.get 3) (local.get 4) (local.get  5)
        (local.get 6) (local.get 7) (local.get 8) (local.get 9) (local.get 10) ;; 40
        (local.get 1) (local.get 2) (local.get 3) (local.get 4) (local.get  5)
        (local.get 6) (local.get 7) (local.get 8) (local.get 9) (local.get 10) ;; 50
        (local.get 1) (local.get 2) (local.get 3) (local.get 4) (local.get  5)
        (local.get 6) (local.get 7) (local.get 8) (local.get 9) (local.get 10) ;; 60
        (local.get 1) (local.get 2) (local.get 3) (local.get 4) (local.get  5)
        (local.get 6) (local.get 7) (local.get 8) (local.get 9) (local.get 10) ;; 70
        (local.get 1) (local.get 2) (local.get 3) (local.get 4) (local.get  5)
        (local.get 6) (local.get 7) (local.get 8) (local.get 9) (local.get 10) ;; 80
        (local.get 1) (local.get 2) (local.get 3) (local.get 4) (local.get  5)
        (local.get 6) (local.get 7) (local.get 8) (local.get 9) (local.get 10) ;; 90
        (local.get 1) (local.get 2) (local.get 3) (local.get 4) (local.get  5)
        (local.get 6) (local.get 7) (local.get 8) (local.get 9) (local.get 10) ;; 100
        ;; Now push a sequence of blocks and `local.get` to force preservation of all locals.
        (block
            (local.get 0)
            (block
                (local.get 1)
                (block
                    (local.get 2)
                    (block
                        (local.get 3)
                        (block
                            (local.get 4)
                            (block
                                (local.get 5)
                                (block
                                    (local.get 6)
                                    (block
                                        (local.get 7)
                                        (block
                                            (local.get 8)
                                            (block
                                                (local.get 9)
                                                (block
                                                    (local.get 10)
                                                    (drop)
                                                )
                                                (drop)
                                            )
                                            (drop)
                                        )
                                        (drop)
                                    )
                                    (drop)
                                )
                                (drop)
                            )
                            (drop)
                        )
                        (drop)
                    )
                    (drop)
                )
                (drop)
            )
            (drop)
        )
        ;; Drop all 100 operands from the stack.
        (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop)
        (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop)
        (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop)
        (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop)
        (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop)
        (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop)
        (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop)
        (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop)
        (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop)
        (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop) (drop)
        ;; Return input to caller.
        (local.get 0)
    )
)
