<!--
Paste-ready report. Fill remaining <…> (versions already detected below) and attach
gen.py + a generated interp/repro.rs (or a reduced .ll) before filing.
Venue: evidence points at LLVM (see "LLVM vs rustc"). File against llvm/llvm-project.
-->

# `-O2`: an `#[inline(always)]` switch returning a constant per case, inlined into many callers, makes switch→lookup-table either bail (jump tables) or take quadratic compile time; `-switch-to-lookup=true` fixes it

## Summary

A small `#[inline(always)]` function containing a `switch` over a dense, contiguous
integer (a `#[repr(u16)]` enum, discriminants `0..N`) that returns a **distinct constant
function pointer per case** is, on its own, turned into a lookup table by the default
`-O2` pipeline — good. But when that function is **inlined into ~N callers** (a classic
threaded-interpreter dispatch loop), the default pipeline degrades badly:

- compile time becomes roughly **quadratic** in `N` (each of N callers carries the N-arm
  switch), and
- the switch→lookup-table transform that fires for the single-call case stops firing per
  inlined copy, so the switches fall through to SelectionDAG and lower as **jump tables**
  (huge `.text`, an indirect branch per dispatch → slower runtime).

Forcing **`-switch-to-lookup=true`** makes every inlined copy collapse to a single shared
constant-array load again, and both problems vanish.

This is not hypothetical: it forced the Wasmi WebAssembly interpreter to set
`-C llvm-args=--switch-to-lookup=true` globally once its instruction set grew past ~1000
opcodes.

## Environment

- LLVM **22.1.2** (as shipped with rustc). The slow default compile also reproduces with
  Apple clang 17 (a different, older LLVM), so it is not rustc-specific; but Apple clang 17
  lacks the `-switch-to-lookup` option, so toggle the flag with rustc (LLVM 22) or an
  upstream `clang`/`opt` new enough to expose it.
- rustc **1.96.0**
- Targets: **aarch64-apple-darwin** (real build measured here); the isolated transform
  reproduces on **x86_64** as well.
- Opt: `-O2` / cargo `--release`. (At `-O0` the transform never runs, so the flag is a
  no-op and only raw code-size overhead remains.)

## Real-world numbers (the `wasmi` crate)

`cargo build --release -p wasmi -F indirect-dispatch`, identical except for the flag:

| build | `wasmi` codegen time |
| --- | ---: |
| `-switch-to-lookup=true` (current workaround) | **13.9 s** |
| no flag (LLVM default) | **348.2 s** (5m50s) |

≈ **25× compile-time regression**, plus a reported ~50–100% **execution** slowdown on the
interpreter hot path (consistent with jump-table dispatch vs. one indexed load).

## Minimal reproducer

`gen.py --shape interp` emits N trivial handlers, each `#[inline(never)]`, each ending by
calling an `#[inline(always)]` `dispatch(code)` that holds the N-arm match and tail-calling
its result — so the match is inlined into all N handlers. (Each handler has a distinct body
so identical-code-folding cannot merge them.)

```rust
#[repr(u16)] pub enum OpCode { V0, /* … */ V{N-1} }   // dense 0..N
pub type Handler = fn(&mut St) -> u64;

#[inline(always)]                       // inlined into every handler below
fn dispatch(code: OpCode) -> Handler {
    match code { OpCode::V0 => h0, /* … */ }
}
#[inline(never)]
fn h0(s:&mut St)->u64 { s.acc=s.acc.wrapping_add(0);
    let c = unsafe { core::mem::transmute::<u16,OpCode>(s.code) }; dispatch(c)(s) }
/* … hN … */
```

```bash
python3 gen.py --lang rust --shape interp --count 1024 --out repro.rs
rustc -O --crate-type=lib --emit=asm                                  repro.rs -o default.s
rustc -O --crate-type=lib --emit=asm -Cllvm-args=--switch-to-lookup=true  repro.rs -o on.s
rustc -O --crate-type=lib --emit=asm -Cllvm-args=--switch-to-lookup=false repro.rs -o off.s
```

Measured (rustc 1.96 / LLVM 22.1.2), N=1024:

| variant | compile | `.s` size | jump tables |
| --- | ---: | ---: | ---: |
| default (no flag) | **32.1 s** | 0.4 MB | 0 |
| `-switch-to-lookup=true` | **0.50 s** | 0.4 MB | 0 |
| `-switch-to-lookup=false` | 80.6 s | **225 MB** | 3075 |

So `=true` is ~**64× faster to compile** than default and produces the same (good) code;
`=false` shows the worst case the default is flirting with (explicit jump tables, 225 MB asm).

Onset over N (default pipeline, `interp` shape) is **super-quadratic** (≈ N^2.8):

| N | 256 | 512 | 768 | 1024 | 1536 |
| --- | ---: | ---: | ---: | ---: | ---: |
| default compile | 0.80s | 4.66s | 13.8s | 32.4s | 99.9s |
| `.s` size | 0.10 MB | 0.20 MB | 0.30 MB | 0.40 MB | 0.61 MB |

(With `-switch-to-lookup=true`, all of these compile in ~0.5s.)

### Contrast: a single, non-inlined switch does NOT reproduce

`gen.py --shape handler` (one `op_code_to_handler` function, not inlined) becomes a lookup
table by default at every N tested (1100 → 16384, x86_64 and arm64); the flag is a no-op
there. The pathology requires the **inline-into-many-callers** multiplication — which is
why a naive minimal example fails to reproduce and why this only surfaced in Wasmi once the
opcode count was large.

## Analysis / where to look

Inputs are ideal for the transform (dense contiguous keys, a constant per case, unreachable
default), so this is a missed optimization / heuristic-tuning issue, not a correctness
constraint. The interesting part is the **interaction with the inliner**: SimplifyCFG forms
the lookup table for the standalone `dispatch`, but after `dispatch` is inlined into N
callers the per-copy switches are left for the backend (jump tables) unless the cl::opt
forces the IR-level transform. Likely places:

- SimplifyCFG `ConvertSwitchToLookupTable` / `SwitchToLookupTable` cost model and where it
  runs relative to inlining.
- `TargetTransformInfo::shouldBuildLookupTables()`.
- Why the default differs from `-switch-to-lookup=true` specifically post-inlining.

Diagnostics:

```bash
rustc -O --emit=llvm-ir --crate-type=lib repro.rs -o repro.ll
opt -passes='default<O2>' -print-after=simplify-cfg repro.ll -S 2>&1 | less
```

## LLVM vs rustc

Evidence points at **LLVM**, not rustc: rustc is not disabling the transform (its default
turns a standalone switch into a lookup table fine), the only lever that changes behavior is
the LLVM `cl::opt`, and the slow default compile reproduces under Apple clang 17 (a
different LLVM) as well — so it is not a rustc front-end quirk. File against
**llvm/llvm-project**.

## A source-level workaround (for the affected project)

Replacing the `match` with an index into a `static [Handler; N]` array removes the switch
entirely, so there is nothing for the backend to mis-lower and the result is flag-independent
(`gen.py --shape table`: 0.48 s at N=1024, no flag). This is a robust fallback for projects
that don't want to depend on a global LLVM `cl::opt`, but the underlying LLVM behavior is
still the bug being reported.

## Questions for maintainers

- Is dropping switch→lookup-table for inlined copies of a constant-returning switch
  intentional? Should the transform run (or re-run) after inlining for this shape?
- Should the decision key on case **density** (dense `0..N` is the obviously-profitable case)
  rather than effectively bailing at scale?
- Is there a supported per-function/per-TU way to request this short of the global cl::opt?
