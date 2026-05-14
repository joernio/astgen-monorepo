mod common;

use crate::common::{TestResult, sysroot_ast_json, call_expr, ident_pat, method_call_expr, name_ref};

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
