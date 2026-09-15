use super::check_impl_type_full_name;

#[test]
fn inherent_impl_with_elided_lifetime() {
    check_impl_type_full_name(
        r#"
struct P<'a>(&'a u8);

$0impl P<'_> {
    fn m(&self) {}
}
"#,
        "ra_test_fixture::P<'a>",
    );
}

#[test]
fn trait_impl_with_lifetime_type_and_const_args() {
    check_impl_type_full_name(
        r#"
trait Tr<'a, T, const N: usize> {
    fn m(&self);
}

struct S;

$0impl<'a> Tr<'a, u8, 3> for S {
    fn m(&self) {}
}

fn f(s: S) {
    s.m();
}

fn main() {}
"#,
        "<ra_test_fixture::S as ra_test_fixture::Tr<'a, u8, 3>>",
    );
}

#[test]
fn trait_impls_per_const_arg_1() {
    check_impl_type_full_name(
        r#"
trait Tr<const N: usize> {
    fn m(&self);
}

struct S;

$0impl Tr<3> for S {
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
        "<ra_test_fixture::S as ra_test_fixture::Tr<3>>",
    );
}

#[test]
fn trait_impls_per_const_arg_2() {
    check_impl_type_full_name(
        r#"
trait Tr<const N: usize> {
    fn m(&self);
}

struct S;

impl Tr<3> for S {
    fn m(&self) {}
}

$0impl Tr<4> for S {
    fn m(&self) {}
}

fn f(s: S) {
    <S as Tr<3>>::m(&s);
    Tr::<4>::m(&s);
}

fn main() {}
"#,
        "<ra_test_fixture::S as ra_test_fixture::Tr<4>>",
    );
}

#[test]
fn inherent_impl_on_type_alias() {
    check_impl_type_full_name(
        r#"
struct S;

type A = S;

$0impl A {
    fn m(&self) {}
}
"#,
        "ra_test_fixture::S",
    );
}

#[test]
fn trait_impl_on_slice() {
    check_impl_type_full_name(
        r#"
trait Tr {}

struct S;

$0impl Tr for [S] {}
"#,
        "<[ra_test_fixture::S] as ra_test_fixture::Tr>",
    );
}

#[test]
fn trait_impl_with_method() {
    check_impl_type_full_name(
        r#"
trait Tr {
    fn m(&self);
}

struct S;

$0impl Tr for S {
    fn m(&self) {}
}
"#,
        "<ra_test_fixture::S as ra_test_fixture::Tr>",
    );
}
