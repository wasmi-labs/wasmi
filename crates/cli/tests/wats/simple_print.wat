(module
    (import "wasi_snapshot_preview1" "fd_write" (func $fd_write 
        (param $fd i32) 
	    (param $iovec i32)
		(param $iovec_len i32)
		(param $size i32) 
        (result i32))
    )

    (memory (export "memory") 1)

    (data (i32.const 8) "Hello World\n")

    (func $main (export "")
        ;; create iovec
        (i32.store (i32.const 0) (i32.const 8))  ;; iovec base 
        (i32.store (i32.const 4) (i32.const 12))  ;; iovec length

        (call $fd_write
            (i32.const 1) ;; fd stdout
            (i32.const 0) ;; iovec list
            (i32.const 1) ;; iovec_len. 1 iovec here
            (i32.const 20) ;; size written 
        )
        drop ;; func main signature has nothing on stack. no return value, so we drop
    )

)
