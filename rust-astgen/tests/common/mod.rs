//! Test fixture that runs `rust_ast_gen` (via cargo) against a temporary directory and
//! parses back the generated JSON. Currently only one crate is supported.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use temp_dir::TempDir;

pub type TestResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn ast_json(crate_name: &str, source: &str) -> TestResult<Value> {
    // NB: automatically deleted when dropped
    let root = TempDir::with_prefix("rust_ast_gen_integration_test_")?;
    let output_dir = root.child("out");
    let source_dir = Path::new("src");
    let source_path = source_dir.join("main.rs");

    fs::create_dir_all(root.path().join(source_dir))?;
    fs::write(
        root.child("Cargo.toml"),
        format!(
            r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2021"
"#
        ),
    )?;
    fs::write(root.path().join(&source_path), source)?;

    let output = Command::new(env!("CARGO_BIN_EXE_rust_ast_gen"))
        .arg("-i")
        .arg(root.path())
        .arg("-o")
        .arg(&output_dir)
        .output()?;

    assert!(
        output.status.success(),
        "rust_ast_gen failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(serde_json::from_str(&fs::read_to_string(
        output_dir.join("src").join("main.rs.json"),
    )?)?)
}
