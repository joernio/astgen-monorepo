use crate::common::{
    TestResult, call_expr, fn_decl, no_sysroot_ast_json, no_sysroot_ast_json_generated,
    nodes_by_kind, sysroot_ast_json, sysroot_ast_json_generated,
};
use rust_ast_gen::config::RustAstGenConfig;
use rust_ast_gen::json_gen;
use std::fs;
use std::path::Path;
use temp_dir::TempDir;

mod common;

#[test]
fn emits_crate_name() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "my_crate_name",
        &[("src/lib.rs", "fn foo() {}")],
        "src/lib.rs",
    )?;

    // crateName being in the wrapped AST
    assert_eq!(json["crateName"].as_str(), Some("my_crate_name"));
    assert_eq!(json["modulePath"].as_str(), None);

    Ok(())
}

#[test]
fn qualifies_build_script_crate_name_with_package_name() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "my-package",
        &[("src/lib.rs", ""), ("build.rs", "fn main() {}")],
        "build.rs",
    )?;

    assert_eq!(json["crateName"].as_str(), Some("my_package_build_script"));
    assert_eq!(
        fn_decl(&json, "fn main() {}").method_full_name(),
        "my_package_build_script::main"
    );

    Ok(())
}

#[test]
fn crate_root_has_no_module_path() -> TestResult<()> {
    let files = [("src/main.rs", "fn foo() {}")];
    let json = no_sysroot_ast_json("my_crate", &files, "src/main.rs")?;

    assert_eq!(json["modulePath"].as_str(), None);
    assert_eq!(json["crateName"].as_str(), Some("my_crate"));
    Ok(())
}

#[test]
fn nested_sub_module_file_has_module_path() -> TestResult<()> {
    let files = [
        ("src/main.rs", "mod foo;\nfn main() {}\n"),
        ("src/foo.rs", "pub mod bar;\n"),
        ("src/foo/bar.rs", "pub fn baz() {}\n"),
    ];
    let json = no_sysroot_ast_json("my_crate", &files, "src/foo/bar.rs")?;
    assert_eq!(json["modulePath"].as_str(), Some("foo::bar"));
    assert_eq!(json["crateName"].as_str(), Some("my_crate"));

    let json = no_sysroot_ast_json("my_crate", &files, "src/foo.rs")?;
    assert_eq!(json["modulePath"].as_str(), Some("foo"));
    assert_eq!(json["crateName"].as_str(), Some("my_crate"));

    let json = no_sysroot_ast_json("my_crate", &files, "src/main.rs")?;
    assert_eq!(json["modulePath"].as_str(), None);
    assert_eq!(json["crateName"].as_str(), Some("my_crate"));
    Ok(())
}

#[test]
fn path_attribute_overrides_module_path() -> TestResult<()> {
    let main = r#"
#[path = "weird/place.rs"]
mod renamed;

fn main() {}
"#;
    let weird_place = r#"
pub fn hello() {}
"#;
    let files = [("src/main.rs", main), ("src/weird/place.rs", weird_place)];

    let json = no_sysroot_ast_json("my_crate", &files, "src/weird/place.rs")?;
    assert_eq!(json["crateName"].as_str(), Some("my_crate"));
    assert_eq!(json["modulePath"].as_str(), Some("renamed"));

    let json = no_sysroot_ast_json("my_crate", &files, "src/main.rs")?;
    assert_eq!(json["crateName"].as_str(), Some("my_crate"));
    assert_eq!(json["modulePath"].as_str(), None);

    Ok(())
}

#[test]
fn emits_relative_file_path() -> TestResult<()> {
    let files = [
        ("src/main.rs", "mod foo;\nfn main() {}\n"),
        ("src/foo.rs", "pub fn bar() {}\n"),
    ];

    let json = no_sysroot_ast_json("my_crate", &files, "src/main.rs")?;
    assert_eq!(
        json["relativeFilePath"].as_str().map(Path::new),
        Some(Path::new("src/main.rs")),
    );

    let json = no_sysroot_ast_json("my_crate", &files, "src/foo.rs")?;
    assert_eq!(
        json["relativeFilePath"].as_str().map(Path::new),
        Some(Path::new("src/foo.rs")),
    );

    Ok(())
}

#[test]
fn lib_and_main_coexist() -> TestResult<()> {
    let lib = r#"
pub mod foo;
pub fn baz() {}
"#;
    let main = r#"
fn bar() {}

fn main() {
    my_crate::foo::bar();
    bar();
}
"#;
    let foo = r#"
pub fn bar() {}
"#;
    let files = [
        ("src/lib.rs", lib),
        ("src/main.rs", main),
        ("src/foo.rs", foo),
    ];

    let json = no_sysroot_ast_json("my_crate", &files, "src/foo.rs")?;
    assert_eq!(json["crateName"].as_str(), Some("my_crate"));
    assert_eq!(json["modulePath"].as_str(), Some("foo"));
    assert_eq!(
        fn_decl(&json, "pub fn bar() {}").method_full_name(),
        "my_crate::foo::bar"
    );

    let json = no_sysroot_ast_json("my_crate", &files, "src/main.rs")?;
    assert_eq!(json["crateName"].as_str(), Some("my_crate_bin"));
    assert_eq!(json["modulePath"].as_str(), None);
    assert_eq!(
        fn_decl(
            &json,
            "fn main() {\n    my_crate::foo::bar();\n    bar();\n}"
        )
        .method_full_name(),
        "my_crate_bin::main"
    );
    assert_eq!(
        call_expr(&json, "my_crate::foo::bar()").method_full_name(),
        "my_crate::foo::bar"
    );
    assert_eq!(
        call_expr(&json, "bar()").method_full_name(),
        "my_crate_bin::bar"
    );

    let json = no_sysroot_ast_json("my_crate", &files, "src/lib.rs")?;
    assert_eq!(json["crateName"].as_str(), Some("my_crate"));
    assert_eq!(json["modulePath"].as_str(), None);
    assert_eq!(
        fn_decl(&json, "pub fn baz() {}").method_full_name(),
        "my_crate::baz"
    );

    Ok(())
}

#[test]
fn named_bin_keeps_its_target_name() -> TestResult<()> {
    let files = [("src/lib.rs", ""), ("src/bin/tool.rs", "fn main() {}")];

    let json = no_sysroot_ast_json("my_crate", &files, "src/bin/tool.rs")?;

    assert_eq!(json["crateName"].as_str(), Some("tool"));
    assert_eq!(
        fn_decl(&json, "fn main() {}").method_full_name(),
        "tool::main"
    );

    Ok(())
}

#[test]
fn skips_a_file_outside_the_module_tree() {
    let files = [
        ("src/lib.rs", "pub fn foo() {}\n"),
        ("src/bar.rs", "pub fn baz() {}\n"),
    ];

    assert!(!no_sysroot_ast_json_generated(
        "my_crate",
        &files,
        "src/bar.rs"
    ));
}

#[test]
fn emits_included_items_at_the_include_site() -> TestResult<()> {
    let files = [
        ("src/lib.rs", "include!(\"s.rs\");\n"),
        ("src/s.rs", "pub struct S;\n"),
    ];

    let json = sysroot_ast_json("my_crate", &files, "src/lib.rs")?;

    let structs = nodes_by_kind(&json, "STRUCT");
    assert_eq!(structs.len(), 1);
    assert_eq!(structs[0]["typeFullName"].as_str(), Some("my_crate::S"));

    Ok(())
}

#[test]
fn skips_an_included_file() {
    let files = [
        ("src/lib.rs", "include!(\"s.rs\");\n"),
        ("src/s.rs", "pub struct S;\n"),
    ];

    assert!(!sysroot_ast_json_generated("my_crate", &files, "src/s.rs"));
}

#[test]
fn emits_type_alias_type_full_name() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "my_crate",
        &[("src/lib.rs", "type A = u8;\n")],
        "src/lib.rs",
    )?;

    let aliases = nodes_by_kind(&json, "TYPE_ALIAS");
    assert_eq!(aliases.len(), 1);
    assert_eq!(aliases[0]["typeFullName"].as_str(), Some("my_crate::A"));

    Ok(())
}

// TODO: based on the 0.0.344 regression, i.e. will fail once we are able to update to a version that fixes it.
// Until then, this test at least shows that we don't crash the entire thread on a panick, i.e. only 1 file gets dropped.
#[test]
fn emits_the_files_after_one_rust_analyzer_panics_on() -> TestResult<()> {
    let root = TempDir::with_prefix("rust_ast_gen_integration_test_")?;
    fs::write(
        root.child("Cargo.toml"),
        r#"[package]
name = "my_crate"
version = "0.1.0"
edition = "2021"
"#,
    )?;
    fs::create_dir(root.child("src"))?;
    fs::write(
        root.child("src/lib.rs"),
        r#"
mod n;
trait Tr { type R<'a>; }
#[derive(Clone)]
struct S<K: Tr>(for<'a> fn(K::R<'a>));
fn m<K: Tr>(s: &S<K>) { s.clone(); }
"#,
    )?;
    fs::write(root.child("src/n.rs"), "fn n() {}\n")?;
    let input_dir = dunce::canonicalize(root.path())?;
    let output_dir = input_dir.join("out");

    // Must specify num_threads: 1 otherwise we can't be sure both files are handled by the same worker.
    let config = RustAstGenConfig::new(
        input_dir,
        output_dir.clone(),
        1,
        true,
        false,
        None,
        vec![],
        false,
        false,
        false,
    )?;
    json_gen::run(&config)?;

    assert!(!output_dir.join("src/lib.rs.json").exists());
    assert!(output_dir.join("src/n.rs.json").exists());

    Ok(())
}
