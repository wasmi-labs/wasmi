use std::env;

/// Emits opt-level dependent Rust `cfg`s to steer `#[inline]` annotations.
///
/// # Note
///
/// This sets the following `cfg` annotations in the Wasmi codebase:
///
/// - `wasmi_opt_size` (if `opt-level` is "s" or "z")
/// - `wasmi_opt_speed` (if `opt-level` is 2 or 3)
///
/// Any other optimization level (e.g. `0` or `1`) does not set
/// either of the above Wasmi specific `cfg` annotations.
///
/// Users may combine the above annotations with `cfg_attr` the following built-ins:
///
/// - `#[inline]`
/// - `#[inline(never)]`
/// - `#[inline(always)]`
/// - `#[cold]`
///
/// Any other combination is forbidden as it would alter the code paths taken
/// on different optimization levels which is something we strictly want to avoid
/// in the Wasmi codebase.
fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rustc-check-cfg=cfg(wasmi_opt_size)");
    println!("cargo::rustc-check-cfg=cfg(wasmi_opt_speed)");
    let opt_level = env::var("OPT_LEVEL").unwrap_or_default();
    match opt_level.as_str() {
        "s" | "z" => println!("cargo::rustc-cfg=wasmi_opt_size"),
        "2" | "3" => println!("cargo::rustc-cfg=wasmi_opt_speed"),
        _ => (),
    }
}
