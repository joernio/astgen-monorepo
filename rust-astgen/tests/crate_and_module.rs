use crate::common::{TestResult, no_sysroot_ast_json};

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
fn lib_and_main_coexist() -> TestResult<()> {
    let lib = r#"
pub mod foo;
"#;
    let main = r#"
fn main() {}
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

    let json = no_sysroot_ast_json("my_crate", &files, "src/main.rs")?;
    assert_eq!(json["crateName"].as_str(), Some("my_crate"));
    assert_eq!(json["modulePath"].as_str(), None);

    let json = no_sysroot_ast_json("my_crate", &files, "src/lib.rs")?;
    assert_eq!(json["crateName"].as_str(), Some("my_crate"));
    assert_eq!(json["modulePath"].as_str(), None);

    Ok(())
}
