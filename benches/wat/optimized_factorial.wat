;; Optimized factorial function, does not use recursion.
(func (export "fac-opt") (param i64) (result i64)
	(local i64)
	(local.set 1 (i64.const 1))
	(block
		(br_if 0 (i64.lt_s (local.get 0) (i64.const 2)))
		(loop
			(local.set 1 (i64.mul (local.get 1) (local.get 0)))
			(local.set 0 (i64.add (local.get 0) (i64.const -1)))
			(br_if 0 (i64.gt_s (local.get 0) (i64.const 1)))
		)
	)
	(local.get 1)
)
