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
