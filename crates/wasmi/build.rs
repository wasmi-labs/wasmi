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
/// in the Wasmi codebase.
///
/// The `wasmi_has_tail_calls` annotation reflects whether the compilation target
/// is known to support LLVM's tail (sibling) call optimization that Wasmi's
/// default tail-call based dispatch relies upon. It is a pure target-architecture
/// property (queried via `CARGO_CFG_TARGET_ARCH`) and is used together with the
/// `auto-dispatch` crate feature to automatically fall back to the portable
/// dispatch backend on targets that do not (reliably) tail-call.
fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rustc-check-cfg=cfg(wasmi_opt_size)");
    println!("cargo::rustc-check-cfg=cfg(wasmi_opt_speed)");
    println!("cargo::rustc-check-cfg=cfg(wasmi_has_tail_calls)");
    let opt_level = env::var("OPT_LEVEL").unwrap_or_default();
    match opt_level.as_str() {
        "s" | "z" => println!("cargo::rustc-cfg=wasmi_opt_size"),
        "2" | "3" => println!("cargo::rustc-cfg=wasmi_opt_speed"),
        _ => (),
    }
    if target_has_tail_calls() {
        println!("cargo::rustc-cfg=wasmi_has_tail_calls");
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
