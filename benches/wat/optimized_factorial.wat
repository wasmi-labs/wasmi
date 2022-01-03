;; Optimized factorial function, does not use recursion.
(func (export "fac-opt") (param i64) (result i64)
	(local i64)
	(set_local 1 (i64.const 1))
	(block
		(br_if 0 (i64.lt_s (get_local 0) (i64.const 2)))
		(loop
			(set_local 1 (i64.mul (get_local 1) (get_local 0)))
			(set_local 0 (i64.add (get_local 0) (i64.const -1)))
			(br_if 0 (i64.gt_s (get_local 0) (i64.const 1)))
		)
	)
	(get_local 1)
)
