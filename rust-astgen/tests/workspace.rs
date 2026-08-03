use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use temp_dir::TempDir;

pub type TestResult<T> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn workspace_with_crates_emits_all_files() -> TestResult<()> {
    let root = TempDir::with_prefix("rust_ast_gen_workspace_test_")?;
    let input_dir = root.child("in");
    let output_dir = root.child("out");

    let files = [
        (
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/core\", \"crates/cli\"]\n",
        ),
        (
            "crates/core/Cargo.toml",
            "[package]\nname = \"core\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        ),
        ("crates/core/src/lib.rs", "pub fn core_fn() -> u32 { 42 }"),
        (
            "crates/cli/Cargo.toml",
            "[package]\nname = \"cli\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ncore = { path = \"../core\" }\n",
        ),
        (
            "crates/cli/src/main.rs",
            "fn main() { println!(\"{}\", core::core_fn()); }",
        ),
    ];
    for (relative_path, content) in files {
        let path = input_dir.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)?;
    }
    fs::create_dir_all(&output_dir)?;

    let output = Command::new(env!("CARGO_BIN_EXE_rust_ast_gen"))
        .arg("-i")
        .arg(&input_dir)
        .arg("-o")
        .arg(&output_dir)
        .arg("--resolve-cfg")
        .env("RUST_LOG", "rust_ast_gen=debug")
        .output()?;

    let mut emitted = Vec::new();
    collect_files(&output_dir, &output_dir, &mut emitted)?;
    emitted.sort();

    let expected = vec![
        PathBuf::from("crates/cli/src/main.rs.json"),
        PathBuf::from("crates/core/src/lib.rs.json"),
    ];
    assert_eq!(
        emitted,
        expected,
        "unexpected output files\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "rust_ast_gen failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

fn collect_files(root: &Path, dir: &Path, result: &mut Vec<PathBuf>) -> TestResult<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(root, &path, result)?;
        } else {
            result.push(path.strip_prefix(root)?.to_path_buf());
        }
    }
    Ok(())
}
