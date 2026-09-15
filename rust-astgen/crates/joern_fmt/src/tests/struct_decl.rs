use super::{
    check_struct_implemented_traits, check_struct_method_full_name, check_struct_type_full_name,
};

#[test]
fn tuple_struct() {
    check_struct_method_full_name(
        r#"
struct $0Plain(i32, bool);

fn main() {
    let _value = Plain(1, true);
}
"#,
        Some("ra_test_fixture::Plain"),
    );
}

#[test]
fn generic_tuple_struct() {
    check_struct_method_full_name(
        r#"
struct $0OneParam<T>(T);

fn main() {
    let _value = OneParam(1u32);
}
"#,
        Some("ra_test_fixture::OneParam<T>"),
    );
}

#[test]
fn tuple_struct_with_two_type_params() {
    check_struct_method_full_name(
        r#"
struct $0Multi<A, B>(A, B);

fn main() {
    let _value = Multi(1u32, true);
}
"#,
        Some("ra_test_fixture::Multi<A, B>"),
    );
}

#[test]
fn tuple_struct_with_bounded_type_param() {
    check_struct_method_full_name(
        r#"
struct $0Bounded<T: Clone>(T);

fn main() {
    let _value = Bounded(1u32);
}
"#,
        Some("ra_test_fixture::Bounded<T>"),
    );
}

#[test]
fn tuple_struct_with_lifetime_param() {
    check_struct_method_full_name(
        r#"
struct $0WithLife<'a>(&'a i32);

fn main() {
    let _value = WithLife(&1i32);
}
"#,
        Some("ra_test_fixture::WithLife"),
    );
}

#[test]
fn tuple_struct_with_lifetime_and_type_params() {
    check_struct_method_full_name(
        r#"
struct $0Mix<'a, T>(&'a T);

fn main() {
    let _value = Mix(&1u32);
}
"#,
        Some("ra_test_fixture::Mix<T>"),
    );
}

#[test]
fn tuple_struct_with_defaulted_type_param() {
    check_struct_method_full_name(
        r#"
struct $0WithDefault<T = i32>(T);

fn main() {
    let _value = WithDefault(1i32);
}
"#,
        Some("ra_test_fixture::WithDefault<T>"),
    );
}

// TODO: `const` params have not been dealt with yet, just recording the status quo.
#[test]
fn tuple_struct_with_const_param() {
    check_struct_method_full_name(
        r#"
struct $0WithConst<const N: usize>(usize);

fn main() {
    let _value = WithConst::<5>(0usize);
}
"#,
        Some("ra_test_fixture::WithConst<N>"),
    );
}

#[test]
fn record_struct() {
    check_struct_method_full_name(
        r#"
struct $0Record { x: i32 }

fn main() {
    let _r = Record { x: 1 };
}
"#,
        None,
    );
}

#[test]
fn unit_struct() {
    check_struct_method_full_name(
        r#"
struct $0Unit;

fn main() {
    let _u = Unit;
}
"#,
        None,
    );
}

#[test]
fn local_tuple_struct() {
    check_struct_type_full_name(
        r#"
fn f() {
    struct $0S(i32);
    let _ = S(1);
}

fn main() { f(); }
"#,
        "ra_test_fixture::f::S",
    );
}

#[test]
fn struct_with_trait_impl() {
    check_struct_implemented_traits(
        r#"
trait Tr {}

struct $0S;

impl Tr for S {}
"#,
        Some(vec!["ra_test_fixture::Tr"]),
    );
}

#[test]
fn struct_with_generic_trait_impl() {
    check_struct_implemented_traits(
        r#"
trait Tr<T> {}

struct $0S;

impl Tr<u8> for S {}
"#,
        Some(vec!["ra_test_fixture::Tr<u8>"]),
    );
}

#[test]
fn struct_with_two_impls_of_generic_trait() {
    check_struct_implemented_traits(
        r#"
trait Tr<T> {}

struct $0S;

impl Tr<u8> for S {}
impl Tr<u16> for S {}
"#,
        Some(vec!["ra_test_fixture::Tr<u16>", "ra_test_fixture::Tr<u8>"]),
    );
}

#[test]
fn struct_with_generic_impl_of_generic_trait() {
    check_struct_implemented_traits(
        r#"
trait Tr<T> {}

struct $0S;

impl<T> Tr<T> for S {}
"#,
        Some(vec!["ra_test_fixture::Tr<T>"]),
    );
}

#[test]
fn struct_with_inherent_impl() {
    check_struct_implemented_traits(
        r#"
struct $0S;

impl S {
    fn m(&self) {}
}
"#,
        None,
    );
}

#[test]
fn struct_with_negative_trait_impl() {
    check_struct_implemented_traits(
        r#"
trait Tr {}

struct $0S;

impl !Tr for S {}
"#,
        None,
    );
}

#[test]
fn struct_with_trait_impl_in_other_mod() {
    check_struct_implemented_traits(
        r#"
//- /main.rs
mod m;

trait Tr {}

struct $0S;

//- /m.rs
use crate::{S, Tr};

impl Tr for S {}
"#,
        Some(vec!["ra_test_fixture::Tr"]),
    );
}

#[test]
fn struct_with_derived_clone() {
    check_struct_implemented_traits(
        r#"
//- minicore: clone, derive
#[derive(Clone)]
struct $0S;

fn main() {}
"#,
        Some(vec!["core::clone::Clone"]),
    );
}

#[test]
fn local_struct_with_trait_impl() {
    check_struct_implemented_traits(
        r#"
trait Tr<T> {}

fn f() {
    struct $0S;
    struct T;
    impl Tr<T> for S {}
}

fn main() { f(); }
"#,
        Some(vec!["ra_test_fixture::Tr<ra_test_fixture::f::T>"]),
    );
}

#[test]
fn struct_with_partial_eq_impl() {
    check_struct_implemented_traits(
        r#"
//- minicore: eq, derive
struct $0S;

impl PartialEq for S {
    fn eq(&self, _: &S) -> bool { true }
}

#[derive(PartialEq)]
struct T;

fn main() {}
"#,
        Some(vec!["core::cmp::PartialEq<ra_test_fixture::S>"]),
    );
}

#[test]
fn struct_with_derived_partial_eq() {
    check_struct_implemented_traits(
        r#"
//- minicore: eq, derive
struct S;

impl PartialEq for S {
    fn eq(&self, _: &S) -> bool { true }
}

#[derive(PartialEq)]
struct $0T;

fn main() {}
"#,
        Some(vec!["core::cmp::PartialEq<ra_test_fixture::T>"]),
    );
}

#[test]
fn trait_impl_with_lifetime_type_and_const_args() {
    check_struct_implemented_traits(
        r#"
trait Tr<'a, T, const N: usize> {
    fn m(&self);
}

struct $0S;

impl<'a> Tr<'a, u8, 3> for S {
    fn m(&self) {}
}

fn f(s: S) {
    s.m();
}

fn main() {}
"#,
        Some(vec!["ra_test_fixture::Tr<'a, u8, 3>"]),
    );
}

#[test]
fn trait_impls_per_const_arg() {
    check_struct_implemented_traits(
        r#"
trait Tr<const N: usize> {
    fn m(&self);
}

struct $0S;

impl Tr<3> for S {
    fn m(&self) {}
}

impl Tr<4> for S {
    fn m(&self) {}
}

fn f(s: S) {
    <S as Tr<3>>::m(&s);
    Tr::<4>::m(&s);
}

fn main() {}
"#,
        Some(vec!["ra_test_fixture::Tr<3>", "ra_test_fixture::Tr<4>"]),
    );
}

#[test]
fn local_struct_in_sibling_block_1() {
    check_struct_type_full_name(
        r#"
fn f() {
    if true {
        struct $0S { x: i32 }
        impl S { fn new() -> S { S { x: 1 } } }
        let a = S::new();
        let _ = a.x;
    }
    if false {
        struct S { x: u8 }
        impl S { fn new() -> S { S { x: 2 } } }
        let b = S::new();
        let _ = b.x;
    }
}

fn main() { f(); }
"#,
        "ra_test_fixture::f::S#1",
    );
}

#[test]
fn local_struct_in_sibling_block_2() {
    check_struct_type_full_name(
        r#"
fn f() {
    if true {
        struct S { x: i32 }
        impl S { fn new() -> S { S { x: 1 } } }
        let a = S::new();
        let _ = a.x;
    }
    if false {
        struct $0S { x: u8 }
        impl S { fn new() -> S { S { x: 2 } } }
        let b = S::new();
        let _ = b.x;
    }
}

fn main() { f(); }
"#,
        "ra_test_fixture::f::S#2",
    );
}

#[test]
fn local_struct_in_sibling_anonymous_const_1() {
    check_struct_type_full_name(
        r#"
const _: () = {
    struct $0S { x: i32 }
    impl S { fn new() -> S { S { x: 1 } } }
    fn make_a() -> S { S::new() }
};
const _: () = {
    struct S { x: u8 }
    impl S { fn new() -> S { S { x: 2 } } }
    fn make_b() -> S { S::new() }
};

fn main() {}
"#,
        "ra_test_fixture::S#1",
    );
}

#[test]
fn local_struct_in_sibling_anonymous_const_2() {
    check_struct_type_full_name(
        r#"
const _: () = {
    struct S { x: i32 }
    impl S { fn new() -> S { S { x: 1 } } }
    fn make_a() -> S { S::new() }
};
const _: () = {
    struct $0S { x: u8 }
    impl S { fn new() -> S { S { x: 2 } } }
    fn make_b() -> S { S::new() }
};

fn main() {}
"#,
        "ra_test_fixture::S#2",
    );
}
