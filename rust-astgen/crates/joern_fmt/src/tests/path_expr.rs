use super::{
    check_path_ctor_type_full_name, check_path_method_full_name, check_path_type_full_name,
};

#[test]
fn free_fn_as_callee() {
    check_path_method_full_name(
        r#"
fn callback() -> u32 { 1 }

fn main() {
    let value = $0callback();
}
"#,
        "ra_test_fixture::callback",
    );
}

#[test]
fn inherent_fn_as_value() {
    check_path_method_full_name(
        r#"
struct Foo;

impl Foo {
    fn new() -> Foo { Foo }
}

fn main() {
    let f = $0Foo::new;
}
"#,
        "ra_test_fixture::Foo::new",
    );
}

#[test]
fn aliased_free_fn_in_mod() {
    check_path_method_full_name(
        r#"
mod inner {
    pub fn helper() -> u32 { 1 }
}

use inner::helper as aliased;

fn main() {
    let f = $0aliased;
}
"#,
        "ra_test_fixture::inner::helper",
    );
}

#[test]
fn dyn_trait_ref() {
    check_path_type_full_name(
        r#"
trait Tr {}
fn f(g: &dyn Tr) {
    let c = $0g;
}
"#,
        "&dyn ra_test_fixture::Tr",
    );
}

#[test]
fn dyn_trait_ref_with_assoc_type_binding() {
    check_path_type_full_name(
        r#"
trait Tr {
    type A;
}
fn f(g: &dyn Tr<A = i32>) {
    let c = $0g;
}
"#,
        "&dyn ra_test_fixture::Tr<A = i32>",
    );
}

#[test]
fn dyn_trait_ref_with_named_lifetime() {
    check_path_type_full_name(
        r#"
trait Tr {}
fn f<'a>(g: &'a (dyn Tr + 'a)) {
    let c = $0g;
}
"#,
        "&dyn ra_test_fixture::Tr",
    );
}

#[test]
fn local_enum_in_sibling_block_1() {
    check_path_type_full_name(
        r#"
fn f() {
    if true {
        enum E { A }
        let _ = $0E::A;
    }
    if false {
        enum E { B }
        let _ = E::B;
    }
}

fn main() { f(); }
"#,
        "ra_test_fixture::f::E#1",
    );
}

#[test]
fn local_enum_in_sibling_block_2() {
    check_path_type_full_name(
        r#"
fn f() {
    if true {
        enum E { A }
        let _ = E::A;
    }
    if false {
        enum E { B }
        let _ = $0E::B;
    }
}

fn main() { f(); }
"#,
        "ra_test_fixture::f::E#2",
    );
}

#[test]
fn unit_struct_as_value() {
    check_path_ctor_type_full_name(
        r#"
struct S;

fn main() {
    let s = $0S;
}
"#,
        "ra_test_fixture::S",
    );
}

#[test]
fn self_in_impl() {
    check_path_ctor_type_full_name(
        r#"
struct S;

impl S {
    fn f() -> Self { $0Self }
}
"#,
        "ra_test_fixture::S",
    );
}

#[test]
fn self_in_trait_impl() {
    check_path_ctor_type_full_name(
        r#"
struct S;

trait Tr {
    fn m() -> Self;
}

impl Tr for S {
    fn m() -> Self { $0Self }
}
"#,
        "ra_test_fixture::S",
    );
}

#[test]
fn self_in_impl_on_type_alias() {
    check_path_ctor_type_full_name(
        r#"
struct S;

type A = S;

impl A {
    fn f() -> Self { $0Self }
}
"#,
        "ra_test_fixture::S",
    );
}

#[test]
fn unit_variant_as_value() {
    check_path_ctor_type_full_name(
        r#"
enum E { V }

fn main() {
    let v = $0E::V;
}
"#,
        "ra_test_fixture::E::V",
    );
}

#[test]
fn glob_imported_unit_variant_as_value() {
    check_path_ctor_type_full_name(
        r#"
enum E { V }

use E::*;

fn main() {
    let v = $0V;
}
"#,
        "ra_test_fixture::E::V",
    );
}

#[test]
fn tuple_struct_as_value() {
    check_path_method_full_name(
        r#"
struct S(u8);

fn main() {
    let f = $0S;
}
"#,
        "ra_test_fixture::S",
    );
}

#[test]
fn self_in_tuple_struct_impl() {
    check_path_method_full_name(
        r#"
struct S(u8);

impl S {
    fn f() -> fn(u8) -> Self { $0Self }
}
"#,
        "ra_test_fixture::S",
    );
}

#[test]
fn tuple_variant_as_value() {
    check_path_method_full_name(
        r#"
enum E { V(u8) }

fn main() {
    let f = $0E::V;
}
"#,
        "ra_test_fixture::E::V",
    );
}

#[test]
fn glob_imported_tuple_variant_as_value() {
    check_path_method_full_name(
        r#"
enum E { V(u8) }

use E::*;

fn main() {
    let f = $0V;
}
"#,
        "ra_test_fixture::E::V",
    );
}
