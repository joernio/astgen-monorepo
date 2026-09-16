use super::{check_enum_implemented_traits, check_enum_type_full_name};

#[test]
fn enum_with_trait_impl() {
    check_enum_implemented_traits(
        r#"
trait Tr {}

enum $0E { A }

impl Tr for E {}
"#,
        Some(vec!["ra_test_fixture::Tr"]),
    );
}

#[test]
fn enum_with_generic_trait_impl() {
    check_enum_implemented_traits(
        r#"
trait Tr<T> {}

enum $0E { A }

impl Tr<u8> for E {}
"#,
        Some(vec!["ra_test_fixture::Tr<u8>"]),
    );
}

#[test]
fn generic_enum() {
    check_enum_type_full_name(
        r#"
enum $0E<T> { A(T) }

fn main() {
    let _ = E::A(1);
}
"#,
        "ra_test_fixture::E<T>",
    );
}

#[test]
fn enum_with_unit_variant() {
    check_enum_type_full_name(
        r#"
enum $0E { A }

fn main() {
    let _ = E::A;
}
"#,
        "ra_test_fixture::E",
    );
}

#[test]
fn local_enum_in_sibling_block_1() {
    check_enum_type_full_name(
        r#"
fn f() {
    if true {
        enum $0E { A }
        let _ = E::A;
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
    check_enum_type_full_name(
        r#"
fn f() {
    if true {
        enum E { A }
        let _ = E::A;
    }
    if false {
        enum $0E { B }
        let _ = E::B;
    }
}

fn main() { f(); }
"#,
        "ra_test_fixture::f::E#2",
    );
}
