;; Exports a function `call` that takes an input `n`.
;; The exported function calls itself `n` times and traps afterwards.
(module
  (func $call (export "call") (param i32) (result i32)
	block (result i32)
	  local.get 0
	  local.get 0
	  i32.eqz
	  br_if 0

	  i32.const 1
	  i32.sub
	  call $call
	end
	unreachable
  )
)
