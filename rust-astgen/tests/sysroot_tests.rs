mod common;

use crate::common::{
    TestResult, call_expr, ident_pat, method_call_expr, name_ref, path_expr, ref_expr, struct_decl,
    sysroot_ast_json,
};
use ra_ap_hir::{Crate, attach_db};
use ra_ap_ide::RootDatabase;
use rust_ast_gen::function_fullnames_gen::{
    FunctionFullNameEntry, dependency_crate_named, load_sysroot_workspace, module_full_names,
    modules_in_crate, unique_by_method_full_name, workspace_root_modules_rc,
};
use serde_json::json;
use std::fs;
use std::process::Command;
use std::rc::Rc;
use temp_dir::TempDir;

#[test]
fn std_dependent_coercions() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        r#"
use std::fmt::Display;
use std::ops::Deref;
use std::rc::Rc;

struct MyBox(String);
impl Deref for MyBox {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

struct Foo {
    field: u32,
}

fn take_str(_s: &str) {}
fn take_mut_str(_s: &mut str) {}
fn take_string(_s: &String) {}

fn through_generic<T: Deref<Target = str>>(x: &T) {
    let _ = x.len();
}

fn main() {
    let owned = String::from("x");
    take_str(&owned);

    let mut owned_mut = String::from("x");
    take_mut_str(&mut owned_mut);

    let b = MyBox(String::from("x"));
    take_string(&b);

    let r = Rc::new(1i32);
    let _n: &i32 = &r;

    let bx = Box::new(Foo { field: 1 });
    let _f = bx.field;

    let _xs: &[i32] = &[1, 2, 3];

    let disp = String::from("x");
    let _d: &dyn Display = &disp;
}
"#,
    )?;

    assert_eq!(
        ref_expr(&json, "&owned").adjustments(),
        vec![
            json!({"kind": "deref", "source": "&alloc::string::String", "target": "alloc::string::String"}),
            json!({
                "kind": "overloadedDeref",
                "source": "alloc::string::String",
                "target": "str",
                "mutable": false,
                "methodFullName": "<alloc::string::String as core::ops::deref::Deref>::deref",
            }),
            json!({"kind": "borrow", "source": "str", "target": "&str"}),
        ],
    );

    assert_eq!(
        ref_expr(&json, "&mut owned_mut").adjustments(),
        vec![
            json!({"kind": "deref", "source": "&mut alloc::string::String", "target": "alloc::string::String"}),
            json!({
                "kind": "overloadedDeref",
                "source": "alloc::string::String",
                "target": "str",
                "mutable": true,
                "methodFullName": "<alloc::string::String as core::ops::deref::DerefMut>::deref_mut",
            }),
            json!({"kind": "borrow", "source": "str", "target": "&mut str"}),
        ],
    );

    assert_eq!(
        ref_expr(&json, "&b").adjustments(),
        vec![
            json!({"kind": "deref", "source": "&rust2cpg::MyBox", "target": "rust2cpg::MyBox"}),
            json!({
                "kind": "overloadedDeref",
                "source": "rust2cpg::MyBox",
                "target": "alloc::string::String",
                "mutable": false,
                "methodFullName": "<rust2cpg::MyBox as core::ops::deref::Deref>::deref",
            }),
            json!({"kind": "borrow", "source": "alloc::string::String", "target": "&alloc::string::String"}),
        ],
    );

    assert_eq!(
        ref_expr(&json, "&r").adjustments(),
        vec![
            json!({"kind": "deref", "source": "&alloc::rc::Rc<i32, alloc::alloc::Global>", "target": "alloc::rc::Rc<i32, alloc::alloc::Global>"}),
            json!({
                "kind": "overloadedDeref",
                "source": "alloc::rc::Rc<i32, alloc::alloc::Global>",
                "target": "i32",
                "mutable": false,
                "methodFullName": "<alloc::rc::Rc<T, A> as core::ops::deref::Deref>::deref",
            }),
            json!({"kind": "borrow", "source": "i32", "target": "&i32"}),
        ],
    );

    assert_eq!(
        path_expr(&json, "x")
            .on_line("    let _ = x.len();")
            .adjustments(),
        vec![
            json!({"kind": "deref", "source": "&T", "target": "T"}),
            json!({
                "kind": "overloadedDeref",
                "source": "T",
                "target": "str",
                "mutable": false,
            }),
            json!({"kind": "borrow", "source": "str", "target": "&str"}),
        ],
    );

    assert_eq!(
        path_expr(&json, "bx")
            .on_line("    let _f = bx.field;")
            .adjustments(),
        vec![json!({
            "kind": "deref",
            "source": "alloc::boxed::Box<rust2cpg::Foo, alloc::alloc::Global>",
            "target": "rust2cpg::Foo",
        })],
    );

    assert_eq!(
        ref_expr(&json, "&[1, 2, 3]").adjustments(),
        vec![
            json!({"kind": "deref", "source": "&[i32; 3]", "target": "[i32; 3]"}),
            json!({"kind": "borrow", "source": "[i32; 3]", "target": "&[i32; 3]"}),
            json!({"kind": "cast", "source": "&[i32; 3]", "target": "&[i32]"}),
        ],
    );

    assert_eq!(
        ref_expr(&json, "&disp").adjustments(),
        vec![
            json!({"kind": "deref", "source": "&alloc::string::String", "target": "alloc::string::String"}),
            json!({"kind": "borrow", "source": "alloc::string::String", "target": "&alloc::string::String"}),
            json!({"kind": "cast", "source": "&alloc::string::String", "target": "&dyn core::fmt::Display"}),
        ],
    );

    Ok(())
}

#[test]
fn emits_names_for_vec_and_dyn_trait() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        r#"
use std::vec::Vec;

trait Sink<T> {
    fn sink(&self) -> T;
}

fn call_dyn(s: &dyn Sink<u32>) {
    let dyn_value = s.sink();
}

fn main() {
    let numbers = Vec::<u32>::new();
    let numbers_copy = numbers;
}
"#,
    )?;

    let vec_new_call = call_expr(&json, "Vec::<u32>::new()");
    assert_eq!(
        vec_new_call.type_full_name(),
        "alloc::vec::Vec<u32, alloc::alloc::Global>"
    );
    assert_eq!(
        vec_new_call.method_full_name(),
        "alloc::vec::Vec<T, alloc::alloc::Global>::new"
    );
    assert_eq!(
        name_ref(&json, "Vec")
            .on_line("let numbers = Vec::<u32>::new();")
            .type_full_name(),
        "alloc::vec::Vec<u32>"
    );
    assert_eq!(
        name_ref(&json, "new")
            .on_line("let numbers = Vec::<u32>::new();")
            .type_full_name(),
        "fn() -> alloc::vec::Vec<u32, alloc::alloc::Global>"
    );
    assert_eq!(
        ident_pat(&json, "numbers").type_full_name(),
        "alloc::vec::Vec<u32, alloc::alloc::Global>"
    );
    assert_eq!(
        name_ref(&json, "numbers").type_full_name(),
        "alloc::vec::Vec<u32, alloc::alloc::Global>"
    );

    let dyn_sink_call = method_call_expr(&json, "s.sink()").on_line("let dyn_value = s.sink();");
    assert_eq!(dyn_sink_call.type_full_name(), "u32");
    assert_eq!(dyn_sink_call.method_full_name(), "rust2cpg::Sink<T>::sink");

    Ok(())
}

#[test]
fn emits_names_for_async_fn_return_type() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        r#"
async fn f() -> i32 {
    1
}

fn main() {
    f();
}
"#,
    )?;

    assert_eq!(
        call_expr(&json, "f()").type_full_name(),
        "impl core::future::future::Future<Output = i32> + core::marker::Sized"
    );

    Ok(())
}

#[test]
fn emits_derived_trait_impls() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        r#"
#[derive(Clone)]
struct S;

fn main() {}
"#,
    )?;

    assert_eq!(
        struct_decl(&json, "#[derive(Clone)]\nstruct S;").implemented_traits(),
        vec!["core::clone::Clone"]
    );

    Ok(())
}

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
    Ok(load_sysroot_workspace(current_dir, None, vec![], false)?)
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
    let selected_modules =
        modules_in_crate(db, krate, Rc::clone(&workspace_roots)).filter(|(module, _)| {
            module.name(db).is_some_and(|name| {
                let name_str = name.display(db, edition).to_string();
                module_names.contains(&name_str.as_str())
            })
        });

    Ok(
        unique_by_method_full_name(selected_modules.flat_map(|(module, parent_is_unstable)| {
            module_full_names(db, module, Rc::clone(&workspace_roots), parent_is_unstable)
        }))
        .collect(),
    )
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
                "clone",
                "array",
                "iterator",
                "option",
                "result",
                "slice",
                "str",
                "wtf8",
                "marker",
                "net::ip_addr",
                "cell",
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
        let phantom_data =
            find_by_method_full_name(core_entries.iter().cloned(), "core::marker::PhantomData<T>");
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

const WINDOWS_TARGET: &str = "x86_64-pc-windows-msvc";

fn windows_target_installed() -> bool {
    Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).contains(WINDOWS_TARGET))
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

    let db = load_sysroot_workspace(std::env::current_dir()?, None, vec![], false)?;

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
