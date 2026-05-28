# wasmi vs stitch — Apple Silicon Comparison

## Machine

- **Chip:** Apple M1 Pro
- **Memory:** 16 GB
- **OS:** macOS (Darwin 25.3.0)
- **Rust:** stable 1.87

## Build Configuration

- **wasmi:** branch `rf-accumulator-based-interpreter-arch`, `codegen-units=1`, `lto=fat`
- **stitch:** v0.1.0 from crates.io (`cargo install makepad-stitch`)

## Results (best of 3, wall-clock seconds)

| Benchmark         | wasmi  | stitch | ratio          |
|-------------------|--------|--------|----------------|
| counter 1B        | 4.74s  | 4.90s  | wasmi 3% faster |
| fuse 1B           | 10.67s | 10.10s | stitch 6% faster |
| fib_iter 100M     | 0.85s  | 1.10s  | wasmi 23% faster |

## Dispatch Sweep (counter 1B, wasmi only)

| Dispatch mode          | Time   | vs default |
|------------------------|--------|------------|
| default (tail-call)    | 4.74s  | baseline   |
| portable (loop+match)  | 10.24s | 2.16x slower |

## Notes

- These results are from an **M1 Pro**. Results may differ significantly on M2/M3/M4.
- stitch does not support the `return_call` proposal, so fibonacci uses a version
  without tail calls (`fibonacci_notail.wat`).
- Workload sizes are large (1B / 100M iterations) to minimize process startup noise.
- The author reports stitch 1.8s vs wasmi 2.8s on counter 1B on a different machine,
  suggesting significant chip-dependent variation.
