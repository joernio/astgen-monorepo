//! `ra_ap_syntax` publishes `rust.ungram` alongside its source-code in the
//! cargo registry. This scripts copies that file from the local/resolved
//! `ra_ap_syntax` version so that our Scala case classes match the AST shape.
//!
//! Parses `Cargo.lock` to find which ra_ap_syntax version we are using
//! and then searches the cargo registry for the `rust.ungram` file
//! inside the ra_ap_syntax-{version} subdirectory.

use serde::Deserialize;
use std::env;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=rust.ungram");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let lockfile_path = manifest_dir.join("Cargo.lock");
    let destination_path = manifest_dir.join("rust.ungram");

    let ra_ap_syntax_version = resolved_ra_ap_syntax_version(&lockfile_path)?;
    let registry_src_path = cargo_home()?.join("registry").join("src");

    let Some(source_path) = find_rust_ungram_file(&registry_src_path, &ra_ap_syntax_version)?
    else {
        println!(
            "cargo:warning=Could not find ra_ap_syntax-{ra_ap_syntax_version}/rust.ungram in cargo registry {}",
            registry_src_path.display()
        );
        return Ok(());
    };

    if rewrite_if_different(&source_path, &destination_path)? {
        println!(
            "cargo:warning=Updated rust.ungram from {}",
            source_path.display()
        );
    }

    Ok(())
}

fn resolved_ra_ap_syntax_version(
    lockfile_path: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let lockfile_contents = std::fs::read_to_string(lockfile_path)?;
    let lockfile: Lockfile = toml::from_str(&lockfile_contents)?;

    lockfile
        .package
        .into_iter()
        .find(|pkg| pkg.name == "ra_ap_syntax")
        .map(|pkg| pkg.version.to_string())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "ra_ap_syntax version not found in Cargo.lock",
            )
            .into()
        })
}

fn find_rust_ungram_file(
    registry_src_path: &Path,
    version: &str,
) -> std::io::Result<Option<PathBuf>> {
    let crate_dir_name = format!("ra_ap_syntax-{version}");

    if !registry_src_path.is_dir() {
        return Ok(None);
    }

    for entry in std::fs::read_dir(registry_src_path)? {
        let entry = entry?;
        let candidate = entry.path().join(&crate_dir_name).join("rust.ungram");
        if candidate.is_file() {
            return Ok(Some(candidate));
        }
    }

    Ok(None)
}

fn cargo_home() -> Result<PathBuf, env::VarError> {
    match env::var("CARGO_HOME") {
        Ok(path) => Ok(PathBuf::from(path)),
        Err(_) => env::var("HOME").map(|home| PathBuf::from(home).join(".cargo")),
    }
}

fn rewrite_if_different(source: &Path, destination: &Path) -> std::io::Result<bool> {
    let source_bytes = std::fs::read(source)?;
    let existing_bytes = std::fs::read(destination)?;

    if source_bytes == existing_bytes {
        return Ok(false);
    }

    std::fs::write(destination, source_bytes)?;
    Ok(true)
}

#[derive(Debug, Deserialize)]
struct Lockfile {
    package: Vec<Package>,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    version: String,
}
