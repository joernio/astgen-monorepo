mod common;

use crate::common::TestResult;
use ra_ap_hir::attach_db;
use ra_ap_ide::RootDatabase;
use rust_ast_gen::function_fullnames_gen::{
    FunctionFullNameEntry, dependency_crate_named, load_sysroot_workspace, module_full_names,
    modules_in_crate, unique_by_method_full_name, workspace_root_modules_rc,
};
use std::rc::Rc;

fn find_by_method_full_name(
    entries: impl IntoIterator<Item = FunctionFullNameEntry>,
    method_full_name: &str,
) -> Option<FunctionFullNameEntry> {
    entries
        .into_iter()
        .find(|entry| entry.method_full_name == method_full_name)
}

fn load_sysroot_only_db() -> TestResult<RootDatabase> {
    // cargo test guarantees cwd is the package root, so we can use it directly.
    // We only need the sysroot crates (like core), not the workspace crates.
    let current_dir = std::env::current_dir()?;
    Ok(load_sysroot_workspace(current_dir)?)
}

fn entries_in_dependency_modules(
    db: &RootDatabase,
    crate_name: &str,
    module_names: &[&str],
) -> TestResult<Vec<FunctionFullNameEntry>> {
    let workspace_roots = workspace_root_modules_rc(db);
    let krate = dependency_crate_named(db, crate_name).ok_or_else(|| {
        format!("dependency crate `{crate_name}` not found in sysroot workspace")
    })?;

    let edition = krate.edition(db);
    let selected_modules = modules_in_crate(db, krate).filter(|module| {
        module.name(db).is_some_and(|name| {
            let name_str = name.display(db, edition).to_string();
            module_names.contains(&name_str.as_str())
        })
    });

    Ok(unique_by_method_full_name(selected_modules.flat_map(|module| {
        module_full_names(db, module, Rc::clone(&workspace_roots))
    }))
    .collect())
}

#[test]
fn dependency_crate_function_fullnames() -> TestResult<()> {
    let db = load_sysroot_only_db()?;
    attach_db(&db, || {
        // Use only the exact module names that contain our test methods
        let core_entries = entries_in_dependency_modules(
            &db,
            "core",
            &["clone", "array", "iterator", "option", "result", "slice", "str"],
        )?;

            // Test enum variant constructor (Option::Some)
            let option_some = FunctionFullNameEntry {
                method_full_name: "core::option::Option<T>::Some".to_owned(),
                has_self_receiver: false,
                is_trait_impl: false,
                is_trait_method_def: false,
                is_nightly_only: false,
            };
            assert_eq!(
                find_by_method_full_name(core_entries.clone(), &option_some.method_full_name),
                Some(option_some),
                "should find Option::Some enum variant constructor"
            );

            // Test inherent method with self receiver (Result::unwrap_or_else)
            let result_unwrap_or_else = FunctionFullNameEntry {
                method_full_name: "core::result::Result<T, E>::unwrap_or_else<F>".to_owned(),
                has_self_receiver: true,
                is_trait_impl: false,
                is_trait_method_def: false,
                is_nightly_only: false,
            };
            assert_eq!(
                find_by_method_full_name(core_entries.clone(), &result_unwrap_or_else.method_full_name),
                Some(result_unwrap_or_else),
                "should find Result::unwrap_or_else inherent method"
            );

            // Test inherent method without self receiver (Option::unwrap_or)
            let option_unwrap_or = FunctionFullNameEntry {
                method_full_name: "core::option::Option<T>::unwrap_or".to_owned(),
                has_self_receiver: true,
                is_trait_impl: false,
                is_trait_method_def: false,
                is_nightly_only: false,
            };
            assert_eq!(
                find_by_method_full_name(core_entries.clone(), &option_unwrap_or.method_full_name),
                Some(option_unwrap_or),
                "should find Option::unwrap_or inherent method"
            );

            // Test trait method definitions
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
            assert!(
                find_by_method_full_name(core_entries.clone(), &iterator_next.method_full_name)
                    == Some(iterator_next.clone())
                    || find_by_method_full_name(core_entries.clone(), &clone_clone.method_full_name)
                        == Some(clone_clone),
                "should find trait method definitions like Iterator::next or Clone::clone"
            );

            // Test trait impl (array Clone)
            let array_clone = FunctionFullNameEntry {
                method_full_name: "<[T; N] as core::clone::Clone>::clone".to_owned(),
                has_self_receiver: true,
                is_trait_impl: true,
                is_trait_method_def: false,
                is_nightly_only: false,
            };
            assert_eq!(
                find_by_method_full_name(core_entries.clone(), &array_clone.method_full_name),
                Some(array_clone),
                "should find trait impl like array's Clone::clone"
            );

            // Test primitive inherent method (str::as_bytes)
            let str_as_bytes = FunctionFullNameEntry {
                method_full_name: "str::as_bytes".to_owned(),
                has_self_receiver: true,
                is_trait_impl: false,
                is_trait_method_def: false,
                is_nightly_only: false,
            };
            assert_eq!(
                find_by_method_full_name(core_entries.clone(), &str_as_bytes.method_full_name),
                Some(str_as_bytes),
                "should find primitive inherent methods like str::as_bytes"
            );

            // Test slice inherent method
            let slice_len = FunctionFullNameEntry {
                method_full_name: "[T]::len".to_owned(),
                has_self_receiver: true,
                is_trait_impl: false,
                is_trait_method_def: false,
                is_nightly_only: false,
            };
            assert_eq!(
                find_by_method_full_name(core_entries.clone(), &slice_len.method_full_name),
                Some(slice_len),
                "should find slice inherent methods like [T]::len"
            );

            // Verify workspace items are excluded (our own test crate functions shouldn't appear)
            assert_eq!(
                find_by_method_full_name(
                    core_entries.clone(),
                    "rust_ast_gen::function_fullnames_gen::run"
                ),
                None,
                "workspace functions should be excluded from dependency entries"
            );

            Ok(())
    })
}
