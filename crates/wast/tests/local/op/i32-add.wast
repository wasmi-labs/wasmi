(module
    (func (export "identity.i32") (param i32) (result i32)
        (local.get 0)
    )
)
(register "utils")

;; Identity

(module
    (func (export "i32.add(x,0)") (param i32) (result i32)
        (i32.add (local.get 0) (i32.const 0))
    )
)
(assert_return (invoke "i32.add(x,0)" (i32.const 0)) (i32.const 0))
(assert_return (invoke "i32.add(x,0)" (i32.const 1)) (i32.const 1))
(assert_return (invoke "i32.add(x,0)" (i32.const -1)) (i32.const -1))
(assert_return (invoke "i32.add(x,0)" (i32.const 42)) (i32.const 42))
(assert_return (invoke "i32.add(x,0)" (i32.const 0x7FFFFFFF)) (i32.const 0x7FFFFFFF))
(assert_return (invoke "i32.add(x,0)" (i32.const 0x80000000)) (i32.const 0x80000000))

(module
    (func (export "i32.add(0,x)") (param i32) (result i32)
        (i32.add (i32.const 0) (local.get 0))
    )
)
(assert_return (invoke "i32.add(0,x)" (i32.const 0)) (i32.const 0))
(assert_return (invoke "i32.add(0,x)" (i32.const 1)) (i32.const 1))
(assert_return (invoke "i32.add(0,x)" (i32.const -1)) (i32.const -1))
(assert_return (invoke "i32.add(0,x)" (i32.const 42)) (i32.const 42))
(assert_return (invoke "i32.add(0,x)" (i32.const 0x7FFFFFFF)) (i32.const 0x7FFFFFFF))
(assert_return (invoke "i32.add(0,x)" (i32.const 0x80000000)) (i32.const 0x80000000))

(module
    (import "utils" "identity.i32" (func $identity.i32 (param i32) (result i32)))
    (func (export "i32.add(0,temp)") (param i32) (result i32)
        (i32.add (i32.const 0) (call $identity.i32 (local.get 0)))
    )
)
(assert_return (invoke "i32.add(0,temp)" (i32.const 0)) (i32.const 0))
(assert_return (invoke "i32.add(0,temp)" (i32.const 1)) (i32.const 1))
(assert_return (invoke "i32.add(0,temp)" (i32.const -1)) (i32.const -1))
(assert_return (invoke "i32.add(0,temp)" (i32.const 42)) (i32.const 42))
(assert_return (invoke "i32.add(0,temp)" (i32.const 0x7FFFFFFF)) (i32.const 0x7FFFFFFF))
(assert_return (invoke "i32.add(0,temp)" (i32.const 0x80000000)) (i32.const 0x80000000))

;; Small `lhs` or `rhs` Constants

(module
    (func (export "i32.add(x,1)") (param i32) (result i32)
        (i32.add (local.get 0) (i32.const 1))
    )
)
(assert_return (invoke "i32.add(x,1)" (i32.const 0)) (i32.const 1))
(assert_return (invoke "i32.add(x,1)" (i32.const 1)) (i32.const 2))
(assert_return (invoke "i32.add(x,1)" (i32.const -1)) (i32.const 0))
(assert_return (invoke "i32.add(x,1)" (i32.const 42)) (i32.const 43))
(assert_return (invoke "i32.add(x,1)" (i32.const 0x7FFFFFFF)) (i32.const 0x80000000))
(assert_return (invoke "i32.add(x,1)" (i32.const 0x80000000)) (i32.const 0x80000001))

(module
    (func (export "i32.add(x,-1)") (param i32) (result i32)
        (i32.add (local.get 0) (i32.const -1))
    )
)
(assert_return (invoke "i32.add(x,-1)" (i32.const 0)) (i32.const -1))
(assert_return (invoke "i32.add(x,-1)" (i32.const 1)) (i32.const 0))
(assert_return (invoke "i32.add(x,-1)" (i32.const -1)) (i32.const -2))
(assert_return (invoke "i32.add(x,-1)" (i32.const 42)) (i32.const 41))
(assert_return (invoke "i32.add(x,-1)" (i32.const 0x7FFFFFFF)) (i32.const 0x7FFFFFFE))
(assert_return (invoke "i32.add(x,-1)" (i32.const 0x80000000)) (i32.const 0x7FFFFFFF))

(module
    (func (export "i32.add(1,x)") (param i32) (result i32)
        (i32.add (i32.const 1) (local.get 0))
    )
)
(assert_return (invoke "i32.add(1,x)" (i32.const 0)) (i32.const 1))
(assert_return (invoke "i32.add(1,x)" (i32.const 1)) (i32.const 2))
(assert_return (invoke "i32.add(1,x)" (i32.const -1)) (i32.const 0))
(assert_return (invoke "i32.add(1,x)" (i32.const 42)) (i32.const 43))
(assert_return (invoke "i32.add(1,x)" (i32.const 0x7FFFFFFF)) (i32.const 0x80000000))
(assert_return (invoke "i32.add(1,x)" (i32.const 0x80000000)) (i32.const 0x80000001))

(module
    (func (export "i32.add(-1,x)") (param i32) (result i32)
        (i32.add (i32.const -1) (local.get 0))
    )
)
(assert_return (invoke "i32.add(-1,x)" (i32.const 0)) (i32.const -1))
(assert_return (invoke "i32.add(-1,x)" (i32.const 1)) (i32.const 0))
(assert_return (invoke "i32.add(-1,x)" (i32.const -1)) (i32.const -2))
(assert_return (invoke "i32.add(-1,x)" (i32.const 42)) (i32.const 41))
(assert_return (invoke "i32.add(-1,x)" (i32.const 0x7FFFFFFF)) (i32.const 0x7FFFFFFE))
(assert_return (invoke "i32.add(-1,x)" (i32.const 0x80000000)) (i32.const 0x7FFFFFFF))

;; Constant Folding

(module
    (func (export "i32.add(0,0)") (result i32)
        (i32.add (i32.const 0) (i32.const 0))
    )
)
(assert_return (invoke "i32.add(0,0)") (i32.const 0))

(module
    (func (export "i32.add(0,1)") (result i32)
        (i32.add (i32.const 0) (i32.const 1))
    )
)
(assert_return (invoke "i32.add(0,1)") (i32.const 1))

(module
    (func (export "i32.add(1,0)") (result i32)
        (i32.add (i32.const 1) (i32.const 0))
    )
)
(assert_return (invoke "i32.add(1,0)") (i32.const 1))

(module
    (func (export "i32.add(1,-1)") (result i32)
        (i32.add (i32.const 1) (i32.const -1))
    )
)
(assert_return (invoke "i32.add(1,-1)") (i32.const 0))

(module
    (func (export "i32.add(max,-1)") (result i32)
        (i32.add (i32.const 0x7FFFFFFF) (i32.const -1))
    )
)
(assert_return (invoke "i32.add(max,-1)") (i32.const 0x7FFFFFFE))

(module
    (func (export "i32.add(max,1)") (result i32)
        (i32.add (i32.const 0x7FFFFFFF) (i32.const 1))
    )
)
(assert_return (invoke "i32.add(max,1)") (i32.const 0x80000000))

(module
    (func (export "i32.add(min,-1)") (result i32)
        (i32.add (i32.const 0x80000000) (i32.const -1))
    )
)
(assert_return (invoke "i32.add(min,-1)") (i32.const 0x7FFFFFFF))

(module
    (func (export "i32.add(min,1)") (result i32)
        (i32.add (i32.const 0x80000000) (i32.const 1))
    )
)
(assert_return (invoke "i32.add(min,1)") (i32.const 0x80000001))
