mod common;

use crate::common::TestResult;
use ra_ap_hir::attach_db;
use ra_ap_ide::RootDatabase;
use rust_ast_gen::function_fullnames_gen::{
    dependency_crate_named, load_sysroot_workspace, modules_in_crate, workspace_root_modules_rc,
};
use std::process::Command;

const WINDOWS_TARGET: &str = "x86_64-pc-windows-msvc";

fn windows_target_installed() -> bool {
    Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout).contains(WINDOWS_TARGET)
        })
        .unwrap_or(false)
}

fn std_module_names(db: &RootDatabase) -> Vec<String> {
    let workspace_roots = workspace_root_modules_rc(db);
    let krate = dependency_crate_named(db, "std")
        .expect("std crate should be available when sysroot is loaded");
    let edition = krate.edition(db);

    modules_in_crate(db, krate, workspace_roots)
        .filter_map(|(module, _)| {
            module
                .name(db)
                .map(|name| name.display(db, edition).to_string())
        })
        .collect()
}

fn std_has_windows_os_modules(module_names: &[String]) -> bool {
    module_names.iter().any(|name| name.contains("windows"))
}

#[test]
fn host_target_does_not_include_windows_os_modules_on_non_windows_hosts() -> TestResult<()> {
    if cfg!(windows) {
        return Ok(());
    }

    let db = load_sysroot_workspace(
        std::env::current_dir()?,
        None,
        vec![],
        false,
    )?;

    attach_db(&db, || {
        let module_names = std_module_names(&db);
        assert!(
            !std_has_windows_os_modules(&module_names),
            "host-target std should not include windows os modules on non-windows hosts, but found: {module_names:?}"
        );
        Ok(())
    })
}

#[test]
fn windows_target_includes_windows_os_modules() -> TestResult<()> {
    if !windows_target_installed() {
        eprintln!("skipping: {WINDOWS_TARGET} target not installed");
        return Ok(());
    }

    let db = load_sysroot_workspace(
        std::env::current_dir()?,
        Some(WINDOWS_TARGET.to_owned()),
        vec![],
        false,
    )?;

    attach_db(&db, || {
        let module_names = std_module_names(&db);
        assert!(
            std_has_windows_os_modules(&module_names),
            "windows-target std should include windows os modules, but found: {module_names:?}"
        );
        Ok(())
    })
}
