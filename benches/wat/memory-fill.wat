;; Exports a function `fill` that fills the bytes of the
;; linear memory with the given `u8`-wrapped `i32` value.
;;
;; # Note
;;
;; The `len` and `offset` parameters tell where to fill
;; contents within the linear memory.
(module
    (memory (export "mem") 1)
    (func (export "fill_bytes") (param $ptr i32) (param $len i32) (param $value i32)
        (local $n i32)
        (block $exit
            (loop $loop
                (br_if ;; exit loop if $n == $len
                    $exit
                    (i32.eq
                        (local.get $n)
                        (local.get $len)
                    )
                )
                (i32.store8 offset=0 ;; store $value at mem[ptr+n]
                    (i32.add
                        (local.get $ptr)
                        (local.get $n)
                    )
                    (local.get $value)
                )
                (local.set $n ;; increment n
                    (i32.add (local.get $n) (i32.const 1))
                 )
                (br $loop) ;; continue loop
            )
        )
        (return)
    )
)
