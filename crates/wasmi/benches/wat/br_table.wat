;; Exports a function `match` that takes an input `n`.
;; The exported function counts an integer `n` times and then returns 0.
(module
    (func (export "br_table") (param $case i32) (result i32)
        (block $case0
            (block $case1
                (block $case2
                    (block $case3
                        (block $case4
                            (block $case5
                                (block $case6
                                    (block $case7
                                        (block $case8
                                            (block $case9
                                                (block $case10
                                                    (block $case11
                                                        (block $case12
                                                            (block $case13
                                                                (block $case14
                                                                    (block $case15
                                                                        (br_table
                                                                            $case0 $case1 $case2 $case3
                                                                            $case4 $case5 $case6 $case7
                                                                            $case8 $case9 $case10 $case11
                                                                            $case12 $case13 $case14 $case15
                                                                            (local.get $case)
                                                                        )
                                                                    )
                                                                    (return i32.const -160)
                                                                )
                                                                (return i32.const -150)
                                                            )
                                                            (return i32.const -140)
                                                        )
                                                        (return i32.const -130)
                                                    )
                                                    (return i32.const -120)
                                                )
                                                (return i32.const -110)
                                            )
                                            (return i32.const -100)
                                        )
                                        (return i32.const -90)
                                    )
                                    (return i32.const -80)
                                )
                                (return i32.const -70)
                            )
                            (return i32.const -60)
                        )
                        (return i32.const -50)
                    )
                    (return i32.const -40)
                )
                (return i32.const -30)
            )
            (return i32.const -20)
        )
        (return i32.const -10)
    )
)
