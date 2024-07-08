(module
    (import "env" "compile" (func $compile))

    (func (export "run")
        (call $compile)
    )
)
