use super::check_record_method_full_name;

#[test]
fn record_struct() {
    check_record_method_full_name(
        r#"
struct S { x: u8 }

fn f() {
    $0S { x: 1 };
}
"#,
        "ra_test_fixture::S",
    );
}

#[test]
fn generic_record_struct() {
    check_record_method_full_name(
        r#"
struct S<T> { x: T }

fn f() {
    $0S { x: 1 };
}
"#,
        "ra_test_fixture::S<T>",
    );
}

#[test]
fn turbofish_generic_record_struct() {
    check_record_method_full_name(
        r#"
struct S<T> { x: T }

fn f() {
    $0S::<u8> { x: 1 };
}
"#,
        "ra_test_fixture::S<T>",
    );
}

#[test]
fn self_in_generic_impl() {
    check_record_method_full_name(
        r#"
struct S<T> { x: T }

impl S<u8> {
    fn f() {
        $0Self { x: 1 };
    }
}
"#,
        "ra_test_fixture::S<T>",
    );
}

#[test]
fn aliased_record_struct() {
    check_record_method_full_name(
        r#"
struct S { x: u8 }

type A = S;

fn f() {
    $0A { x: 1 };
}
"#,
        "ra_test_fixture::S",
    );
}

#[test]
fn aliased_generic_record_struct() {
    check_record_method_full_name(
        r#"
struct S<T> { x: T }

type A<T> = S<T>;

fn f() {
    $0A { x: 1 };
}
"#,
        "ra_test_fixture::S<T>",
    );
}

#[test]
fn record_variant() {
    check_record_method_full_name(
        r#"
enum E { V { x: u8 } }

fn f() {
    $0E::V { x: 1 };
}
"#,
        "ra_test_fixture::E::V",
    );
}

#[test]
fn generic_record_variant() {
    check_record_method_full_name(
        r#"
enum E<T> { V { x: T } }

fn f() {
    $0E::V { x: 1 };
}
"#,
        "ra_test_fixture::E<T>::V",
    );
}

#[test]
fn self_variant_in_impl() {
    check_record_method_full_name(
        r#"
enum E { V { x: u8 } }

impl E {
    fn f() {
        $0Self::V { x: 1 };
    }
}
"#,
        "ra_test_fixture::E::V",
    );
}

#[test]
fn glob_imported_record_variant() {
    check_record_method_full_name(
        r#"
enum E { V { x: u8 } }

use E::*;

fn f() {
    $0V { x: 1 };
}
"#,
        "ra_test_fixture::E::V",
    );
}

#[test]
fn unit_variant_with_braces() {
    check_record_method_full_name(
        r#"
enum E { W }

fn f() {
    $0E::W {};
}
"#,
        "ra_test_fixture::E::W",
    );
}

#[test]
fn tuple_variant_with_braces() {
    check_record_method_full_name(
        r#"
enum E { T(u8) }

fn f() {
    $0E::T { 0: 1 };
}
"#,
        "ra_test_fixture::E::T",
    );
}

#[test]
fn tuple_struct_with_braces() {
    check_record_method_full_name(
        r#"
struct S(u8);

fn f() {
    $0S { 0: 1 };
}
"#,
        "ra_test_fixture::S",
    );
}

#[test]
fn union() {
    check_record_method_full_name(
        r#"
union U { a: u8 }

fn f() {
    $0U { a: 1 };
}
"#,
        "ra_test_fixture::U",
    );
}

#[test]
fn functional_update() {
    check_record_method_full_name(
        r#"
struct S { x: u8 }

fn f(s: S) {
    $0S { ..s };
}
"#,
        "ra_test_fixture::S",
    );
}

#[test]
fn local_record_struct_in_sibling_block_1() {
    check_record_method_full_name(
        r#"
fn f() {
    if true {
        struct S { x: u8 }
        $0S { x: 1 };
    }
    if false {
        struct S { x: u8 }
        S { x: 1 };
    }
}
"#,
        "ra_test_fixture::f::S#1",
    );
}

#[test]
fn local_record_struct_in_sibling_block_2() {
    check_record_method_full_name(
        r#"
fn f() {
    if true {
        struct S { x: u8 }
        S { x: 1 };
    }
    if false {
        struct S { x: u8 }
        $0S { x: 1 };
    }
}
"#,
        "ra_test_fixture::f::S#2",
    );
}
