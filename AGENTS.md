# AGENTS.md

Guidance for AI agents working on Wasmi, and for the humans using them.

Wasmi follows the spirit of the [Rust project's LLM policy](https://forge.rust-lang.org/policies/llm-usage.html):
use LLMs to learn, understand, check and write *better*, not to hand off the hard problems.
The human is the contributor; the agent is a tool.

## Humans in the loop

- A human owns every contribution and must understand every line of it. "The LLM wrote it" is no excuse.
- Communication is between humans. Agents never open, edit or comment on issues, PRs or reviews, and never
  contact anyone. Agents may draft; the human edits, shortens and posts under their own name.
- Disclose LLM use in the PR description: which parts were generated.
- Discuss features and larger changes in an issue first, see [CONTRIBUTING.md](CONTRIBUTING.md).
- Do not mass-produce PRs or reports.

## Bugs and security

- Verify before reporting: reproduce on latest `main` with a minimal `.wat`/`.wasm` and check for duplicates.
- Keep reports short: symptom, cause (with a permalink), proposed fix.
- Security issues go private, one per report, see [SECURITY.md](SECURITY.md). The human verifies the impact;
  unverified LLM findings are not reports.

## Working style

- Never guess. Measure, or ask the human. Prove compiler or lint claims by compiling, perf claims by measuring
  against a baseline.
- Small steps: work in small, separately reviewable steps.
- Minimal diffs: no drive-by refactors, renames or reformatting. Separate concerns go into separate PRs.
- Match the surrounding code. Keep docs and comments terse; unlike code, they silently drift out of sync.
- Commits are made and signed by the human. No AI co-authorship (`Co-Authored-By` or similar):
  every commit and code change is owned by a human.
- Prefer rebasing onto `main` over merging it into a branch.

## Code

- Library crates must build with stable Rust (edition 2024, MSRV as `rust-version` in `Cargo.toml`).
  `wasmi`, `wasmi_core`, `wasmi_ir`, `wasmi_collections` and `wasmi_c_api_impl` must also build without `std`.
- Wasmi mirrors Wasmtime's API and behaviour on a best-effort basis. Follow Wasmtime; diverge only with a reason
  stated in the PR.
- No new dependencies without discussing them in an issue first.

## Tests

- Prefer `.wast` tests. New ones go in `crates/wast/tests/local/` and are registered as `local_*` in
  `crates/wast/tests/mod.rs`. Use Rust tests only when `.wast` cannot express the case.
- Existing `.wast` tests define the expected behaviour: fix Wasmi, never the test.
- Never edit `crates/wast/tests/spec` (upstream testsuite) or the `crates/wast/tests/wasmi` submodule.
- While iterating, run only the relevant test files, e.g. `cargo test -p wasmi_wast local_if`, to keep the
  edit-compile-test loop fast.
- A regression test must fail without the fix. The value stack is reused across invocations, so use distinct
  values per assertion.
- Use as little instance state as possible: params over globals over memories/tables.
- Cross-check new `.wast` tests with `wasmtime wast`.
- Name tests after the behaviour they check.

## Soundness and performance

- Prefer safe Rust. Use `unsafe` only where it yields a measurable performance gain on a performance-critical
  path, which is primarily WebAssembly execution.
- `unsafe` code must pass Miri; ideally under both Stacked and Tree Borrows.
- Avoid unnecessary heap allocations; reuse existing buffers where possible and where it makes sense.
- No locks or atomics on the execution hot path.
- Executor handlers must tail-call; a missed sibling call leaks a native stack frame per op
  ([#1942](https://github.com/wasmi-labs/wasmi/issues/1942)).
- `Op`/`OpCode` are generated from `crates/ir/build/isa.rs`; edit that, not the output.
- Optimizations need benchmarks against `main`; regressions need a strong reason.

## Checks

```sh
git submodule update --init --recursive
cargo test --workspace
cargo +nightly fmt
cargo +nightly clippy --workspace --all-features -- -D warnings
```

CI runs more (feature combos, `no_std`, MSRV, docs, Miri), see `.github/workflows/rust.yml`.

For `unsafe` changes run Miri. CI checks Stacked Borrows only; ideally also run it with Tree Borrows:

```sh
cargo +nightly miri nextest run --target x86_64-unknown-linux-gnu --lib --workspace --exclude wasmi_c_api_macros
MIRIFLAGS="-Zmiri-tree-borrows" cargo +nightly miri nextest run --target x86_64-unknown-linux-gnu --lib --workspace --exclude wasmi_c_api_macros
```

The full benchmark suite is very slow; run a subset. The Rust-based cases and fibonacci are the minimal baseline:

```sh
F="execute/(sort|prime_sieve|matrix_mul|nbody|argon2|tiny_keccak|mandelbrot|spectralnorm|compression|word_count|json_parse|reverse_complement|regex_redux|fibonacci)"
cargo bench --bench benches -p wasmi -- --save-baseline main "$F"  # on main
cargo bench --bench benches -p wasmi -- --baseline main "$F"       # on your branch
```
