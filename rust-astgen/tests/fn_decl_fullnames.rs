mod common;

use crate::common::{TestResult, call_expr, fn_decl, method_call_expr, no_sysroot_ast_json};

#[test]
fn free_function_decl_matches_call_site() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn foo() -> u32 { 1 }

fn main() {
    let value = foo();
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        fn_decl(&json, "fn foo() -> u32 { 1 }").method_full_name(),
        "rust2cpg::foo"
    );
    assert_eq!(
        call_expr(&json, "foo()").method_full_name(),
        "rust2cpg::foo"
    );

    Ok(())
}

#[test]
fn free_function_in_module_is_qualified() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
mod inner {
    pub fn foo() -> u32 { 1 }
}

fn main() {
    let value = inner::foo();
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        fn_decl(&json, "pub fn foo() -> u32 { 1 }").method_full_name(),
        "rust2cpg::inner::foo"
    );
    assert_eq!(
        call_expr(&json, "inner::foo()").method_full_name(),
        "rust2cpg::inner::foo"
    );

    Ok(())
}

#[test]
fn generic_free_function_keeps_type_param_suffix() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn identity<T>(value: T) -> T { value }

fn main() {
    let value = identity::<u32>(1);
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        fn_decl(&json, "fn identity<T>(value: T) -> T { value }").method_full_name(),
        "rust2cpg::identity<T>"
    );
    assert_eq!(
        call_expr(&json, "identity::<u32>(1)").method_full_name(),
        "rust2cpg::identity<T>"
    );

    Ok(())
}

#[test]
fn inherent_associated_function_decl_matches_call_site() -> TestResult<()> {
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
    let value = Foo::new();
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        fn_decl(&json, "fn new() -> Foo { Foo }").method_full_name(),
        "rust2cpg::Foo::new"
    );
    assert_eq!(
        call_expr(&json, "Foo::new()").method_full_name(),
        "rust2cpg::Foo::new"
    );

    Ok(())
}

#[test]
fn inherent_associated_function_on_generic_type_keeps_type_params() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct Wrapper<T>(T);

impl<T> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> { Wrapper(value) }
}

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        fn_decl(&json, "fn new(value: T) -> Wrapper<T> { Wrapper(value) }").method_full_name(),
        "rust2cpg::Wrapper<T>::new"
    );

    Ok(())
}

#[test]
fn trait_impl_method_decl_matches_dot_call_site() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct Foo;

trait Greet {
    fn hello(&self) -> bool;
}

impl Greet for Foo {
    fn hello(&self) -> bool { true }
}

fn main() {
    let foo = Foo;
    let value = foo.hello();
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        fn_decl(&json, "fn hello(&self) -> bool { true }").method_full_name(),
        "<rust2cpg::Foo as rust2cpg::Greet>::hello"
    );
    assert_eq!(
        method_call_expr(&json, "foo.hello()").method_full_name(),
        "<rust2cpg::Foo as rust2cpg::Greet>::hello"
    );

    Ok(())
}

#[test]
fn trait_impl_associated_function_without_self_is_qualified() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct Foo;

trait Make {
    fn make() -> Foo;
}

impl Make for Foo {
    fn make() -> Foo { Foo }
}

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        fn_decl(&json, "fn make() -> Foo { Foo }").method_full_name(),
        "<rust2cpg::Foo as rust2cpg::Make>::make"
    );

    Ok(())
}

#[test]
fn trait_body_method_declaration_is_type_erased() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Greet {
    fn hello(&self) -> bool;
}

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        fn_decl(&json, "fn hello(&self) -> bool;").method_full_name(),
        "rust2cpg::Greet::hello"
    );

    Ok(())
}

#[test]
fn trait_body_default_method_is_type_erased() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Greet {
    fn hello(&self) -> bool { true }
}

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        fn_decl(&json, "fn hello(&self) -> bool { true }").method_full_name(),
        "rust2cpg::Greet::hello"
    );

    Ok(())
}

#[test]
fn local_function_decl_matches_call_site() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() {
    fn helper() -> u32 { 1 }
    let value = helper();
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        fn_decl(&json, "fn helper() -> u32 { 1 }").method_full_name(),
        "rust2cpg::main::helper"
    );
    assert_eq!(
        call_expr(&json, "helper()").method_full_name(),
        "rust2cpg::main::helper"
    );

    Ok(())
}

#[test]
fn local_functions_in_sibling_blocks_are_disambiguated() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn f() {
    if true {
        fn g() -> u32 { 1 }
        let _a = g();
    }
    if false {
        fn g() -> u8 { 2 }
        let _b = g();
    }
}

fn main() { f(); }
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        fn_decl(&json, "fn g() -> u32 { 1 }").method_full_name(),
        "rust2cpg::f::g#1"
    );
    assert_eq!(
        fn_decl(&json, "fn g() -> u8 { 2 }").method_full_name(),
        "rust2cpg::f::g#2"
    );
    assert_eq!(
        call_expr(&json, "g()")
            .on_line("        let _a = g();")
            .method_full_name(),
        "rust2cpg::f::g#1"
    );
    assert_eq!(
        call_expr(&json, "g()")
            .on_line("        let _b = g();")
            .method_full_name(),
        "rust2cpg::f::g#2"
    );

    Ok(())
}
