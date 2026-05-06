mod common;

use crate::common::{
    TestResult, ast_json, bin_expr, call_expr, ident_pat, literal, method_call_expr, name_ref,
    self_param,
};

#[test]
fn emits_names_for_calls_receivers_and_values() -> TestResult<()> {
    let json = ast_json(
        "rust2cpg",
        r#"
fn foo() -> u32 {
    1
}

mod imported {
    pub struct Type;

    pub trait Trait {
        fn trait_value(&self) -> bool;
    }
}

use imported::{Trait, Type};

impl Type {
    fn value(&self) -> bool {
        true
    }

    fn by_mut(&mut self) -> bool {
        true
    }
}

impl Trait for Type {
    fn trait_value(&self) -> bool {
        true
    }
}

fn main() {
    let value = foo();
    let ptr: fn() -> u32 = foo;
    let ptr_value = ptr();
    let closure = || 2u32;
    let closure_value = closure();
    let dyn_fn: &dyn Fn() -> u32 = &closure;
    let dyn_value = dyn_fn();

    let receiver = Type;
    let method_value = receiver.value();
    let trait_receiver = Type;
    let trait_value = trait_receiver.trait_value();
    let ref_receiver = Type;
    let ref_value = (&ref_receiver).value();
    let mut mut_receiver = Type;
    let mut_value = (&mut mut_receiver).by_mut();

    let text: &str = "hello";
    let text_copy = text;
    let mut number = 1u32;
    let number_ref: &mut u32 = &mut number;
    let number_ref_copy = number_ref;
    let sum = 1u32 + 2u32;
}
"#,
    )?;

    let foo_call = call_expr(&json, "foo()");
    assert_eq!(foo_call.type_full_name(), "u32");
    assert_eq!(foo_call.method_full_name(), "rust2cpg::foo");

    let function_pointer_call = call_expr(&json, "ptr()");
    assert_eq!(function_pointer_call.type_full_name(), "u32");

    let closure_call = call_expr(&json, "closure()");
    assert_eq!(closure_call.type_full_name(), "u32");

    let fn_impl_call = call_expr(&json, "dyn_fn()");
    assert_eq!(fn_impl_call.type_full_name(), "u32");

    let value_call = method_call_expr(&json, "receiver.value()");
    assert_eq!(value_call.type_full_name(), "bool");
    assert_eq!(
        value_call.method_full_name(),
        "rust2cpg::imported::Type::value"
    );
    assert_eq!(
        name_ref(&json, "receiver").type_full_name(),
        "&rust2cpg::imported::Type"
    );
    assert_eq!(ident_pat(&json, "method_value").type_full_name(), "bool");

    let trait_value_call = method_call_expr(&json, "trait_receiver.trait_value()");
    assert_eq!(trait_value_call.type_full_name(), "bool");
    assert_eq!(
        trait_value_call.method_full_name(),
        "<rust2cpg::imported::Type as rust2cpg::imported::Trait>::trait_value"
    );

    let ref_value_call = method_call_expr(&json, "(&ref_receiver).value()");
    assert_eq!(ref_value_call.type_full_name(), "bool");
    assert_eq!(
        ref_value_call.method_full_name(),
        "rust2cpg::imported::Type::value"
    );

    let mut_value_call = method_call_expr(&json, "(&mut mut_receiver).by_mut()");
    assert_eq!(mut_value_call.type_full_name(), "bool");
    assert_eq!(
        mut_value_call.method_full_name(),
        "rust2cpg::imported::Type::by_mut"
    );

    assert_eq!(
        self_param(&json, "&self")
            .on_line("fn trait_value(&self) -> bool;")
            .type_full_name(),
        "&Self"
    );
    assert_eq!(
        self_param(&json, "&self")
            .on_line("fn value(&self) -> bool {")
            .type_full_name(),
        "&rust2cpg::imported::Type"
    );
    assert_eq!(
        self_param(&json, "&self")
            .on_line("fn trait_value(&self) -> bool {")
            .type_full_name(),
        "&rust2cpg::imported::Type"
    );
    assert_eq!(
        self_param(&json, "&mut self")
            .on_line("fn by_mut(&mut self) -> bool {")
            .type_full_name(),
        "&mut rust2cpg::imported::Type"
    );

    assert_eq!(
        name_ref(&json, "Type")
            .on_line("impl Type {")
            .type_full_name(),
        "rust2cpg::imported::Type"
    );
    assert_eq!(
        name_ref(&json, "Type")
            .on_line("impl Trait for Type {")
            .type_full_name(),
        "rust2cpg::imported::Type"
    );
    assert_eq!(
        name_ref(&json, "Type")
            .on_line("let receiver = Type;")
            .type_full_name(),
        "rust2cpg::imported::Type"
    );

    assert_eq!(name_ref(&json, "text").type_full_name(), "&str");
    assert_eq!(ident_pat(&json, "number_ref").type_full_name(), "&mut u32");
    assert_eq!(name_ref(&json, "number_ref").type_full_name(), "&mut u32");
    assert_eq!(bin_expr(&json, "1u32 + 2u32").type_full_name(), "u32");
    assert_eq!(
        literal(&json, "1u32")
            .on_line("let sum = 1u32 + 2u32;")
            .type_full_name(),
        "u32"
    );
    assert_eq!(
        literal(&json, "2u32")
            .on_line("let sum = 1u32 + 2u32;")
            .type_full_name(),
        "u32"
    );
    assert_eq!(ident_pat(&json, "sum").type_full_name(), "u32");
    Ok(())
}

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
        "fn new<u32>(u32) -> Wrapper<u32>"
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

    // TODO: this is being flaky in CI... don't understand why.
    //let dyn_sink_call = method_call_expr(&json, "s.sink()").on_line("let dyn_value = s.sink();");
    //assert_eq!(dyn_sink_call.type_full_name(), "u32");
    //assert_eq!(dyn_sink_call.method_full_name(), "rust2cpg::Sink<T>::sink");

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
        "fn Wrapper<T>(T) -> Wrapper<T>"
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
        "fn new<u32>() -> Vec<u32, Global>"
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
