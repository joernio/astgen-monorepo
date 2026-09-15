use super::check_name_ref_type_full_name;

#[test]
fn trait_in_trait_impl() {
    check_name_ref_type_full_name(
        r#"
trait Foo {}

struct Bar;

impl $0Foo for Bar {}

fn main() {}
"#,
        "ra_test_fixture::Foo",
    );
}

#[test]
fn struct_in_trait_impl() {
    check_name_ref_type_full_name(
        r#"
trait Foo {}

struct Bar;

impl Foo for $0Bar {}

fn main() {}
"#,
        "ra_test_fixture::Bar",
    );
}

#[test]
fn generic_trait_in_trait_impl() {
    check_name_ref_type_full_name(
        r#"
trait Extract<T> {}

struct Bar;

impl $0Extract<i32> for Bar {}

fn main() {}
"#,
        "ra_test_fixture::Extract<i32>",
    );
}

#[test]
fn struct_in_generic_trait_impl() {
    check_name_ref_type_full_name(
        r#"
trait Extract<T> {}

struct Bar;

impl Extract<i32> for $0Bar {}

fn main() {}
"#,
        "ra_test_fixture::Bar",
    );
}

#[test]
fn struct_in_turbofish_inherent_fn_call() {
    check_name_ref_type_full_name(
        r#"
struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        Wrapper(value)
    }
}

fn main() {
    let wrapped = $0Wrapper::<u32>::new(1);
}
"#,
        "ra_test_fixture::Wrapper<u32>",
    );
}

#[test]
fn fn_in_turbofish_inherent_fn_call() {
    check_name_ref_type_full_name(
        r#"
struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        Wrapper(value)
    }
}

fn main() {
    let wrapped = Wrapper::<u32>::$0new(1);
}
"#,
        "fn(u32) -> ra_test_fixture::Wrapper<u32>",
    );
}

#[test]
fn generic_tuple_struct_in_impl() {
    check_name_ref_type_full_name(
        r#"
struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        $0Wrapper(value)
    }
}

fn main() {}
"#,
        "fn(T) -> ra_test_fixture::Wrapper<T>",
    );
}

#[test]
fn struct_in_generic_inherent_impl_header() {
    check_name_ref_type_full_name(
        r#"
trait Extract<T> {
    fn extract(&self) -> T;
}

struct Wrapper<T>(T);

impl<T: Copy> $0Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        Wrapper(value)
    }
}

impl<T: Copy> Extract<T> for Wrapper<T> {
    fn extract(&self) -> T {
        self.0
    }
}

fn main() {}
"#,
        "ra_test_fixture::Wrapper<T>",
    );
}

#[test]
fn struct_in_generic_fn_return_type() {
    check_name_ref_type_full_name(
        r#"
trait Extract<T> {
    fn extract(&self) -> T;
}

struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> $0Wrapper<T> {
        Wrapper(value)
    }
}

impl<T: Copy> Extract<T> for Wrapper<T> {
    fn extract(&self) -> T {
        self.0
    }
}

fn main() {}
"#,
        "ra_test_fixture::Wrapper<T>",
    );
}

#[test]
fn struct_in_generic_trait_impl_header() {
    check_name_ref_type_full_name(
        r#"
trait Extract<T> {
    fn extract(&self) -> T;
}

struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        Wrapper(value)
    }
}

impl<T: Copy> Extract<T> for $0Wrapper<T> {
    fn extract(&self) -> T {
        self.0
    }
}

fn main() {}
"#,
        "ra_test_fixture::Wrapper<T>",
    );
}

#[test]
fn moved_generic_tuple_struct() {
    check_name_ref_type_full_name(
        r#"
struct Wrapper<T>(T);

fn main() {
    let plain = Wrapper(3u32);
    let copied = $0plain;
}
"#,
        "ra_test_fixture::Wrapper<u32>",
    );
}

#[test]
fn self_in_generic_trait() {
    check_name_ref_type_full_name(
        r#"
trait Tr<T> {
    fn m() -> $0Self;
}

fn main() {}
"#,
        "ra_test_fixture::Tr<T>",
    );
}

#[test]
fn trait_impl_for_lifetime_generic_struct() {
    check_name_ref_type_full_name(
        r#"
trait Tr {
    fn m(&self);
}

struct Mix<'a, T> {
    value: &'a T,
}

impl<'a, T> Tr for $0Mix<'a, T> {
    fn m(&self) {}
}

fn main() {}
"#,
        "ra_test_fixture::Mix<'a, T>",
    );
}

#[test]
fn const_generic_struct_with_literal_arg() {
    check_name_ref_type_full_name(
        r#"
trait Tr {
    fn m(&self);
}

struct Foo<const N: usize>;

struct Pair<T, const N: usize> {
    value: T,
}

impl<const N: usize> Tr for Foo<N> {
    fn m(&self) {}
}

fn concrete(value: &$0Foo<3>) {}

fn expression(value: &Foo<{ 2 + 1 }>) {}

fn mixed<const N: usize>(value: &Pair<u32, N>) {}

fn main() {}
"#,
        "ra_test_fixture::Foo<3>",
    );
}

#[test]
fn const_generic_struct_with_expr_arg() {
    check_name_ref_type_full_name(
        r#"
trait Tr {
    fn m(&self);
}

struct Foo<const N: usize>;

struct Pair<T, const N: usize> {
    value: T,
}

impl<const N: usize> Tr for Foo<N> {
    fn m(&self) {}
}

fn concrete(value: &Foo<3>) {}

fn expression(value: &$0Foo<{ 2 + 1 }>) {}

fn mixed<const N: usize>(value: &Pair<u32, N>) {}

fn main() {}
"#,
        "ra_test_fixture::Foo<{ 2 + 1 }>",
    );
}

#[test]
fn trait_impl_for_const_generic_struct() {
    check_name_ref_type_full_name(
        r#"
trait Tr {
    fn m(&self);
}

struct Foo<const N: usize>;

struct Pair<T, const N: usize> {
    value: T,
}

impl<const N: usize> Tr for $0Foo<N> {
    fn m(&self) {}
}

fn concrete(value: &Foo<3>) {}

fn expression(value: &Foo<{ 2 + 1 }>) {}

fn mixed<const N: usize>(value: &Pair<u32, N>) {}

fn main() {}
"#,
        "ra_test_fixture::Foo<N>",
    );
}

#[test]
fn struct_with_type_and_const_param_arg() {
    check_name_ref_type_full_name(
        r#"
trait Tr {
    fn m(&self);
}

struct Foo<const N: usize>;

struct Pair<T, const N: usize> {
    value: T,
}

impl<const N: usize> Tr for Foo<N> {
    fn m(&self) {}
}

fn concrete(value: &Foo<3>) {}

fn expression(value: &Foo<{ 2 + 1 }>) {}

fn mixed<const N: usize>(value: &$0Pair<u32, N>) {}

fn main() {}
"#,
        "ra_test_fixture::Pair<u32, N>",
    );
}

#[test]
fn dyn_trait_without_assoc_type_binding() {
    check_name_ref_type_full_name(
        r#"
trait Tr {
    type Assoc;
}

struct Foo;

impl Tr for Foo {
    type Assoc = u32;
}

fn bare(value: &dyn $0Tr) {}

fn bound(value: &dyn Tr<Assoc = u32>) {}

fn main() {}
"#,
        "ra_test_fixture::Tr",
    );
}

#[test]
fn dyn_trait_with_assoc_type_binding() {
    check_name_ref_type_full_name(
        r#"
trait Tr {
    type Assoc;
}

struct Foo;

impl Tr for Foo {
    type Assoc = u32;
}

fn bare(value: &dyn Tr) {}

fn bound(value: &dyn $0Tr<Assoc = u32>) {}

fn main() {}
"#,
        "ra_test_fixture::Tr<Assoc = u32>",
    );
}

#[test]
fn lifetime_generic_struct_with_named_lifetime() {
    check_name_ref_type_full_name(
        r#"
struct Foo<'a> {
    value: &'a str,
}

fn named<'a>(value: &$0Foo<'a>) {}

fn elided(value: &Foo<'_>) {}

fn borrowed_static(value: &'static Foo<'static>) {}

fn main() {}
"#,
        "ra_test_fixture::Foo<'a>",
    );
}

#[test]
fn lifetime_generic_struct_with_elided_lifetime() {
    check_name_ref_type_full_name(
        r#"
struct Foo<'a> {
    value: &'a str,
}

fn named<'a>(value: &Foo<'a>) {}

fn elided(value: &$0Foo<'_>) {}

fn borrowed_static(value: &'static Foo<'static>) {}

fn main() {}
"#,
        "ra_test_fixture::Foo<'_>",
    );
}

#[test]
fn lifetime_generic_struct_with_static_lifetime() {
    check_name_ref_type_full_name(
        r#"
struct Foo<'a> {
    value: &'a str,
}

fn named<'a>(value: &Foo<'a>) {}

fn elided(value: &Foo<'_>) {}

fn borrowed_static(value: &'static $0Foo<'static>) {}

fn main() {}
"#,
        "ra_test_fixture::Foo<'static>",
    );
}

#[test]
fn trait_impls_for_two_lifetime_generic_structs_1() {
    check_name_ref_type_full_name(
        r#"
trait Tr {
    fn m(&self);
}

struct Alpha<'a> {
    value: &'a str,
}

struct Beta<'a> {
    value: &'a str,
}

impl<'a> Tr for $0Alpha<'a> {
    fn m(&self) {}
}

impl<'a> Tr for Beta<'a> {
    fn m(&self) {}
}

fn main() {}
"#,
        "ra_test_fixture::Alpha<'a>",
    );
}

#[test]
fn trait_impls_for_two_lifetime_generic_structs_2() {
    check_name_ref_type_full_name(
        r#"
trait Tr {
    fn m(&self);
}

struct Alpha<'a> {
    value: &'a str,
}

struct Beta<'a> {
    value: &'a str,
}

impl<'a> Tr for Alpha<'a> {
    fn m(&self) {}
}

impl<'a> Tr for $0Beta<'a> {
    fn m(&self) {}
}

fn main() {}
"#,
        "ra_test_fixture::Beta<'a>",
    );
}

#[test]
fn generic_enum_variant_in_pattern() {
    check_name_ref_type_full_name(
        r#"
enum E<T> { A(T) }

fn main() {
    let a = E::A(1);
    if let E::$0A(v) = a {}
}
"#,
        "ra_test_fixture::E<T>::A",
    );
}

#[test]
fn imported_struct_in_inherent_impl_header() {
    check_name_ref_type_full_name(
        r#"
mod imported {
    pub struct Type;
}

use imported::Type;

impl $0Type {
    fn value(&self) -> bool {
        true
    }
}

fn main() {
    let receiver = Type;
    let method_value = receiver.value();
}
"#,
        "ra_test_fixture::imported::Type",
    );
}

#[test]
fn imported_unit_struct_as_value() {
    check_name_ref_type_full_name(
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
    let receiver = $0Type;
    let method_value = receiver.value();
}
"#,
        "ra_test_fixture::imported::Type",
    );
}

#[test]
fn imported_struct_in_trait_impl_header() {
    check_name_ref_type_full_name(
        r#"
mod imported {
    pub struct Type;

    pub trait Trait {
        fn trait_value(&self) -> bool;
    }
}

use imported::{Trait, Type};

impl Trait for $0Type {
    fn trait_value(&self) -> bool {
        true
    }
}

fn main() {
    let trait_receiver = Type;
    let trait_value = trait_receiver.trait_value();
}
"#,
        "ra_test_fixture::imported::Type",
    );
}

#[test]
fn str_ref() {
    check_name_ref_type_full_name(
        r#"
fn main() {
    let text: &str = "hello";
    let text_copy = $0text;
    let mut number = 1u32;
    let number_ref: &mut u32 = &mut number;
    let number_ref_copy = number_ref;
}
"#,
        "&str",
    );
}

#[test]
fn mut_ref() {
    check_name_ref_type_full_name(
        r#"
fn main() {
    let text: &str = "hello";
    let text_copy = text;
    let mut number = 1u32;
    let number_ref: &mut u32 = &mut number;
    let number_ref_copy = $0number_ref;
}
"#,
        "&mut u32",
    );
}

#[test]
fn self_in_trait() {
    check_name_ref_type_full_name(
        r#"
trait Tr {
    fn m() -> $0Self;
}

fn main() {}
"#,
        "ra_test_fixture::Tr",
    );
}

#[test]
fn self_in_trait_impl_for_assoc_type() {
    check_name_ref_type_full_name(
        r#"
struct S;
struct T;
trait Tr {
    type A;
}
impl Tr for S {
    type A = T;
}
trait Sink {
    fn m() -> Self;
}
impl Sink for <S as Tr>::A {
    fn m() -> $0Self { T }
}
"#,
        "ra_test_fixture::T",
    );
}

#[test]
fn assoc_type_in_trait_impl_header() {
    check_name_ref_type_full_name(
        r#"
struct S;
struct T;
trait Tr {
    type A;
}
impl Tr for S {
    type A = T;
}
trait Sink {}
impl Sink for <S as Tr>::$0A {}
"#,
        "ra_test_fixture::T",
    );
}

#[test]
fn assoc_type_of_type_param() {
    check_name_ref_type_full_name(
        r#"
trait Tr {
    type A;
}
fn f<X: Tr>(x: <X as Tr>::$0A) {}
"#,
        "ra_test_fixture::Tr::A",
    );
}

#[test]
fn tuple_enum_variant_in_pattern() {
    check_name_ref_type_full_name(
        r#"
enum E { A(i32), B { x: i32 } }

fn main() {
    let a = E::A(1);
    if let E::$0A(v) = a {}
    let b = E::B { x: 1 };
    if let E::B { x } = b {}
}
"#,
        "ra_test_fixture::E::A",
    );
}

#[test]
fn record_enum_variant_in_pattern() {
    check_name_ref_type_full_name(
        r#"
enum E { A(i32), B { x: i32 } }

fn main() {
    let a = E::A(1);
    if let E::A(v) = a {}
    let b = E::B { x: 1 };
    if let E::$0B { x } = b {}
}
"#,
        "ra_test_fixture::E::B",
    );
}
