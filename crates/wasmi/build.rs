use std::env;

/// Emits target and opt-level dependent Rust `cfg`s used by the Wasmi codebase.
///
/// # Note
///
/// This sets the following `cfg` annotations in the Wasmi codebase:
///
/// - `wasmi_opt_size` (if `opt-level` is "s" or "z")
/// - `wasmi_opt_speed` (if `opt-level` is 2 or 3)
/// - `wasmi_has_tail_calls` (if the target is known to support LLVM tail calls)
/// - `wasmi_use_tail_calls` (if `wasmi_has_tail_calls` and the build is optimizing)
///
/// Any other optimization level (e.g. `0` or `1`) does not set
/// either of the `wasmi_opt_*` Wasmi specific `cfg` annotations.
///
/// The `wasmi_opt_*` annotations may be combined with `cfg_attr` and the
/// following built-ins:
///
/// - `#[inline]`
/// - `#[inline(never)]`
/// - `#[inline(always)]`
/// - `#[cold]`
///
/// Any other combination is forbidden as it would alter the code paths taken
/// on different optimization levels which is something we strictly want to avoid
/// in the Wasmi codebase. In particular, dispatch backend selection keys off the
/// dedicated `wasmi_use_tail_calls` annotation below instead of `wasmi_opt_*`.
///
/// The `wasmi_has_tail_calls` annotation reflects whether the compilation *target*
/// is known to support LLVM's tail (sibling) call optimization that Wasmi's
/// default tail-call based dispatch relies upon. It is a pure target-architecture
/// property (queried via `CARGO_CFG_TARGET_ARCH`).
///
/// The `wasmi_use_tail_calls` annotation is the derived decision actually consumed
/// by Wasmi's dispatch backend selection: it is set when the target supports tail
/// calls (`wasmi_has_tail_calls`) *and* the build is optimizing (`opt-level` in
/// "s", "z", 2 or 3), since LLVM does not perform the required sibling-call
/// optimization at `opt-level` 0 or 1. Together with the `auto-dispatch` crate
/// feature this drives the automatic fallback to the portable dispatch backend.
fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rustc-check-cfg=cfg(wasmi_opt_size)");
    println!("cargo::rustc-check-cfg=cfg(wasmi_opt_speed)");
    println!("cargo::rustc-check-cfg=cfg(wasmi_has_tail_calls)");
    println!("cargo::rustc-check-cfg=cfg(wasmi_use_tail_calls)");
    let opt_level = env::var("OPT_LEVEL").unwrap_or_default();
    match opt_level.as_str() {
        "s" | "z" => println!("cargo::rustc-cfg=wasmi_opt_size"),
        "2" | "3" => println!("cargo::rustc-cfg=wasmi_opt_speed"),
        _ => (),
    }
    let has_tail_calls = target_has_tail_calls();
    if has_tail_calls {
        println!("cargo::rustc-cfg=wasmi_has_tail_calls");
    }
    // Whether the build is optimizing enough for LLVM to perform sibling-call optimization.
    let is_optimizing = matches!(opt_level.as_str(), "s" | "z" | "2" | "3");
    if has_tail_calls && is_optimizing {
        println!("cargo::rustc-cfg=wasmi_use_tail_calls");
    }
}

/// Returns `true` if the compilation target is known to support LLVM tail calls.
///
/// # Note
///
/// This is a per-architecture (LLVM backend) property, not a per-OS one, so the
/// decision is made purely on `CARGO_CFG_TARGET_ARCH`. The listed architectures
/// are those verified to lower tail (sibling) calls by LLVM. Every unlisted
/// architecture conservatively returns `false` so that Wasmi safely falls back
/// to its portable dispatch backend instead of risking a native stack overflow.
///
/// The two notable exceptions to broad LLVM tail-call support are:
///
/// - `powerpc`/`powerpc64`: cannot tail-call due to TOC pointer restoration.
/// - `wasm32`/`wasm64`: can only tail-call with the `tail-call` Wasm feature.
fn target_has_tail_calls() -> bool {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_features = env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
    match target_arch.as_str() {
        "x86" | "x86_64" | "arm" | "aarch64" | "riscv32" | "riscv64" | "loongarch64" | "s390x" => {
            true
        }
        // Wasm can only tail-call if the `tail-call` Wasm proposal is enabled.
        "wasm32" | "wasm64" => target_features
            .split(',')
            .any(|feature| feature == "tail-call"),
        _ => false,
    }
}
