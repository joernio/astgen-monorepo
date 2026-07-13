mod common;

use crate::common::TestResult;
use ra_ap_hir::attach_db;
use ra_ap_ide::RootDatabase;
use rust_ast_gen::function_fullnames_gen::{
    FunctionFullNameEntry, dependency_crate_named, load_sysroot_workspace, module_full_names,
    modules_in_crate, unique_by_method_full_name, workspace_root_modules_rc,
};
use std::fs;
use std::rc::Rc;
use temp_dir::TempDir;

fn find_by_method_full_name(
    entries: impl IntoIterator<Item = FunctionFullNameEntry>,
    method_full_name: &str,
) -> Option<FunctionFullNameEntry> {
    entries
        .into_iter()
        .find(|entry| entry.method_full_name == method_full_name)
}

fn with_workspace_db(
    crate_name: &str,
    file_code_pairs: &[(&str, &str)],
    test: impl FnOnce(&RootDatabase) -> TestResult<()>,
) -> TestResult<()> {
    let root = TempDir::with_prefix("rust_ast_gen_function_fullnames_test_")?;

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

    for (relative_path, content) in file_code_pairs {
        let path = root.path().join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)?;
    }

    let db = load_sysroot_workspace(root.path().to_path_buf())?;
    attach_db(&db, || test(&db)).map_err(Into::into)
}

fn entries_in_dependency_crate(
    db: &RootDatabase,
    crate_name: &str,
) -> TestResult<Vec<FunctionFullNameEntry>> {
    let workspace_roots = workspace_root_modules_rc(db);
    let krate = dependency_crate_named(db, crate_name).ok_or_else(|| {
        format!("dependency crate `{crate_name}` not found in sysroot workspace")
    })?;

    Ok(unique_by_method_full_name(modules_in_crate(db, krate).flat_map(
        |module| module_full_names(db, module, Rc::clone(&workspace_roots)),
    ))
    .collect())
}

#[test]
fn dumps_public_dependency_callables_and_excludes_workspace_items() -> TestResult<()> {
    with_workspace_db(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct LocalTuple(i32);

fn workspace_fn() {}

fn main() {
}
"#,
        )],
        |db| {
            let vec_new = FunctionFullNameEntry {
                method_full_name: "alloc::vec::Vec<T, alloc::alloc::Global>::new".to_owned(),
                has_self_receiver: false,
                is_trait_impl: false,
                is_trait_method_def: false,
                is_nightly_only: false,
            };
            assert_eq!(
                find_by_method_full_name(entries_in_dependency_crate(db, "alloc")?, &vec_new.method_full_name),
                Some(vec_new)
            );

            let option_some = FunctionFullNameEntry {
                method_full_name: "core::option::Option<T>::Some".to_owned(),
                has_self_receiver: false,
                is_trait_impl: false,
                is_trait_method_def: false,
                is_nightly_only: false,
            };
            assert_eq!(
                find_by_method_full_name(entries_in_dependency_crate(db, "core")?, &option_some.method_full_name),
                Some(option_some)
            );

            let result_unwrap_or_else = FunctionFullNameEntry {
                method_full_name: "core::result::Result<T, E>::unwrap_or_else<F>".to_owned(),
                has_self_receiver: true,
                is_trait_impl: false,
                is_trait_method_def: false,
                is_nightly_only: false,
            };
            assert_eq!(
                find_by_method_full_name(
                    entries_in_dependency_crate(db, "core")?,
                    &result_unwrap_or_else.method_full_name
                ),
                Some(result_unwrap_or_else)
            );

            let iterator_next = FunctionFullNameEntry {
                method_full_name: "core::iter::traits::iterator::Iterator::next".to_owned(),
                has_self_receiver: true,
                is_trait_impl: false,
                is_trait_method_def: true,
                is_nightly_only: false,
            };
            let clone_clone = FunctionFullNameEntry {
                method_full_name: "core::clone::Clone::clone".to_owned(),
                has_self_receiver: true,
                is_trait_impl: false,
                is_trait_method_def: true,
                is_nightly_only: false,
            };
            let core_entries = entries_in_dependency_crate(db, "core")?;
            assert!(
                find_by_method_full_name(core_entries.clone(), &iterator_next.method_full_name)
                    == Some(iterator_next.clone())
                    || find_by_method_full_name(core_entries, &clone_clone.method_full_name)
                        == Some(clone_clone),
                "expected Iterator::next or Clone::clone in core dependency dump"
            );

            let alloc_entries = entries_in_dependency_crate(db, "alloc")?;
            assert_eq!(
                find_by_method_full_name(alloc_entries.clone(), "rust2cpg::workspace_fn"),
                None,
                "workspace free function should be excluded"
            );
            assert_eq!(
                find_by_method_full_name(alloc_entries, "rust2cpg::LocalTuple"),
                None,
                "workspace tuple struct ctor should be excluded"
            );

            Ok(())
        },
    )
}

#[test]
fn includes_std_trait_impl_and_inherent_callables() -> TestResult<()> {
    with_workspace_db("rust2cpg", &[("src/main.rs", "fn main() {}\n")], |db| {
        let string_deref = FunctionFullNameEntry {
            method_full_name: "<alloc::string::String as core::ops::deref::Deref>::deref".to_owned(),
            has_self_receiver: true,
            is_trait_impl: true,
            is_trait_method_def: false,
            is_nightly_only: false,
        };
        assert_eq!(
            find_by_method_full_name(
                entries_in_dependency_crate(db, "alloc")?,
                &string_deref.method_full_name
            ),
            Some(string_deref)
        );

        let array_clone = FunctionFullNameEntry {
            method_full_name: "<[T; N] as core::clone::Clone>::clone".to_owned(),
            has_self_receiver: true,
            is_trait_impl: true,
            is_trait_method_def: false,
            is_nightly_only: false,
        };
        assert_eq!(
            find_by_method_full_name(
                entries_in_dependency_crate(db, "core")?,
                &array_clone.method_full_name
            ),
            Some(array_clone)
        );

        let str_as_bytes = FunctionFullNameEntry {
            method_full_name: "str::as_bytes".to_owned(),
            has_self_receiver: true,
            is_trait_impl: false,
            is_trait_method_def: false,
            is_nightly_only: false,
        };
        assert_eq!(
            find_by_method_full_name(
                entries_in_dependency_crate(db, "core")?,
                &str_as_bytes.method_full_name
            ),
            Some(str_as_bytes)
        );

        Ok(())
    })
}
