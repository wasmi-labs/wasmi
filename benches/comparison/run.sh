#!/usr/bin/env bash
# wasmi vs stitch comparison benchmark
# Usage: ./benches/comparison/run.sh
#
# Prerequisites:
#   cargo install makepad-stitch
#   cargo build --release -p wasmi_cli  (or with CARGO_PROFILE_RELEASE_LTO=fat)
#   brew install wat2wasm (or wabt)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WASMI="${WASMI:-target/release/wasmi}"
STITCH="${STITCH:-makepad-stitch}"
TMPDIR="${TMPDIR:-/tmp}/wasmi-bench-comparison"
RUNS="${RUNS:-3}"

mkdir -p "$TMPDIR"

# Compile WAT to WASM
echo "Compiling WAT files..."
wat2wasm crates/wasmi/benches/wat/counter.wat -o "$TMPDIR/counter.wasm"
wat2wasm crates/wasmi/benches/wat/fuse.wat -o "$TMPDIR/fuse.wasm"
wat2wasm "$SCRIPT_DIR/fibonacci_notail.wat" -o "$TMPDIR/fibonacci_notail.wasm"

echo ""
echo "=== Machine ==="
sysctl -n machdep.cpu.brand_string 2>/dev/null || echo "unknown CPU"
uname -ms
echo ""

# Benchmark function: run N times, report all and best
bench() {
    local label="$1"; shift
    local cmd=("$@")
    echo "  $label:"
    local best=999999
    for i in $(seq 1 "$RUNS"); do
        local t
        t=$( { /usr/bin/time -p "${cmd[@]}" > /dev/null; } 2>&1 | awk '/^real/ {print $2}' )
        printf "    run %d: %ss\n" "$i" "$t"
        if awk "BEGIN{exit !($t < $best)}"; then best="$t"; fi
    done
    printf "    best: %ss\n" "$best"
    echo ""
}

echo "=== counter 1B iterations ==="
bench "wasmi" "$WASMI" run --invoke run "$TMPDIR/counter.wasm" 1000000000
bench "stitch" "$STITCH" "$TMPDIR/counter.wasm" run 1000000000

echo "=== fuse 1B iterations ==="
bench "wasmi" "$WASMI" run --invoke test "$TMPDIR/fuse.wasm" 1000000000
bench "stitch" "$STITCH" "$TMPDIR/fuse.wasm" test 1000000000

echo "=== fibonacci_iter 100M ==="
bench "wasmi" "$WASMI" run --invoke fibonacci_iter "$TMPDIR/fibonacci_notail.wasm" 100000000
bench "stitch" "$STITCH" "$TMPDIR/fibonacci_notail.wasm" fibonacci_iter 100000000

echo "=== dispatch sweep (counter 1B, wasmi only) ==="
echo "Requires rebuilding wasmi with different features."
echo "Run manually:"
echo "  CARGO_PROFILE_RELEASE_LTO=fat cargo build --release -p wasmi_cli"
echo "  # default (tail-call dispatch)"
echo "  time target/release/wasmi run --invoke run $TMPDIR/counter.wasm 1000000000"
echo ""
echo "  CARGO_PROFILE_RELEASE_LTO=fat cargo build --release -p wasmi_cli --features portable-dispatch"
echo "  # portable (loop+match dispatch)"
echo "  time target/release/wasmi run --invoke run $TMPDIR/counter.wasm 1000000000"
