(module
    (import "env" "instantiate" (func $instantiate))

    (func (export "run")
        (call $instantiate)
    )
)
