mod common;

use crate::common::{
    TestResult, call_expr, fn_decl, nodes_by_kind, sysroot_ast_json, sysroot_no_proc_macro_ast_json,
};
use serde_json::Value;

fn with_proc_macro_crate(lib_rs: &'static str) -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "Cargo.toml",
            r#"[package]
name = "rust2cpg"
version = "0.1.0"
edition = "2021"

[dependencies]
pm = { path = "pm" }
"#,
        ),
        (
            "pm/Cargo.toml",
            r#"[package]
name = "pm"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true
"#,
        ),
        (
            "pm/src/lib.rs",
            r#"
use proc_macro::TokenStream;

#[proc_macro]
pub fn emit_f(_input: TokenStream) -> TokenStream {
    "fn f() {}".parse().unwrap()
}

#[proc_macro_attribute]
pub fn prepend_g(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut output: TokenStream = "fn g() {}".parse().unwrap();
    output.extend(item);
    output
}
"#,
        ),
        ("src/lib.rs", lib_rs),
    ]
}

#[test]
fn expands_function_like_proc_macro() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        &with_proc_macro_crate(
            r#"
pm::emit_f!();
"#,
        ),
        "src/lib.rs",
    )?;

    let fns = nodes_by_kind(&json, "FN");
    assert_eq!(fns.len(), 1);
    assert_eq!(
        fns[0].get("methodFullName").and_then(Value::as_str),
        Some("rust2cpg::f")
    );

    Ok(())
}

#[test]
fn resolves_call_to_proc_macro_generated_fn() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        &with_proc_macro_crate(
            r#"
pm::emit_f!();

fn g() {
    f();
}
"#,
        ),
        "src/lib.rs",
    )?;

    assert_eq!(call_expr(&json, "f()").method_full_name(), "rust2cpg::f");

    Ok(())
}

#[test]
fn no_proc_macro_unexpanded() -> TestResult<()> {
    let json = sysroot_no_proc_macro_ast_json(
        "rust2cpg",
        &with_proc_macro_crate(
            r#"
pm::emit_f!();
"#,
        ),
        "src/lib.rs",
    )?;

    assert!(nodes_by_kind(&json, "FN").is_empty());

    Ok(())
}

#[test]
fn expands_attribute() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        &with_proc_macro_crate(
            r#"
#[pm::prepend_g]
fn h() {}
"#,
        ),
        "src/lib.rs",
    )?;

    // NB: Expected -- `fn h` is consumed by the proc macro, so only its expansion
    // has methodFullName.
    assert_eq!(
        fn_decl(&json, "#[pm::prepend_g]\nfn h() {}").method_full_name_opt(),
        None
    );
    assert_eq!(fn_decl(&json, "fn g(){}").method_full_name(), "rust2cpg::g");
    assert_eq!(fn_decl(&json, "fn h(){}").method_full_name(), "rust2cpg::h");

    Ok(())
}

#[test]
fn resolves_call_to_generated_fn() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        &with_proc_macro_crate(
            r#"
#[pm::prepend_g]
fn h() {}

fn k() {
    g();
}
"#,
        ),
        "src/lib.rs",
    )?;

    assert_eq!(call_expr(&json, "g()").method_full_name(), "rust2cpg::g");

    Ok(())
}

#[test]
fn no_proc_macro_leaves_attribute() -> TestResult<()> {
    let json = sysroot_no_proc_macro_ast_json(
        "rust2cpg",
        &with_proc_macro_crate(
            r#"
#[pm::prepend_g]
fn h() {}
"#,
        ),
        "src/lib.rs",
    )?;

    assert_eq!(
        fn_decl(&json, "#[pm::prepend_g]\nfn h() {}").method_full_name(),
        "rust2cpg::h"
    );
    assert!(!fn_decl(&json, "fn h(){}").exists());

    Ok(())
}
