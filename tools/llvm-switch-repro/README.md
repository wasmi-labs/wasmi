# `--switch-to-lookup` codegen-pathology reproducer

Minimal, parameterized reproducer for the compiler issue that forces Wasmi to set

```toml
# .cargo/config.toml
[build]
rustflags = ["-C", "llvm-args=--switch-to-lookup=true"]
```

Without that flag, building Wasmi with `indirect-dispatch` is ~10–20x slower to compile
and produces a slower binary. See [`ISSUE.md`](./ISSUE.md) for the full write-up.

## What it reproduces

Wasmi's hot-path dispatch helper `op_code_to_handler`
(`crates/wasmi/src/engine/executor/handler/dispatch/backend/tail.rs`) is a `match` over a
`#[repr(u16)]` enum with **dense, contiguous** discriminants `0..N` where every arm
returns a **distinct constant** (a function pointer). That is the textbook candidate for
LLVM's SimplifyCFG *switch-to-lookup-table* transform — it should collapse to a single
indexed load from a constant array, independent of `N`. Once `N` grows past ~1000 the
default `-O2` pipeline stops forming the table and lowers the switch as a jump table /
decision tree with ~`N` blocks instead.

The reproducer recreates exactly this shape, parameterized by the case count `N`, in both
C and Rust. Each handler returns its own index so the bodies differ and cannot be merged
(identical bodies would let the compiler fold the switch into a constant and hide the
issue).

## Files

| File | Purpose |
| --- | --- |
| `gen.py` | Emit the C or Rust reproducer for a given `N` (Python 3 stdlib only). |
| `measure.sh` | Sweep `N`; build each with `-switch-to-lookup` **default / off / on**; print a Markdown table of compile time + `.text` size; dump asm for one `N`. |
| `ISSUE.md` | Paste-ready bug report (fill the version + measurement placeholders). |

## Quick start

```bash
# Generate the faithful reproducer (the one that actually reproduces by default)
python3 gen.py --lang rust --shape interp --count 1024 --out /tmp/repro.rs

# Full sweep + asm dump (prints a Markdown table)
bash measure.sh

# Narrower/faster sweep
NS="256 512 1024" ASM_N=1024 bash measure.sh
```

`--shape interp` is the faithful case (inlined N-arm switch). `--shape handler` is a single
non-inlined switch that does NOT reproduce. `--shape table` is the candidate fix.

## Comparing default vs on vs off (the key)

Use **rustc** (its LLVM 22 has the option). At N=1024 the default already shows the
pathology; the flag toggles it:

```bash
rustc -O --crate-type=lib --emit=asm                                      /tmp/repro.rs -o default.s  # slow (~32s)
rustc -O --crate-type=lib --emit=asm -Cllvm-args=--switch-to-lookup=true  /tmp/repro.rs -o on.s       # fast (~0.5s)
rustc -O --crate-type=lib --emit=asm -Cllvm-args=--switch-to-lookup=false /tmp/repro.rs -o off.s      # worst (jump tables, huge)
```

`default.s` and `on.s` are the same shape (no jump tables) — but `default` takes ~64× longer
to *produce*; `off.s` shows the explicit jump-table codegen (and explodes in size).

Note on clang: Apple clang 17 reproduces the slow **default** compile (so this is an
LLVM-level issue, not rustc-specific), but that LLVM predates the `-switch-to-lookup`
option, so it can't toggle the flag. Use an upstream `clang`/`opt` new enough to expose it:

```bash
clang -O2 -emit-llvm -S /tmp/repro.c -o repro.ll
opt -passes='default<O2>' -switch-to-lookup=true  repro.ll -S -o on.ll
opt -passes='default<O2>' -switch-to-lookup=false repro.ll -S -o off.ll
```

## Findings (rustc 1.96 / LLVM 22.1.2) — root cause confirmed

The trigger is **inlining**, not a lone large switch. Wasmi's `op_code_to_handler`
(an N-arm `match` returning a fn pointer) is `#[inline(always)]` and gets inlined into the
tail of every one of the ~N `#[inline(never)]` instruction handlers (via `dispatch!`). So
the N-arm switch is replicated N times. Use `--shape`:

- `handler` — a single, non-inlined switch. rustc's `default` already turns it into a
  lookup table at every N (1100…16384, arm64 + x86_64). **Does not reproduce** the bug;
  the flag is a no-op. This is why a naive minimal example fails.
- `interp` — the faithful shape (N handlers, each inlining the N-arm `dispatch` match).
  **Reproduces it.** At N=1024: `default` 32.1s vs `-switch-to-lookup=true` 0.50s (~64×);
  `=false` is the catastrophe (80s, 225 MB asm, explicit jump tables). Default compile
  time is super-quadratic in N (≈ N^2.8).
- `table` — candidate fix: index a `static [Handler; N]` instead of `match`. No switch to
  lower → 0.48s flag-independent.

Real `wasmi` crate, `cargo build --release -p wasmi -F indirect-dispatch`:
`-switch-to-lookup=true` **13.9s** vs no flag **348.2s** (5m50s) ≈ **25×**, plus a reported
~50–100% execution slowdown. (At `-O0`/dev the flag is a no-op — it's an optimization pass —
so the dev slowdown from `indirect-dispatch` is just raw code-size overhead, ~3×.)

## Candidate Wasmi-side fixes (validated in the toy, N=1024, default pipeline, no flag)

Both remove the dependency on the global `-switch-to-lookup` flag:

| approach | default compile | notes |
| --- | ---: | --- |
| current (`#[inline(always)]` match, inlined into N handlers) | ~32 s | the bug |
| **A. drop `#[inline(always)]`** on `op_code_to_handler` (one shared switch) | **0.46 s** | one-line diff; adds a non-inlined `call` per dispatch before the indirect tail-call (possible small hot-path cost) |
| **B. `static [Handler; N]` + inlined indexed load** (`--shape table`) | **0.48 s** | no switch to lower; stays fully inlined (just a load) so it should preserve runtime best; needs the array generated by `for_each_op!` |

Both still need confirming on the **real** crate (build `wasmi -F indirect-dispatch --release`
with the change and `RUSTFLAGS=""`). B is the better bet for keeping the ~50–100% runtime
back.

## Is it an LLVM bug or a rustc bug?

**LLVM.** rustc is not disabling the transform (its `default` forms the table for a
standalone switch); the only lever that changes behavior is the LLVM `cl::opt`, and `clang`
reproduces via `-mllvm -switch-to-lookup=…`. The issue is that switch→lookup-table stops
firing for the *inlined copies* of a constant-returning switch. File against
**llvm/llvm-project**. (Note: on this machine `clang` is a different LLVM (Apple 17) than
rustc's (22), so use rustc for an apples-to-apples comparison.)

## Notes

- `measure.sh` uses `/usr/bin/time -p` (POSIX) and `llvm-size`/`size` when available,
  falling back to object byte size. Local timings are noisy — run on a quiet, controlled
  machine and fill the numbers into `ISSUE.md`.
- This folder is a diagnostic aid; it is not part of the Wasmi build or CI and has no
  third-party dependencies.
