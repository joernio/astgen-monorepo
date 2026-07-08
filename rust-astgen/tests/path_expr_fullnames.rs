mod common;

use crate::common::{TestResult, call_expr, fn_decl, no_sysroot_ast_json, path_expr};

#[test]
fn aliased_func_method_full_name() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
mod inner {
    pub fn helper() -> u32 { 1 }
}

use inner::helper as aliased;

fn main() {
    let f = aliased;
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        path_expr(&json, "aliased").method_full_name(),
        fn_decl(&json, "pub fn helper() -> u32 { 1 }").method_full_name()
    );

    Ok(())
}

#[test]
fn assoc_func_method_full_name() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct Foo;

impl Foo {
    fn new() -> Foo { Foo }
}

fn main() {
    let f = Foo::new;
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        path_expr(&json, "Foo::new").method_full_name(),
        fn_decl(&json, "fn new() -> Foo { Foo }").method_full_name()
    );

    Ok(())
}

#[test]
fn func_call_and_path_method_full_name() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn callback() -> u32 { 1 }

fn main() {
    let value = callback();
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        call_expr(&json, "callback()").method_full_name(),
        "rust2cpg::callback"
    );
    assert_eq!(
        path_expr(&json, "callback").method_full_name(),
        "rust2cpg::callback"
    );

    Ok(())
}
