mod common;

use crate::common::{
    TestResult, call_expr, ident_pat, impl_decl, method_call_expr, name_ref, path_expr,
    sysroot_ast_json,
};

#[test]
fn emits_names_for_vec_and_dyn_trait() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
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
        )],
        "src/main.rs",
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
        &[(
            "src/main.rs",
            r#"
async fn f() -> i32 {
    1
}

fn main() {
    f();
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        call_expr(&json, "f()").type_full_name(),
        "impl core::future::future::Future<Output = i32> + core::marker::Sized"
    );

    Ok(())
}

#[test]
fn emits_qualified_trait_for_dyn_type_with_auto_traits() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr {}
fn f(g: Box<dyn Tr + Sync + Send>) {
    let c = g;
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        path_expr(&json, "g").type_full_name(),
        "alloc::boxed::Box<dyn rust2cpg::Tr + core::marker::Send + core::marker::Sync, alloc::alloc::Global>"
    );

    Ok(())
}

#[test]
fn emits_qualified_trait_for_dyn_type_without_auto_traits_of_nested_types() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr {
    type A;
}
trait Other {}
fn f(g: &dyn Tr<A = Box<dyn Other + Send>>) {
    let c = g;
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        path_expr(&json, "g").type_full_name(),
        "&dyn rust2cpg::Tr<A = alloc::boxed::Box<dyn rust2cpg::Other + core::marker::Send, alloc::alloc::Global>>"
    );

    Ok(())
}

#[test]
fn emits_distinct_trait_impls_for_dyn_types_differing_in_auto_traits() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr {}
trait Mark {}

impl Mark for Box<dyn Tr> {}
impl Mark for Box<dyn Tr + Send> {}
impl Mark for Box<dyn Tr + Sync> {}
impl Mark for Box<dyn Tr + Send + Sync> {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        impl_decl(&json, "impl Mark for Box<dyn Tr> {}").type_full_name(),
        "<alloc::boxed::Box<dyn rust2cpg::Tr, alloc::alloc::Global> as rust2cpg::Mark>"
    );
    assert_eq!(
        impl_decl(&json, "impl Mark for Box<dyn Tr + Send> {}").type_full_name(),
        "<alloc::boxed::Box<dyn rust2cpg::Tr + core::marker::Send, alloc::alloc::Global> as rust2cpg::Mark>"
    );
    assert_eq!(
        impl_decl(&json, "impl Mark for Box<dyn Tr + Sync> {}").type_full_name(),
        "<alloc::boxed::Box<dyn rust2cpg::Tr + core::marker::Sync, alloc::alloc::Global> as rust2cpg::Mark>"
    );
    assert_eq!(
        impl_decl(&json, "impl Mark for Box<dyn Tr + Send + Sync> {}").type_full_name(),
        "<alloc::boxed::Box<dyn rust2cpg::Tr + core::marker::Send + core::marker::Sync, alloc::alloc::Global> as rust2cpg::Mark>"
    );

    Ok(())
}
