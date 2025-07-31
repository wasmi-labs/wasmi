(module
    (func (export "identity.i64") (param i64) (result i64)
        (local.get 0)
    )
)
(register "utils")

;; Identity

(module
    (func (export "i64.mul(x,1)") (param i64) (result i64)
        (i64.mul (local.get 0) (i64.const 1))
    )
)
(assert_return (invoke "i64.mul(x,1)" (i64.const 0)) (i64.const 0))
(assert_return (invoke "i64.mul(x,1)" (i64.const 1)) (i64.const 1))
(assert_return (invoke "i64.mul(x,1)" (i64.const -1)) (i64.const -1))
(assert_return (invoke "i64.mul(x,1)" (i64.const 42)) (i64.const 42))
(assert_return (invoke "i64.mul(x,1)" (i64.const 0x7FFF_FFFF_FFFF_FFFF)) (i64.const 0x7FFF_FFFF_FFFF_FFFF))
(assert_return (invoke "i64.mul(x,1)" (i64.const 0x8000_0000_0000_0000)) (i64.const 0x8000_0000_0000_0000))

(module
    (func (export "i64.mul(1,x)") (param i64) (result i64)
        (i64.mul (i64.const 1) (local.get 0))
    )
)
(assert_return (invoke "i64.mul(1,x)" (i64.const 0)) (i64.const 0))
(assert_return (invoke "i64.mul(1,x)" (i64.const 1)) (i64.const 1))
(assert_return (invoke "i64.mul(1,x)" (i64.const -1)) (i64.const -1))
(assert_return (invoke "i64.mul(1,x)" (i64.const 42)) (i64.const 42))
(assert_return (invoke "i64.mul(1,x)" (i64.const 0x7FFF_FFFF_FFFF_FFFF)) (i64.const 0x7FFF_FFFF_FFFF_FFFF))
(assert_return (invoke "i64.mul(1,x)" (i64.const 0x8000_0000_0000_0000)) (i64.const 0x8000_0000_0000_0000))

(module
    (import "utils" "identity.i64" (func $identity.i64 (param i64) (result i64)))
    (func (export "i64.mul(1,temp)") (param i64) (result i64)
        (i64.mul (i64.const 1) (call $identity.i64 (local.get 0)))
    )
)
(assert_return (invoke "i64.mul(1,temp)" (i64.const 0)) (i64.const 0))
(assert_return (invoke "i64.mul(1,temp)" (i64.const 1)) (i64.const 1))
(assert_return (invoke "i64.mul(1,temp)" (i64.const -1)) (i64.const -1))
(assert_return (invoke "i64.mul(1,temp)" (i64.const 42)) (i64.const 42))
(assert_return (invoke "i64.mul(1,temp)" (i64.const 0x7FFF_FFFF_FFFF_FFFF)) (i64.const 0x7FFF_FFFF_FFFF_FFFF))
(assert_return (invoke "i64.mul(1,temp)" (i64.const 0x8000_0000_0000_0000)) (i64.const 0x8000_0000_0000_0000))

;; Zero (Annihilator)

(module
    (func (export "i64.mul(x,0)") (param i64) (result i64)
        (i64.mul (local.get 0) (i64.const 0))
    )
)
(assert_return (invoke "i64.mul(x,0)" (i64.const 0)) (i64.const 0))
(assert_return (invoke "i64.mul(x,0)" (i64.const 1)) (i64.const 0))
(assert_return (invoke "i64.mul(x,0)" (i64.const -1)) (i64.const 0))
(assert_return (invoke "i64.mul(x,0)" (i64.const 42)) (i64.const 0))
(assert_return (invoke "i64.mul(x,0)" (i64.const 0x7FFF_FFFF_FFFF_FFFF)) (i64.const 0))
(assert_return (invoke "i64.mul(x,0)" (i64.const 0x8000_0000_0000_0000)) (i64.const 0))

(module
    (func (export "i64.mul(0,x)") (param i64) (result i64)
        (i64.mul (i64.const 0) (local.get 0))
    )
)
(assert_return (invoke "i64.mul(0,x)" (i64.const 0)) (i64.const 0))
(assert_return (invoke "i64.mul(0,x)" (i64.const 1)) (i64.const 0))
(assert_return (invoke "i64.mul(0,x)" (i64.const -1)) (i64.const 0))
(assert_return (invoke "i64.mul(0,x)" (i64.const 42)) (i64.const 0))
(assert_return (invoke "i64.mul(0,x)" (i64.const 0x7FFF_FFFF_FFFF_FFFF)) (i64.const 0))
(assert_return (invoke "i64.mul(0,x)" (i64.const 0x8000_0000_0000_0000)) (i64.const 0))

(module
    (import "utils" "identity.i64" (func $identity.i64 (param i64) (result i64)))
    (func (export "i64.mul(0,temp)") (param i64) (result i64)
        (i64.mul (i64.const 0) (call $identity.i64 (local.get 0)))
    )
)
(assert_return (invoke "i64.mul(0,temp)" (i64.const 0)) (i64.const 0))
(assert_return (invoke "i64.mul(0,temp)" (i64.const 1)) (i64.const 0))
(assert_return (invoke "i64.mul(0,temp)" (i64.const -1)) (i64.const 0))
(assert_return (invoke "i64.mul(0,temp)" (i64.const 42)) (i64.const 0))
(assert_return (invoke "i64.mul(0,temp)" (i64.const 0x7FFF_FFFF_FFFF_FFFF)) (i64.const 0))
(assert_return (invoke "i64.mul(0,temp)" (i64.const 0x8000_0000_0000_0000)) (i64.const 0))

;; Small `lhs` or `rhs` Constants

(module
    (func (export "i64.mul(x,10)") (param i64) (result i64)
        (i64.mul (local.get 0) (i64.const 10))
    )
)
(assert_return (invoke "i64.mul(x,10)" (i64.const 0)) (i64.const 0))
(assert_return (invoke "i64.mul(x,10)" (i64.const 1)) (i64.const 10))
(assert_return (invoke "i64.mul(x,10)" (i64.const -1)) (i64.const -10))
(assert_return (invoke "i64.mul(x,10)" (i64.const 42)) (i64.const 420))
(assert_return (invoke "i64.mul(x,10)" (i64.const 0x8000_0000_0000_0000)) (i64.const 0))

(module
    (func (export "i64.mul(x,-10)") (param i64) (result i64)
        (i64.mul (local.get 0) (i64.const -10))
    )
)
(assert_return (invoke "i64.mul(x,-10)" (i64.const 0)) (i64.const 0))
(assert_return (invoke "i64.mul(x,-10)" (i64.const 1)) (i64.const -10))
(assert_return (invoke "i64.mul(x,-10)" (i64.const -1)) (i64.const 10))
(assert_return (invoke "i64.mul(x,-10)" (i64.const 42)) (i64.const -420))
(assert_return (invoke "i64.mul(x,-10)" (i64.const 0x8000_0000_0000_0000)) (i64.const 0))

(module
    (func (export "i64.mul(10,x)") (param i64) (result i64)
        (i64.mul (i64.const 10) (local.get 0))
    )
)
(assert_return (invoke "i64.mul(10,x)" (i64.const 0)) (i64.const 0))
(assert_return (invoke "i64.mul(10,x)" (i64.const 1)) (i64.const 10))
(assert_return (invoke "i64.mul(10,x)" (i64.const -1)) (i64.const -10))
(assert_return (invoke "i64.mul(10,x)" (i64.const 42)) (i64.const 420))
(assert_return (invoke "i64.mul(10,x)" (i64.const 0x8000_0000_0000_0000)) (i64.const 0))

(module
    (func (export "i64.mul(-10,x)") (param i64) (result i64)
        (i64.mul (i64.const -10) (local.get 0))
    )
)
(assert_return (invoke "i64.mul(-10,x)" (i64.const 0)) (i64.const 0))
(assert_return (invoke "i64.mul(-10,x)" (i64.const 1)) (i64.const -10))
(assert_return (invoke "i64.mul(-10,x)" (i64.const -1)) (i64.const 10))
(assert_return (invoke "i64.mul(-10,x)" (i64.const 42)) (i64.const -420))
(assert_return (invoke "i64.mul(-10,x)" (i64.const 0x8000_0000_0000_0000)) (i64.const 0))

;; Constant Folding

(module
    (func (export "i64.mul(0,0)") (result i64)
        (i64.mul (i64.const 0) (i64.const 0))
    )
)
(assert_return (invoke "i64.mul(0,0)") (i64.const 0))

(module
    (func (export "i64.mul(0,1)") (result i64)
        (i64.mul (i64.const 0) (i64.const 1))
    )
)
(assert_return (invoke "i64.mul(0,1)") (i64.const 0))

(module
    (func (export "i64.mul(1,0)") (result i64)
        (i64.mul (i64.const 1) (i64.const 0))
    )
)
(assert_return (invoke "i64.mul(1,0)") (i64.const 0))

(module
    (func (export "i64.mul(1,1)") (result i64)
        (i64.mul (i64.const 1) (i64.const 1))
    )
)
(assert_return (invoke "i64.mul(1,1)") (i64.const 1))

(module
    (func (export "i64.mul(3,-1)") (result i64)
        (i64.mul (i64.const 3) (i64.const -1))
    )
)
(assert_return (invoke "i64.mul(3,-1)") (i64.const -3))

(module
    (func (export "i64.mul(-1,3)") (result i64)
        (i64.mul (i64.const -1) (i64.const 3))
    )
)
(assert_return (invoke "i64.mul(-1,3)") (i64.const -3))

(module
    (func (export "i64.mul(3,4)") (result i64)
        (i64.mul (i64.const 3) (i64.const 4))
    )
)
(assert_return (invoke "i64.mul(3,4)") (i64.const 12))

(module
    (func (export "i64.mul(-3,-4)") (result i64)
        (i64.mul (i64.const -3) (i64.const -4))
    )
)
(assert_return (invoke "i64.mul(-3,-4)") (i64.const 12))

(module
    (func (export "i64.mul(min,-1)") (result i64)
        (i64.mul (i64.const 0x8000_0000_0000_0000) (i64.const -1))
    )
)
(assert_return (invoke "i64.mul(min,-1)") (i64.const 0x8000_0000_0000_0000))

(module
    (func (export "i64.mul(max,-1)") (result i64)
        (i64.mul (i64.const 0x7FFF_FFFF_FFFF_FFFF) (i64.const -1))
    )
)
(assert_return (invoke "i64.mul(max,-1)") (i64.const 0x8000_0000_0000_0001))
