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
    let krate = dependency_crate_named(db, crate_name)
        .ok_or_else(|| format!("dependency crate `{crate_name}` not found in sysroot workspace"))?;

    let edition = krate.edition(db);
    let selected_modules = modules_in_crate(db, krate, Rc::clone(&workspace_roots)).filter(|(module, _)| {
        module.name(db).is_some_and(|name| {
            let name_str = name.display(db, edition).to_string();
            module_names.contains(&name_str.as_str())
        })
    });

    Ok(unique_by_method_full_name(
        selected_modules
            .flat_map(|(module, parent_is_unstable)| module_full_names(db, module, Rc::clone(&workspace_roots), parent_is_unstable)),
    )
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
            &[
                "clone", "array", "iterator", "option", "result", "slice", "str", "wtf8",
                "marker", "net::ip_addr", "cell",
            ],
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
            find_by_method_full_name(
                core_entries.clone(),
                &result_unwrap_or_else.method_full_name
            ),
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

        // Test that items in unstable modules are marked as nightly_only
        // wtf8 is a public module but marked #![unstable]
        let wtf8_items: Vec<_> = core_entries
            .iter()
            .filter(|entry| entry.method_full_name.contains("wtf8"))
            .collect();

        assert!(
            !wtf8_items.is_empty(),
            "Should find items in the wtf8 module"
        );

        for entry in wtf8_items {
            assert!(
                entry.is_nightly_only,
                "Item {} in unstable module wtf8 should be marked as nightly_only",
                entry.method_full_name
            );
        }

        // Test that items in unstable impl blocks are marked as nightly_only
        // Cell::get_cloned is in an impl block marked #[unstable(feature = "cell_get_cloned")]
        let cell_get_cloned = find_by_method_full_name(
            core_entries.iter().cloned(),
            "core::cell::Cell<T>::get_cloned",
        );
        assert!(
            cell_get_cloned.is_some(),
            "Should find Cell::get_cloned method"
        );
        assert!(
            cell_get_cloned.unwrap().is_nightly_only,
            "Cell::get_cloned in unstable impl block should be marked as nightly_only"
        );

        // Test constructor field visibility checks
        // Option::Some has all public fields and should have a constructor
        let option_some = find_by_method_full_name(
            core_entries.iter().cloned(),
            "core::option::Option<T>::Some",
        );
        assert!(
            option_some.is_some(),
            "Option::Some should have a constructor (tuple variant with public field)"
        );

        // Option::None is a unit variant and should NOT have a constructor
        let option_none = find_by_method_full_name(
            core_entries.iter().cloned(),
            "core::option::Option<T>::None",
        );
        assert!(
            option_none.is_none(),
            "Option::None should not have a constructor (unit variant)"
        );

        // Result::Ok has a public field and should have a constructor
        let result_ok = find_by_method_full_name(
            core_entries.iter().cloned(),
            "core::result::Result<T, E>::Ok",
        );
        assert!(
            result_ok.is_some(),
            "Result::Ok should have a constructor (tuple variant with public field)"
        );

        // Result::Err has a public field and should have a constructor
        let result_err = find_by_method_full_name(
            core_entries.iter().cloned(),
            "core::result::Result<T, E>::Err",
        );
        assert!(
            result_err.is_some(),
            "Result::Err should have a constructor (tuple variant with public field)"
        );

        // PhantomData is a unit struct and should NOT have a constructor
        let phantom_data = find_by_method_full_name(
            core_entries.iter().cloned(),
            "core::marker::PhantomData<T>",
        );
        assert!(
            phantom_data.is_none(),
            "PhantomData should not have a constructor (unit struct)"
        );

        // IpAddr::V4 has private fields and should NOT have a constructor
        let ipaddr_v4 = find_by_method_full_name(
            core_entries.iter().cloned(),
            "core::net::ip_addr::IpAddr::V4",
        );
        assert!(
            ipaddr_v4.is_none(),
            "IpAddr::V4 should not have a constructor (tuple variant with private fields), but found: {:?}",
            ipaddr_v4
        );

        // IpAddr::V6 has private fields and should NOT have a constructor
        let ipaddr_v6 = find_by_method_full_name(
            core_entries.iter().cloned(),
            "core::net::ip_addr::IpAddr::V6",
        );
        assert!(
            ipaddr_v6.is_none(),
            "IpAddr::V6 should not have a constructor (tuple variant with private fields), but found: {:?}",
            ipaddr_v6
        );

        Ok(())
    })
}
