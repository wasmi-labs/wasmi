#![allow(warnings)]

use std::{
    env,
    fs,
    path::{Path, PathBuf},
};

#[path = "build/mod.rs"]
mod build;

fn main() {
    watch_dir_recursively(Path::new("build"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    build::generate_code(&out_dir).unwrap()
}

fn watch_dir_recursively(path: &Path) {
    if !path.is_dir() {
        return;
    }
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) => panic!("failed to read directory: {error}"),
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => panic!("failed to read directory entry: {error}"),
        };
        let path = entry.path();
        if path.is_file() {
            println!("cargo:rerun-if-changed={}", path.display());
        }
        watch_dir_recursively(&path);
    }
}
