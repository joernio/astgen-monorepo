mod common;

use crate::common::{TestResult, ast_json, call_expr, ident_pat, method_call_expr, name_ref};

#[test]
fn emits_names_for_generic_paths_and_trait_impl_calls() -> TestResult<()> {
    let json = ast_json(
        "rust2cpg",
        r#"
trait Extract<T> {
    fn extract(&self) -> T;
}

trait Sink<T> {
    fn sink(&self) -> T;
}

use std::vec::Vec;

fn identity<T>(value: T) -> T {
    value
}

struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        Wrapper(value)
    }

    fn value(&self) -> T {
        self.0
    }

    fn value_mut(&mut self) -> T {
        self.0
    }

    fn passthrough<U>(&self, value: U) -> U {
        value
    }
}

impl<T: Copy> Extract<T> for Wrapper<T> {
    fn extract(&self) -> T {
        self.0
    }
}

fn call_bound<S: Sink<u32>>(s: S) {
    let bound_value = s.sink();
}

fn call_dyn(s: &dyn Sink<u32>) {
    let dyn_value = s.sink();
}

fn main() {
    let identity_value = identity::<u32>(1);
    let wrapped = Wrapper::<u32>::new(1);
    let direct = wrapped.value();
    let by_ref = (&wrapped).value();
    let mut mutable = Wrapper::<u32>::new(2);
    let by_mut = (&mut mutable).value_mut();
    let passthrough = wrapped.passthrough::<bool>(true);
    let extracted = wrapped.extract();
    let plain = Wrapper(3u32);
    let copied = plain;
    let numbers = Vec::<u32>::new();
    let numbers_copy = numbers;
}
"#,
    )?;

    let tuple_constructor = call_expr(&json, "Wrapper(value)");
    assert_eq!(tuple_constructor.type_full_name(), "rust2cpg::Wrapper<T>");
    assert_eq!(tuple_constructor.method_full_name(), "rust2cpg::Wrapper<T>");

    let identity_call = call_expr(&json, "identity::<u32>(1)");
    assert_eq!(identity_call.type_full_name(), "u32");
    assert_eq!(identity_call.method_full_name(), "rust2cpg::identity<T>");

    let new_call = call_expr(&json, "Wrapper::<u32>::new(1)");
    assert_eq!(new_call.type_full_name(), "rust2cpg::Wrapper<u32>");
    assert_eq!(new_call.method_full_name(), "rust2cpg::Wrapper<T>::new");
    assert_eq!(
        name_ref(&json, "Wrapper")
            .on_line("let wrapped = Wrapper::<u32>::new(1);")
            .type_full_name(),
        "rust2cpg::Wrapper<u32>"
    );
    assert_eq!(
        name_ref(&json, "new")
            .on_line("let wrapped = Wrapper::<u32>::new(1);")
            .type_full_name(),
        "fn(u32) -> rust2cpg::Wrapper<u32>"
    );
    assert_eq!(
        ident_pat(&json, "wrapped").type_full_name(),
        "rust2cpg::Wrapper<u32>"
    );

    let direct_call = method_call_expr(&json, "wrapped.value()");
    assert_eq!(direct_call.type_full_name(), "u32");
    assert_eq!(
        direct_call.method_full_name(),
        "rust2cpg::Wrapper<T>::value"
    );

    let by_ref_call = method_call_expr(&json, "(&wrapped).value()");
    assert_eq!(by_ref_call.type_full_name(), "u32");
    assert_eq!(
        by_ref_call.method_full_name(),
        "rust2cpg::Wrapper<T>::value"
    );

    let by_mut_call = method_call_expr(&json, "(&mut mutable).value_mut()");
    assert_eq!(by_mut_call.type_full_name(), "u32");
    assert_eq!(
        by_mut_call.method_full_name(),
        "rust2cpg::Wrapper<T>::value_mut"
    );

    let passthrough_call = method_call_expr(&json, "wrapped.passthrough::<bool>(true)");
    assert_eq!(passthrough_call.type_full_name(), "bool");
    assert_eq!(
        passthrough_call.method_full_name(),
        "rust2cpg::Wrapper<T>::passthrough<U>"
    );

    let extract_call = method_call_expr(&json, "wrapped.extract()");
    assert_eq!(extract_call.type_full_name(), "u32");
    assert_eq!(
        extract_call.method_full_name(),
        "<rust2cpg::Wrapper<T> as rust2cpg::Extract<T>>::extract"
    );

    let bound_sink_call =
        method_call_expr(&json, "s.sink()").on_line("let bound_value = s.sink();");
    assert_eq!(bound_sink_call.type_full_name(), "u32");
    assert_eq!(
        bound_sink_call.method_full_name(),
        "rust2cpg::Sink<T>::sink"
    );

    let dyn_sink_call = method_call_expr(&json, "s.sink()").on_line("let dyn_value = s.sink();");
    assert_eq!(dyn_sink_call.type_full_name(), "u32");
    assert_eq!(dyn_sink_call.method_full_name(), "rust2cpg::Sink<T>::sink");

    assert_eq!(
        ident_pat(&json, "value")
            .on_line("fn new(value: T) -> Wrapper<T> {")
            .type_full_name(),
        "T"
    );
    assert_eq!(
        name_ref(&json, "Wrapper")
            .on_line("impl<T: Copy> Wrapper<T> {")
            .type_full_name(),
        "rust2cpg::Wrapper<T>"
    );
    assert_eq!(
        name_ref(&json, "Wrapper")
            .on_line("fn new(value: T) -> Wrapper<T> {")
            .type_full_name(),
        "rust2cpg::Wrapper<T>"
    );
    assert_eq!(
        name_ref(&json, "Wrapper")
            .on_line("Wrapper(value)")
            .type_full_name(),
        "fn(T) -> rust2cpg::Wrapper<T>"
    );
    assert_eq!(
        name_ref(&json, "Wrapper")
            .on_line("impl<T: Copy> Extract<T> for Wrapper<T> {")
            .type_full_name(),
        "rust2cpg::Wrapper<T>"
    );

    assert_eq!(
        ident_pat(&json, "plain").type_full_name(),
        "rust2cpg::Wrapper<u32>"
    );
    assert_eq!(
        ident_pat(&json, "copied").type_full_name(),
        "rust2cpg::Wrapper<u32>"
    );
    assert_eq!(
        name_ref(&json, "plain").type_full_name(),
        "rust2cpg::Wrapper<u32>"
    );

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
    Ok(())
}
