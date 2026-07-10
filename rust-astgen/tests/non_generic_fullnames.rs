mod common;

use crate::common::{
    TestResult, bin_expr, call_expr, fn_decl, ident_pat, literal, method_call_expr, name_ref,
    no_sysroot_ast_json, path_expr, self_param,
};
use serde_json::json;

#[test]
fn emits_names_for_free_function_call() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn foo() -> u32 {
    1
}

fn main() {
    let value = foo();
}
"#,
        )],
        "src/main.rs",
    )?;

    let foo_call = call_expr(&json, "foo()");
    assert_eq!(foo_call.type_full_name(), "u32");
    assert_eq!(foo_call.method_full_name(), "rust2cpg::foo");

    Ok(())
}

#[test]
fn emits_names_for_function_pointer_call() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn foo() -> u32 {
    1
}

fn main() {
    let ptr: fn() -> u32 = foo;
    let ptr_value = ptr();
}
"#,
        )],
        "src/main.rs",
    )?;

    let ptr_call = call_expr(&json, "ptr()");
    assert_eq!(ptr_call.type_full_name(), "u32");

    Ok(())
}

#[test]
fn emits_names_for_closure_call() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() {
    let closure = || 2u32;
    let closure_value = closure();
}
"#,
        )],
        "src/main.rs",
    )?;

    let closure_call = call_expr(&json, "closure()");
    assert_eq!(closure_call.type_full_name(), "u32");

    Ok(())
}

#[test]
fn emits_names_for_dyn_fn_call() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() {
    let closure = || 2u32;
    let dyn_fn: &dyn Fn() -> u32 = &closure;
    let dyn_value = dyn_fn();
}
"#,
        )],
        "src/main.rs",
    )?;

    let dyn_fn_call = call_expr(&json, "dyn_fn()");
    assert_eq!(dyn_fn_call.type_full_name(), "u32");

    Ok(())
}

#[test]
fn emits_names_for_method_call() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
mod imported {
    pub struct Type;
}

use imported::Type;

impl Type {
    fn value(&self) -> bool {
        true
    }
}

fn main() {
    let receiver = Type;
    let method_value = receiver.value();
}
"#,
        )],
        "src/main.rs",
    )?;

    let value_call = method_call_expr(&json, "receiver.value()");
    assert_eq!(value_call.type_full_name(), "bool");
    assert_eq!(
        value_call.method_full_name(),
        "rust2cpg::imported::Type::value"
    );
    assert_eq!(ident_pat(&json, "method_value").type_full_name(), "bool");
    assert_eq!(
        name_ref(&json, "Type")
            .on_line("impl Type {")
            .type_full_name(),
        "rust2cpg::imported::Type"
    );
    assert_eq!(
        name_ref(&json, "Type")
            .on_line("let receiver = Type;")
            .type_full_name(),
        "rust2cpg::imported::Type"
    );

    Ok(())
}

#[test]
fn emits_unadjusted_type_for_method_receiver() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
mod imported {
    pub struct Type;
}

use imported::Type;

impl Type {
    fn value(&self) -> bool {
        true
    }
}

fn main() {
    let receiver = Type;
    let method_value = receiver.value();
}
"#,
        )],
        "src/main.rs",
    )?;
    assert_eq!(
        name_ref(&json, "receiver").type_full_name(),
        "rust2cpg::imported::Type"
    );
    assert_eq!(
        path_expr(&json, "receiver").adjustments(),
        vec![json!({
            "kind": "borrow",
            "source": "rust2cpg::imported::Type",
            "target": "&rust2cpg::imported::Type",
        })],
    );

    Ok(())
}

#[test]
fn emits_names_for_trait_method_call() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
mod imported {
    pub struct Type;

    pub trait Trait {
        fn trait_value(&self) -> bool;
    }
}

use imported::{Trait, Type};

impl Trait for Type {
    fn trait_value(&self) -> bool {
        true
    }
}

fn main() {
    let trait_receiver = Type;
    let trait_value = trait_receiver.trait_value();
}
"#,
        )],
        "src/main.rs",
    )?;

    let trait_value_call = method_call_expr(&json, "trait_receiver.trait_value()");
    assert_eq!(trait_value_call.type_full_name(), "bool");
    assert_eq!(
        trait_value_call.method_full_name(),
        "<rust2cpg::imported::Type as rust2cpg::imported::Trait>::trait_value"
    );
    assert_eq!(
        name_ref(&json, "Type")
            .on_line("impl Trait for Type {")
            .type_full_name(),
        "rust2cpg::imported::Type"
    );

    Ok(())
}

#[test]
fn emits_concrete_impl_method_for_fully_qualified_path_call() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr {
    fn m(&self) -> i32;
}
struct S;
impl Tr for S {
    fn m(&self) -> i32 { 0 }
}
fn f(s: S) {
    let a = <S as Tr>::m(&s);
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        call_expr(&json, "<S as Tr>::m(&s)").method_full_name(),
        "<rust2cpg::S as rust2cpg::Tr>::m"
    );
    assert_eq!(
        fn_decl(&json, "fn m(&self) -> i32 { 0 }").method_full_name(),
        "<rust2cpg::S as rust2cpg::Tr>::m"
    );

    Ok(())
}

#[test]
fn emits_concrete_impl_method_for_trait_qualified_path_call() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr {
    fn m(&self) -> i32;
}
struct S;
impl Tr for S {
    fn m(&self) -> i32 { 0 }
}
fn f(s: S) {
    let b = Tr::m(&s);
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        call_expr(&json, "Tr::m(&s)").method_full_name(),
        "<rust2cpg::S as rust2cpg::Tr>::m"
    );
    assert_eq!(
        fn_decl(&json, "fn m(&self) -> i32 { 0 }").method_full_name(),
        "<rust2cpg::S as rust2cpg::Tr>::m"
    );

    Ok(())
}

#[test]
fn emits_trait_method_for_dyn_receiver_path_call() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr {
    fn m(&self) -> i32;
}
struct S;
impl Tr for S {
    fn m(&self) -> i32 { 0 }
}
fn f(g: &dyn Tr) {
    let c = Tr::m(g);
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        call_expr(&json, "Tr::m(g)").method_full_name(),
        "rust2cpg::Tr::m"
    );
    assert_eq!(
        fn_decl(&json, "fn m(&self) -> i32;").method_full_name(),
        "rust2cpg::Tr::m"
    );

    Ok(())
}

#[test]
fn emits_names_for_methods_via_ref_receiver() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
mod imported {
    pub struct Type;
}

use imported::Type;

impl Type {
    fn value(&self) -> bool {
        true
    }

    fn by_mut(&mut self) -> bool {
        true
    }
}

fn main() {
    let ref_receiver = Type;
    let ref_value = (&ref_receiver).value();
    let mut mut_receiver = Type;
    let mut_value = (&mut mut_receiver).by_mut();
}
"#,
        )],
        "src/main.rs",
    )?;

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

    Ok(())
}

#[test]
fn emits_names_for_self_param() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
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

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

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

    Ok(())
}

#[test]
fn emits_names_for_references() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() {
    let text: &str = "hello";
    let text_copy = text;
    let mut number = 1u32;
    let number_ref: &mut u32 = &mut number;
    let number_ref_copy = number_ref;
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(name_ref(&json, "text").type_full_name(), "&str");
    assert_eq!(ident_pat(&json, "text_copy").type_full_name(), "&str");
    assert_eq!(ident_pat(&json, "number_ref").type_full_name(), "&mut u32");
    assert_eq!(name_ref(&json, "number_ref").type_full_name(), "&mut u32");
    assert_eq!(
        ident_pat(&json, "number_ref_copy").type_full_name(),
        "&mut u32"
    );

    Ok(())
}

#[test]
fn emits_names_for_literals_and_binary_expressions() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() {
    let sum = 1u32 + 2u32;
}
"#,
        )],
        "src/main.rs",
    )?;

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
fn emits_names_for_raw_pointer_parameters() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn takes(p_const: *const i32, p_mut: *mut i32) {
    let _ = p_const;
    let _ = p_mut;
}

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(ident_pat(&json, "p_const").type_full_name(), "*const i32");
    assert_eq!(ident_pat(&json, "p_mut").type_full_name(), "*mut i32");

    Ok(())
}

#[test]
fn emits_names_for_raw_pointer_return_type() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn returns_const() -> *const i32 {
    0 as *const i32
}

fn returns_mut() -> *mut i32 {
    0 as *mut i32
}

fn main() {
    let const_ptr = returns_const();
    let mut_ptr = returns_mut();
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        call_expr(&json, "returns_const()").type_full_name(),
        "*const i32"
    );
    assert_eq!(
        call_expr(&json, "returns_mut()").type_full_name(),
        "*mut i32"
    );
    assert_eq!(ident_pat(&json, "const_ptr").type_full_name(), "*const i32");
    assert_eq!(ident_pat(&json, "mut_ptr").type_full_name(), "*mut i32");

    Ok(())
}

#[test]
fn emits_names_for_raw_pointer_to_user_adt() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct Type;

fn takes(const_p: *const Type, mut_p: *mut Type) {
    let _ = const_p;
    let _ = mut_p;
}

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        ident_pat(&json, "const_p").type_full_name(),
        "*const rust2cpg::Type"
    );
    assert_eq!(
        ident_pat(&json, "mut_p").type_full_name(),
        "*mut rust2cpg::Type"
    );

    Ok(())
}

#[test]
fn emits_names_for_nested_raw_pointer() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn takes(p: *const *mut i32) {
    let _ = p;
}

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(ident_pat(&json, "p").type_full_name(), "*const *mut i32");

    Ok(())
}

#[test]
fn emits_names_for_raw_pointer_to_slice() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn takes(p: *const [i32]) {
    let _ = p;
}

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(ident_pat(&json, "p").type_full_name(), "*const [i32]");

    Ok(())
}

#[test]
fn emits_names_for_tuple_types() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() {
    let unit = ();
    let single = (1i32,);
    let multi = (1i32, 2u32, true);
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(ident_pat(&json, "unit").type_full_name(), "()");
    assert_eq!(ident_pat(&json, "single").type_full_name(), "(i32,)");
    assert_eq!(
        ident_pat(&json, "multi").type_full_name(),
        "(i32, u32, bool)"
    );

    Ok(())
}

#[test]
fn emits_names_for_array_type() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() {
    let arr = [0, 1, 2, 3];
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(ident_pat(&json, "arr").type_full_name(), "[i32; 4]");

    Ok(())
}

#[test]
fn emits_names_for_slice_type() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() {
    let arr: [i32; 3] = [1, 2, 3];
    let slice: &[i32] = &arr;
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(ident_pat(&json, "slice").type_full_name(), "&[i32]");

    Ok(())
}

#[test]
fn emits_names_for_fn_pointer_type() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn no_args() -> u32 {
    1
}

fn with_args(a: u32, b: bool) -> u32 {
    if b { a + 1 } else { a }
}

fn main() {
    let no_arg_ptr: fn() -> u32 = no_args;
    let multi_arg_ptr: fn(u32, bool) -> u32 = with_args;
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        ident_pat(&json, "no_arg_ptr").type_full_name(),
        "fn() -> u32"
    );
    assert_eq!(
        ident_pat(&json, "multi_arg_ptr").type_full_name(),
        "fn(u32, bool) -> u32"
    );

    Ok(())
}
