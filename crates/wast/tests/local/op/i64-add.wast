(module
    (func (export "identity.i64") (param i64) (result i64)
        (local.get 0)
    )
)
(register "utils")

;; Identity

(module
    (func (export "i64.add(x,0)") (param i64) (result i64)
        (i64.add (local.get 0) (i64.const 0))
    )
)
(assert_return (invoke "i64.add(x,0)" (i64.const 0)) (i64.const 0))
(assert_return (invoke "i64.add(x,0)" (i64.const 1)) (i64.const 1))
(assert_return (invoke "i64.add(x,0)" (i64.const -1)) (i64.const -1))
(assert_return (invoke "i64.add(x,0)" (i64.const 42)) (i64.const 42))
(assert_return (invoke "i64.add(x,0)" (i64.const 0x7FFF_FFFF_FFFF_FFFF)) (i64.const 0x7FFF_FFFF_FFFF_FFFF))
(assert_return (invoke "i64.add(x,0)" (i64.const 0x8000_0000_0000_0000)) (i64.const 0x8000_0000_0000_0000))

(module
    (func (export "i64.add(0,x)") (param i64) (result i64)
        (i64.add (i64.const 0) (local.get 0))
    )
)
(assert_return (invoke "i64.add(0,x)" (i64.const 0)) (i64.const 0))
(assert_return (invoke "i64.add(0,x)" (i64.const 1)) (i64.const 1))
(assert_return (invoke "i64.add(0,x)" (i64.const -1)) (i64.const -1))
(assert_return (invoke "i64.add(0,x)" (i64.const 42)) (i64.const 42))
(assert_return (invoke "i64.add(0,x)" (i64.const 0x7FFF_FFFF_FFFF_FFFF)) (i64.const 0x7FFF_FFFF_FFFF_FFFF))
(assert_return (invoke "i64.add(0,x)" (i64.const 0x8000_0000_0000_0000)) (i64.const 0x8000_0000_0000_0000))

(module
    (import "utils" "identity.i64" (func $identity.i64 (param i64) (result i64)))
    (func (export "i64.add(0,temp)") (param i64) (result i64)
        (i64.add (i64.const 0) (call $identity.i64 (local.get 0)))
    )
)
(assert_return (invoke "i64.add(0,temp)" (i64.const 0)) (i64.const 0))
(assert_return (invoke "i64.add(0,temp)" (i64.const 1)) (i64.const 1))
(assert_return (invoke "i64.add(0,temp)" (i64.const -1)) (i64.const -1))
(assert_return (invoke "i64.add(0,temp)" (i64.const 42)) (i64.const 42))
(assert_return (invoke "i64.add(0,temp)" (i64.const 0x7FFF_FFFF_FFFF_FFFF)) (i64.const 0x7FFF_FFFF_FFFF_FFFF))
(assert_return (invoke "i64.add(0,temp)" (i64.const 0x8000_0000_0000_0000)) (i64.const 0x8000_0000_0000_0000))

;; Small `lhs` or `rhs` Constants

(module
    (func (export "i64.add(x,1)") (param i64) (result i64)
        (i64.add (local.get 0) (i64.const 1))
    )
)
(assert_return (invoke "i64.add(x,1)" (i64.const 0)) (i64.const 1))
(assert_return (invoke "i64.add(x,1)" (i64.const 1)) (i64.const 2))
(assert_return (invoke "i64.add(x,1)" (i64.const -1)) (i64.const 0))
(assert_return (invoke "i64.add(x,1)" (i64.const 42)) (i64.const 43))
(assert_return (invoke "i64.add(x,1)" (i64.const 0x7FFF_FFFF_FFFF_FFFF)) (i64.const 0x8000_0000_0000_0000))
(assert_return (invoke "i64.add(x,1)" (i64.const 0x8000_0000_0000_0000)) (i64.const 0x8000_0000_0000_0001))

(module
    (func (export "i64.add(x,-1)") (param i64) (result i64)
        (i64.add (local.get 0) (i64.const -1))
    )
)
(assert_return (invoke "i64.add(x,-1)" (i64.const 0)) (i64.const -1))
(assert_return (invoke "i64.add(x,-1)" (i64.const 1)) (i64.const 0))
(assert_return (invoke "i64.add(x,-1)" (i64.const -1)) (i64.const -2))
(assert_return (invoke "i64.add(x,-1)" (i64.const 42)) (i64.const 41))
(assert_return (invoke "i64.add(x,-1)" (i64.const 0x7FFF_FFFF_FFFF_FFFF)) (i64.const 0x7FFF_FFFF_FFFF_FFFE))
(assert_return (invoke "i64.add(x,-1)" (i64.const 0x8000_0000_0000_0000)) (i64.const 0x7FFF_FFFF_FFFF_FFFF))

(module
    (func (export "i64.add(1,x)") (param i64) (result i64)
        (i64.add (i64.const 1) (local.get 0))
    )
)
(assert_return (invoke "i64.add(1,x)" (i64.const 0)) (i64.const 1))
(assert_return (invoke "i64.add(1,x)" (i64.const 1)) (i64.const 2))
(assert_return (invoke "i64.add(1,x)" (i64.const -1)) (i64.const 0))
(assert_return (invoke "i64.add(1,x)" (i64.const 42)) (i64.const 43))
(assert_return (invoke "i64.add(1,x)" (i64.const 0x7FFF_FFFF_FFFF_FFFF)) (i64.const 0x8000_0000_0000_0000))
(assert_return (invoke "i64.add(1,x)" (i64.const 0x8000_0000_0000_0000)) (i64.const 0x8000_0000_0000_0001))

(module
    (func (export "i64.add(-1,x)") (param i64) (result i64)
        (i64.add (i64.const -1) (local.get 0))
    )
)
(assert_return (invoke "i64.add(-1,x)" (i64.const 0)) (i64.const -1))
(assert_return (invoke "i64.add(-1,x)" (i64.const 1)) (i64.const 0))
(assert_return (invoke "i64.add(-1,x)" (i64.const -1)) (i64.const -2))
(assert_return (invoke "i64.add(-1,x)" (i64.const 42)) (i64.const 41))
(assert_return (invoke "i64.add(-1,x)" (i64.const 0x7FFF_FFFF_FFFF_FFFF)) (i64.const 0x7FFF_FFFF_FFFF_FFFE))
(assert_return (invoke "i64.add(-1,x)" (i64.const 0x8000_0000_0000_0000)) (i64.const 0x7FFF_FFFF_FFFF_FFFF))

;; Constant Folding

(module
    (func (export "i64.add(0,0)") (result i64)
        (i64.add (i64.const 0) (i64.const 0))
    )
)
(assert_return (invoke "i64.add(0,0)") (i64.const 0))

(module
    (func (export "i64.add(0,1)") (result i64)
        (i64.add (i64.const 0) (i64.const 1))
    )
)
(assert_return (invoke "i64.add(0,1)") (i64.const 1))

(module
    (func (export "i64.add(1,0)") (result i64)
        (i64.add (i64.const 1) (i64.const 0))
    )
)
(assert_return (invoke "i64.add(1,0)") (i64.const 1))

(module
    (func (export "i64.add(1,-1)") (result i64)
        (i64.add (i64.const 1) (i64.const -1))
    )
)
(assert_return (invoke "i64.add(1,-1)") (i64.const 0))

(module
    (func (export "i64.add(max,-1)") (result i64)
        (i64.add (i64.const 0x7FFF_FFFF_FFFF_FFFF) (i64.const -1))
    )
)
(assert_return (invoke "i64.add(max,-1)") (i64.const 0x7FFF_FFFF_FFFF_FFFE))

(module
    (func (export "i64.add(max,1)") (result i64)
        (i64.add (i64.const 0x7FFF_FFFF_FFFF_FFFF) (i64.const 1))
    )
)
(assert_return (invoke "i64.add(max,1)") (i64.const 0x8000_0000_0000_0000))

(module
    (func (export "i64.add(min,-1)") (result i64)
        (i64.add (i64.const 0x8000_0000_0000_0000) (i64.const -1))
    )
)
(assert_return (invoke "i64.add(min,-1)") (i64.const 0x7FFF_FFFF_FFFF_FFFF))

(module
    (func (export "i64.add(min,1)") (result i64)
        (i64.add (i64.const 0x8000_0000_0000_0000) (i64.const 1))
    )
)
(assert_return (invoke "i64.add(min,1)") (i64.const 0x8000_0000_0000_0001))
