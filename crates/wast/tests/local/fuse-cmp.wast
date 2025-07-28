;; Regression tests asserting that `fuse-eqz` and `fuse-nez` properly
;; update the type of the resulting operand they successfully fused.
;;
;; The bug made translation of `i64.extend_i32_u` panic since `fuse-eqz`
;; and `fuse-nez` did not properly update the type on the translation type
;; upon successful op-code fusion.

(module
    ;; fuse-nez

    (func (export "fuse.and+eqz") (param i64 i64) (result i64)
        (i64.eqz (i64.and (local.get 0) (local.get 1)))
        i64.extend_i32_u
    )

    (func (export "fuse.or+eqz") (param i64 i64) (result i64)
        (i64.eqz (i64.or (local.get 0) (local.get 1)))
        i64.extend_i32_u
    )

    (func (export "fuse.xor+eqz") (param i64 i64) (result i64)
        (i64.eqz (i64.xor (local.get 0) (local.get 1)))
        i64.extend_i32_u
    )

    ;; fuse-nez

    (func (export "fuse.and+nez") (param i64 i64) (result i64)
        (i64.ne
            (i64.and (local.get 0) (local.get 1))
            (i64.const 0)
        )
        i64.extend_i32_u
    )

    (func (export "fuse.or+nez") (param i64 i64) (result i64)
        (i64.ne
            (i64.or (local.get 0) (local.get 1))
            (i64.const 0)
        )
        i64.extend_i32_u
    )

    (func (export "fuse.xor+nez") (param i64 i64) (result i64)
        (i64.ne
            (i64.xor (local.get 0) (local.get 1))
            (i64.const 0)
        )
        i64.extend_i32_u
    )
)

;; and + eqz

(assert_return
    (invoke "fuse.and+eqz" (i64.const 0) (i64.const 0))
    (i64.const 1)
)
(assert_return
    (invoke "fuse.and+eqz" (i64.const 0) (i64.const 1))
    (i64.const 1)
)
(assert_return
    (invoke "fuse.and+eqz" (i64.const 1) (i64.const 0))
    (i64.const 1)
)
(assert_return
    (invoke "fuse.and+eqz" (i64.const 1) (i64.const 1))
    (i64.const 0)
)

;; or + eqz

(assert_return
    (invoke "fuse.or+eqz" (i64.const 0) (i64.const 0))
    (i64.const 1)
)
(assert_return
    (invoke "fuse.or+eqz" (i64.const 0) (i64.const 1))
    (i64.const 0)
)
(assert_return
    (invoke "fuse.or+eqz" (i64.const 1) (i64.const 0))
    (i64.const 0)
)
(assert_return
    (invoke "fuse.or+eqz" (i64.const 1) (i64.const 1))
    (i64.const 0)
)

;; xor + eqz

(assert_return
    (invoke "fuse.xor+eqz" (i64.const 0) (i64.const 0))
    (i64.const 1)
)
(assert_return
    (invoke "fuse.xor+eqz" (i64.const 0) (i64.const 1))
    (i64.const 0)
)
(assert_return
    (invoke "fuse.xor+eqz" (i64.const 1) (i64.const 0))
    (i64.const 0)
)
(assert_return
    (invoke "fuse.xor+eqz" (i64.const 1) (i64.const 1))
    (i64.const 1)
)

;; and + nez

(assert_return
    (invoke "fuse.and+nez" (i64.const 0) (i64.const 0))
    (i64.const 0)
)
(assert_return
    (invoke "fuse.and+nez" (i64.const 0) (i64.const 1))
    (i64.const 0)
)
(assert_return
    (invoke "fuse.and+nez" (i64.const 1) (i64.const 0))
    (i64.const 0)
)
(assert_return
    (invoke "fuse.and+nez" (i64.const 1) (i64.const 1))
    (i64.const 1)
)

;; or + nez

(assert_return
    (invoke "fuse.or+nez" (i64.const 0) (i64.const 0))
    (i64.const 0)
)
(assert_return
    (invoke "fuse.or+nez" (i64.const 0) (i64.const 1))
    (i64.const 1)
)
(assert_return
    (invoke "fuse.or+nez" (i64.const 1) (i64.const 0))
    (i64.const 1)
)
(assert_return
    (invoke "fuse.or+nez" (i64.const 1) (i64.const 1))
    (i64.const 1)
)

;; xor + nez

(assert_return
    (invoke "fuse.xor+nez" (i64.const 0) (i64.const 0))
    (i64.const 0)
)
(assert_return
    (invoke "fuse.xor+nez" (i64.const 0) (i64.const 1))
    (i64.const 1)
)
(assert_return
    (invoke "fuse.xor+nez" (i64.const 1) (i64.const 0))
    (i64.const 1)
)
(assert_return
    (invoke "fuse.xor+nez" (i64.const 1) (i64.const 1))
    (i64.const 0)
)

;; Regression tests to check that `fuse-nez` and `fuse-eqz` result in fused
;; `cmp` instructions with the correct result `Reg`. The bug yielded incorrect
;; result `Reg` due to stack heights when `lhs` was the zero immediate and
;; `rhs` was a `temp` operand on the translation stack.

(module
    (func (export "nez.imm.temp") (param i64) (result i32)
        (i32.ne
            (i32.const 0)
            (i64.lt_u (local.get 0) (i64.const 1))
        )
    )

    (func (export "eqz.imm.temp") (param i64) (result i32)
        (i32.eq
            (i32.const 0)
            (i64.lt_u (local.get 0) (i64.const 0))
        )
    )
)

(assert_return
    (invoke "nez.imm.temp" (i64.const 0))
    (i32.const 1)
)
(assert_return
    (invoke "eqz.imm.temp" (i64.const 0))
    (i32.const 1)
)

(module
  (global i32 (i32.const 42))
  (func (export "invalid-fuse-global-get") (param i32 i32) (result i32 i32)
    (i32.lt_s
        (local.get 0)
        (local.get 1)
    )
    (global.get 0)
    (i32.eqz) ;; must not fuse with `i32.lt_s`
  )
)

(assert_return
    (invoke "invalid-fuse-global-get" (i32.const 0) (i32.const 0))
    (i32.const 0) (i32.const 0)
)
(assert_return
    (invoke "invalid-fuse-global-get" (i32.const 0) (i32.const 1))
    (i32.const 1) (i32.const 0)
)
(assert_return
    (invoke "invalid-fuse-global-get" (i32.const 1) (i32.const 0))
    (i32.const 0) (i32.const 0)
)
(assert_return
    (invoke "invalid-fuse-global-get" (i32.const 1) (i32.const 1))
    (i32.const 0) (i32.const 0)
)

(module
  (func (export "invalid.fuse.f32.nan") (param f32) (result i32)
    (f32.le
        (local.get 0)
        (local.get 0)
    )
    (i32.eqz)
  )

  (func (export "invalid.fuse.f64.nan") (param f64) (result i32)
    (f64.le
        (local.get 0)
        (local.get 0)
    )
    (i32.eqz)
  )
)

(assert_return
    (invoke "invalid.fuse.f32.nan" (f32.const -nan:0x7fffff))
    (i32.const 1)
)
(assert_return
    (invoke "invalid.fuse.f64.nan" (f64.const -nan:0xfffffffffffff))
    (i32.const 1)
)
