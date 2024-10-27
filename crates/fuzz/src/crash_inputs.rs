use core::fmt::Write as _;
use sha2::{Digest, Sha256};
use std::fs;

/// Writes `.wasm` and `.wat` files for `target` with `wasm` contents.
///
/// Returns the `hex` encoded string of the SHA-2 of the `wasm` input upon success.
///
/// # Errors
///
/// - If hex encoding fails.
/// - If converting `.wasm` to `.wat` fails.
/// - If writing the `.wasm` or `.wat` files fails.
pub fn generate_crash_inputs(target: &str, wasm: &[u8]) -> Result<String, anyhow::Error> {
    let mut sha2 = Sha256::default();
    sha2.update(wasm);
    let hash: [u8; 32] = sha2.finalize().into();
    let hash_str = hash_str(hash)?;
    let wat = wasmprinter::print_bytes(wasm)?;
    let file_path = format!("fuzz/crash-inputs/{target}/crash-{hash_str}");
    fs::write(format!("{file_path}.wasm"), wasm)?;
    fs::write(format!("{file_path}.wat"), wat)?;
    Ok(hash_str)
}

/// Returns the `hex` string of the `[u8; 32]` hash.
fn hash_str(hash: [u8; 32]) -> Result<String, anyhow::Error> {
    let mut s = String::new();
    for byte in hash {
        write!(s, "{byte:02X}")?;
    }
    Ok(s)
}
