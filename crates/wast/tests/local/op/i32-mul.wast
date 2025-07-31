(module
    (func (export "identity.i32") (param i32) (result i32)
        (local.get 0)
    )
)
(register "utils")

;; Identity

(module
    (func (export "i32.mul(x,1)") (param i32) (result i32)
        (i32.mul (local.get 0) (i32.const 1))
    )
)
(assert_return (invoke "i32.mul(x,1)" (i32.const 0)) (i32.const 0))
(assert_return (invoke "i32.mul(x,1)" (i32.const 1)) (i32.const 1))
(assert_return (invoke "i32.mul(x,1)" (i32.const -1)) (i32.const -1))
(assert_return (invoke "i32.mul(x,1)" (i32.const 42)) (i32.const 42))
(assert_return (invoke "i32.mul(x,1)" (i32.const 0x7FFF_FFFF)) (i32.const 0x7FFF_FFFF))
(assert_return (invoke "i32.mul(x,1)" (i32.const 0x8000_0000)) (i32.const 0x8000_0000))

(module
    (func (export "i32.mul(1,x)") (param i32) (result i32)
        (i32.mul (i32.const 1) (local.get 0))
    )
)
(assert_return (invoke "i32.mul(1,x)" (i32.const 0)) (i32.const 0))
(assert_return (invoke "i32.mul(1,x)" (i32.const 1)) (i32.const 1))
(assert_return (invoke "i32.mul(1,x)" (i32.const -1)) (i32.const -1))
(assert_return (invoke "i32.mul(1,x)" (i32.const 42)) (i32.const 42))
(assert_return (invoke "i32.mul(1,x)" (i32.const 0x7FFF_FFFF)) (i32.const 0x7FFF_FFFF))
(assert_return (invoke "i32.mul(1,x)" (i32.const 0x8000_0000)) (i32.const 0x8000_0000))

(module
    (import "utils" "identity.i32" (func $identity.i32 (param i32) (result i32)))
    (func (export "i32.mul(1,temp)") (param i32) (result i32)
        (i32.mul (i32.const 1) (call $identity.i32 (local.get 0)))
    )
)
(assert_return (invoke "i32.mul(1,temp)" (i32.const 0)) (i32.const 0))
(assert_return (invoke "i32.mul(1,temp)" (i32.const 1)) (i32.const 1))
(assert_return (invoke "i32.mul(1,temp)" (i32.const -1)) (i32.const -1))
(assert_return (invoke "i32.mul(1,temp)" (i32.const 42)) (i32.const 42))
(assert_return (invoke "i32.mul(1,temp)" (i32.const 0x7FFF_FFFF)) (i32.const 0x7FFF_FFFF))
(assert_return (invoke "i32.mul(1,temp)" (i32.const 0x8000_0000)) (i32.const 0x8000_0000))

;; Zero (Annihilator)

(module
    (func (export "i32.mul(x,0)") (param i32) (result i32)
        (i32.mul (local.get 0) (i32.const 0))
    )
)
(assert_return (invoke "i32.mul(x,0)" (i32.const 0)) (i32.const 0))
(assert_return (invoke "i32.mul(x,0)" (i32.const 1)) (i32.const 0))
(assert_return (invoke "i32.mul(x,0)" (i32.const -1)) (i32.const 0))
(assert_return (invoke "i32.mul(x,0)" (i32.const 42)) (i32.const 0))
(assert_return (invoke "i32.mul(x,0)" (i32.const 0x7FFF_FFFF)) (i32.const 0))
(assert_return (invoke "i32.mul(x,0)" (i32.const 0x8000_0000)) (i32.const 0))

(module
    (func (export "i32.mul(0,x)") (param i32) (result i32)
        (i32.mul (i32.const 0) (local.get 0))
    )
)
(assert_return (invoke "i32.mul(0,x)" (i32.const 0)) (i32.const 0))
(assert_return (invoke "i32.mul(0,x)" (i32.const 1)) (i32.const 0))
(assert_return (invoke "i32.mul(0,x)" (i32.const -1)) (i32.const 0))
(assert_return (invoke "i32.mul(0,x)" (i32.const 42)) (i32.const 0))
(assert_return (invoke "i32.mul(0,x)" (i32.const 0x7FFF_FFFF)) (i32.const 0))
(assert_return (invoke "i32.mul(0,x)" (i32.const 0x8000_0000)) (i32.const 0))

(module
    (import "utils" "identity.i32" (func $identity.i32 (param i32) (result i32)))
    (func (export "i32.mul(0,temp)") (param i32) (result i32)
        (i32.mul (i32.const 0) (call $identity.i32 (local.get 0)))
    )
)
(assert_return (invoke "i32.mul(0,temp)" (i32.const 0)) (i32.const 0))
(assert_return (invoke "i32.mul(0,temp)" (i32.const 1)) (i32.const 0))
(assert_return (invoke "i32.mul(0,temp)" (i32.const -1)) (i32.const 0))
(assert_return (invoke "i32.mul(0,temp)" (i32.const 42)) (i32.const 0))
(assert_return (invoke "i32.mul(0,temp)" (i32.const 0x7FFF_FFFF)) (i32.const 0))
(assert_return (invoke "i32.mul(0,temp)" (i32.const 0x8000_0000)) (i32.const 0))

;; Small `lhs` or `rhs` Constants

(module
    (func (export "i32.mul(x,10)") (param i32) (result i32)
        (i32.mul (local.get 0) (i32.const 10))
    )
)
(assert_return (invoke "i32.mul(x,10)" (i32.const 0)) (i32.const 0))
(assert_return (invoke "i32.mul(x,10)" (i32.const 1)) (i32.const 10))
(assert_return (invoke "i32.mul(x,10)" (i32.const -1)) (i32.const -10))
(assert_return (invoke "i32.mul(x,10)" (i32.const 42)) (i32.const 420))
(assert_return (invoke "i32.mul(x,10)" (i32.const 0x8000_0000)) (i32.const 0))

(module
    (func (export "i32.mul(x,-10)") (param i32) (result i32)
        (i32.mul (local.get 0) (i32.const -10))
    )
)
(assert_return (invoke "i32.mul(x,-10)" (i32.const 0)) (i32.const 0))
(assert_return (invoke "i32.mul(x,-10)" (i32.const 1)) (i32.const -10))
(assert_return (invoke "i32.mul(x,-10)" (i32.const -1)) (i32.const 10))
(assert_return (invoke "i32.mul(x,-10)" (i32.const 42)) (i32.const -420))
(assert_return (invoke "i32.mul(x,-10)" (i32.const 0x8000_0000)) (i32.const 0))

(module
    (func (export "i32.mul(10,x)") (param i32) (result i32)
        (i32.mul (i32.const 10) (local.get 0))
    )
)
(assert_return (invoke "i32.mul(10,x)" (i32.const 0)) (i32.const 0))
(assert_return (invoke "i32.mul(10,x)" (i32.const 1)) (i32.const 10))
(assert_return (invoke "i32.mul(10,x)" (i32.const -1)) (i32.const -10))
(assert_return (invoke "i32.mul(10,x)" (i32.const 42)) (i32.const 420))
(assert_return (invoke "i32.mul(10,x)" (i32.const 0x8000_0000)) (i32.const 0))

(module
    (func (export "i32.mul(-10,x)") (param i32) (result i32)
        (i32.mul (i32.const -10) (local.get 0))
    )
)
(assert_return (invoke "i32.mul(-10,x)" (i32.const 0)) (i32.const 0))
(assert_return (invoke "i32.mul(-10,x)" (i32.const 1)) (i32.const -10))
(assert_return (invoke "i32.mul(-10,x)" (i32.const -1)) (i32.const 10))
(assert_return (invoke "i32.mul(-10,x)" (i32.const 42)) (i32.const -420))
(assert_return (invoke "i32.mul(-10,x)" (i32.const 0x8000_0000)) (i32.const 0))

;; Constant Folding

(module
    (func (export "i32.mul(0,0)") (result i32)
        (i32.mul (i32.const 0) (i32.const 0))
    )
)
(assert_return (invoke "i32.mul(0,0)") (i32.const 0))

(module
    (func (export "i32.mul(0,1)") (result i32)
        (i32.mul (i32.const 0) (i32.const 1))
    )
)
(assert_return (invoke "i32.mul(0,1)") (i32.const 0))

(module
    (func (export "i32.mul(1,0)") (result i32)
        (i32.mul (i32.const 1) (i32.const 0))
    )
)
(assert_return (invoke "i32.mul(1,0)") (i32.const 0))

(module
    (func (export "i32.mul(1,1)") (result i32)
        (i32.mul (i32.const 1) (i32.const 1))
    )
)
(assert_return (invoke "i32.mul(1,1)") (i32.const 1))

(module
    (func (export "i32.mul(3,-1)") (result i32)
        (i32.mul (i32.const 3) (i32.const -1))
    )
)
(assert_return (invoke "i32.mul(3,-1)") (i32.const -3))

(module
    (func (export "i32.mul(-1,3)") (result i32)
        (i32.mul (i32.const -1) (i32.const 3))
    )
)
(assert_return (invoke "i32.mul(-1,3)") (i32.const -3))

(module
    (func (export "i32.mul(3,4)") (result i32)
        (i32.mul (i32.const 3) (i32.const 4))
    )
)
(assert_return (invoke "i32.mul(3,4)") (i32.const 12))

(module
    (func (export "i32.mul(-3,-4)") (result i32)
        (i32.mul (i32.const -3) (i32.const -4))
    )
)
(assert_return (invoke "i32.mul(-3,-4)") (i32.const 12))

(module
    (func (export "i32.mul(min,-1)") (result i32)
        (i32.mul (i32.const 0x8000_0000) (i32.const -1))
    )
)
(assert_return (invoke "i32.mul(min,-1)") (i32.const 0x8000_0000))

(module
    (func (export "i32.mul(max,-1)") (result i32)
        (i32.mul (i32.const 0x7FFF_FFFF) (i32.const -1))
    )
)
(assert_return (invoke "i32.mul(max,-1)") (i32.const 0x8000_0001))
