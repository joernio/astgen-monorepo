mod common;

use crate::common::TestResult;
use ra_ap_hir::{Crate, attach_db};
use ra_ap_ide::RootDatabase;
use rust_ast_gen::function_fullnames_gen::{
    FunctionFullNameEntry, load_sysroot_workspace, module_full_names, modules_in_crate,
    unique_by_method_full_name, workspace_root_modules_rc,
};
use std::fs;
use std::rc::Rc;
use temp_dir::TempDir;

fn write_feature_fixture(root: &temp_dir::TempDir) -> TestResult<()> {
    fs::write(
        root.path().join("Cargo.toml"),
        r#"[package]
name = "feature_root"
version = "0.1.0"
edition = "2021"

[features]
enabled = ["gated_dep/enabled"]

[dependencies]
gated_dep = { path = "gated_dep" }
"#,
    )?;
    fs::create_dir_all(root.path().join("src"))?;
    fs::write(root.path().join("src/lib.rs"), "pub use gated_dep::*;")?;
    fs::create_dir_all(root.path().join("gated_dep/src"))?;
    fs::write(
        root.path().join("gated_dep/Cargo.toml"),
        r#"[package]
name = "gated_dep"
version = "0.1.0"
edition = "2021"

[features]
enabled = []
"#,
    )?;
    fs::write(
        root.path().join("gated_dep/src/lib.rs"),
        r#"
pub fn always() {}

#[cfg(feature = "enabled")]
pub fn gated() {}
"#,
    )?;
    Ok(())
}

fn load_fixture_db(root: &TempDir, features: Vec<String>) -> TestResult<RootDatabase> {
    Ok(load_sysroot_workspace(
        root.path().to_path_buf(),
        None,
        features,
        false,
    )?)
}

fn crate_named(db: &RootDatabase, name: &str) -> Option<Crate> {
    Crate::all(db).into_iter().find(|krate| {
        krate
            .display_name(db)
            .is_some_and(|crate_name| crate_name.as_str() == name)
    })
}

fn entries_for_crate(db: &RootDatabase, crate_name: &str) -> Vec<FunctionFullNameEntry> {
    let workspace_roots = workspace_root_modules_rc(db);
    let krate = crate_named(db, crate_name).expect("crate should exist in loaded workspace");

    unique_by_method_full_name(
        modules_in_crate(db, krate, Rc::clone(&workspace_roots)).flat_map(
            |(module, parent_is_unstable)| {
                module_full_names(db, module, Rc::clone(&workspace_roots), parent_is_unstable)
            },
        ),
    )
    .collect()
}

fn has_method(entries: &[FunctionFullNameEntry], method_full_name: &str) -> bool {
    entries
        .iter()
        .any(|entry| entry.method_full_name == method_full_name)
}

#[test]
fn feature_gated_dependency_function_is_included_when_enabled() -> TestResult<()> {
    let root = TempDir::with_prefix("rust_ast_gen_feature_test_")?;
    write_feature_fixture(&root)?;
    let db = load_fixture_db(&root, vec!["enabled".to_owned()])?;

    attach_db(&db, || {
        let entries = entries_for_crate(&db, "gated_dep");
        assert!(
            has_method(&entries, "gated_dep::always"),
            "expected always-visible dependency function, got: {:?}",
            entries
                .iter()
                .map(|e| &e.method_full_name)
                .collect::<Vec<_>>()
        );
        assert!(
            has_method(&entries, "gated_dep::gated"),
            "expected feature-gated dependency function when feature is enabled"
        );
        Ok(())
    })
}

#[test]
fn feature_gated_dependency_function_is_excluded_without_feature() -> TestResult<()> {
    let root = TempDir::with_prefix("rust_ast_gen_feature_test_")?;
    write_feature_fixture(&root)?;
    let db = load_fixture_db(&root, vec![])?;

    attach_db(&db, || {
        let entries = entries_for_crate(&db, "gated_dep");
        assert!(has_method(&entries, "gated_dep::always"));
        assert!(
            !has_method(&entries, "gated_dep::gated"),
            "feature-gated dependency function should be excluded without enabled feature"
        );
        Ok(())
    })
}
