;; The below `.wat` file exports a function `call` that takes a `n` of type `i64`.
;; It will iterate `n` times and call the imported function `host_call` every time.
;; 
;; This benchmarks tests the performance of host calls.
;; 
;; After successful execution the `call` function will return `0`.
(module
  (import "benchmark" "host_call" (func $host_call (param i64) (result i64)))
  (func $call (export "call") (param i64) (result i64)
	(block
		(loop
			(br_if
				1
				(i64.eq (local.get 0) (i64.const 0))
			)
			(local.set 0
				(i64.sub
					(call $host_call (local.get 0))
					(i64.const 1)
				)
			)
			(br 0)
		)
	)
	(local.get 0)
  )
)
