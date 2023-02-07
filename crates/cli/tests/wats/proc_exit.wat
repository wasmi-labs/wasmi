;; Test Notes
;; Tests fd_write and proc_exit wasi 'syscalls'
;; Also tests environ_get

(module
  (import "wasi_snapshot_preview1" "proc_exit" (func $proc_exit (param i32)))
  (func $main (export "")
    (call $proc_exit (i32.const 1))
  )
  (memory (export "memory") 1)
)