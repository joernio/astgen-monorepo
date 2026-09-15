use super::check_fn_method_full_name;

#[test]
fn empty_main() {
    check_fn_method_full_name(
        r#"
fn $0main() {}
"#,
        "ra_test_fixture::main",
    );
}

#[test]
fn free_fn_in_custom_crate() {
    check_fn_method_full_name(
        r#"
//- /lib.rs crate:my_crate
fn $0foo() {}
    "#,
        "my_crate::foo",
    )
}

#[test]
fn method_without_body_in_trait() {
    check_fn_method_full_name(
        r#"
trait Greet {
    fn $0hello(&self) -> bool;
}

fn main() {}
"#,
        "ra_test_fixture::Greet::hello",
    );
}

#[test]
fn method_with_body_in_trait() {
    check_fn_method_full_name(
        r#"
trait Greet {
    fn $0hello(&self) -> bool { true }
}

fn main() {}
"#,
        "ra_test_fixture::Greet::hello",
    );
}

#[test]
fn free_fn() {
    check_fn_method_full_name(
        r#"
fn $0foo() -> u32 { 1 }

fn main() {
    let value = foo();
}
"#,
        "ra_test_fixture::foo",
    );
}

#[test]
fn inherent_method() {
    check_fn_method_full_name(
        r#"
struct Foo;

impl Foo {
    fn $0foo(&self) -> u32 { 1 }
}

fn main() {
    let value = Foo.foo();
}
"#,
        "ra_test_fixture::Foo::foo",
    );
}

#[test]
fn free_fn_in_mod() {
    check_fn_method_full_name(
        r#"
mod inner {
    pub fn $0foo() -> u32 { 1 }
}

fn main() {
    let value = inner::foo();
}
"#,
        "ra_test_fixture::inner::foo",
    );
}

#[test]
fn generic_free_fn() {
    check_fn_method_full_name(
        r#"
fn $0identity<T>(value: T) -> T { value }

fn main() {
    let value = identity::<u32>(1);
}
"#,
        "ra_test_fixture::identity<T>",
    );
}

#[test]
fn inherent_fn_in_generic_impl() {
    check_fn_method_full_name(
        r#"
struct Wrapper<T>(T);

impl<T> Wrapper<T> {
    fn $0new(value: T) -> Wrapper<T> { Wrapper(value) }
}

fn main() {}
"#,
        "ra_test_fixture::Wrapper<T>::new",
    );
}

#[test]
fn inherent_fn() {
    check_fn_method_full_name(
        r#"
struct Foo;

impl Foo {
    fn $0new() -> Foo { Foo }
}

fn main() {
    let value = Foo::new();
}
"#,
        "ra_test_fixture::Foo::new",
    );
}

#[test]
fn trait_impl_fn() {
    check_fn_method_full_name(
        r#"
struct Foo;

trait Make {
    fn make() -> Foo;
}

impl Make for Foo {
    fn $0make() -> Foo { Foo }
}

fn main() {}
"#,
        "<ra_test_fixture::Foo as ra_test_fixture::Make>::make",
    );
}

#[test]
fn trait_impl_method() {
    check_fn_method_full_name(
        r#"
struct Foo;

trait Greet {
    fn hello(&self) -> bool;
}

impl Greet for Foo {
    fn $0hello(&self) -> bool { true }
}

fn main() {
    let foo = Foo;
    let value = foo.hello();
}
"#,
        "<ra_test_fixture::Foo as ra_test_fixture::Greet>::hello",
    );
}

#[test]
fn local_fn() {
    check_fn_method_full_name(
        r#"
fn main() {
    fn $0helper() -> u32 { 1 }
    let value = helper();
}
"#,
        "ra_test_fixture::main::helper",
    );
}

#[test]
fn local_fn_in_sibling_block_1() {
    check_fn_method_full_name(
        r#"
fn f() {
    if true {
        fn $0g() -> u32 { 1 }
        let _a = g();
    }
    if false {
        fn g() -> u8 { 2 }
        let _b = g();
    }
}

fn main() { f(); }
"#,
        "ra_test_fixture::f::g#1",
    );
}

#[test]
fn local_fn_in_sibling_block_2() {
    check_fn_method_full_name(
        r#"
fn f() {
    if true {
        fn g() -> u32 { 1 }
        let _a = g();
    }
    if false {
        fn $0g() -> u8 { 2 }
        let _b = g();
    }
}

fn main() { f(); }
"#,
        "ra_test_fixture::f::g#2",
    );
}

#[test]
fn local_fn_in_sibling_anonymous_const_1() {
    check_fn_method_full_name(
        r#"
const _: () = {
    fn $0helper() -> u32 { 1 }
    fn __ctor() -> u32 { helper() }
};
const _: () = {
    fn helper() -> u8 { 2 }
    fn __ctor() -> u8 { helper() }
};

fn main() {}
"#,
        "ra_test_fixture::helper#1",
    );
}

#[test]
fn local_fn_in_sibling_anonymous_const_2() {
    check_fn_method_full_name(
        r#"
const _: () = {
    fn helper() -> u32 { 1 }
    fn __ctor() -> u32 { helper() }
};
const _: () = {
    fn $0helper() -> u8 { 2 }
    fn __ctor() -> u8 { helper() }
};

fn main() {}
"#,
        "ra_test_fixture::helper#2",
    );
}

#[test]
fn local_caller_in_sibling_anonymous_const_1() {
    check_fn_method_full_name(
        r#"
const _: () = {
    fn helper() -> u32 { 1 }
    fn $0__ctor() -> u32 { helper() }
};
const _: () = {
    fn helper() -> u8 { 2 }
    fn __ctor() -> u8 { helper() }
};

fn main() {}
"#,
        "ra_test_fixture::__ctor#1",
    );
}

#[test]
fn local_caller_in_sibling_anonymous_const_2() {
    check_fn_method_full_name(
        r#"
const _: () = {
    fn helper() -> u32 { 1 }
    fn __ctor() -> u32 { helper() }
};
const _: () = {
    fn helper() -> u8 { 2 }
    fn $0__ctor() -> u8 { helper() }
};

fn main() {}
"#,
        "ra_test_fixture::__ctor#2",
    );
}

#[test]
fn inherent_fn_as_value() {
    check_fn_method_full_name(
        r#"
struct Foo;

impl Foo {
    fn $0new() -> Foo { Foo }
}

fn main() {
    let f = Foo::new;
}
"#,
        "ra_test_fixture::Foo::new",
    );
}

#[test]
fn aliased_free_fn_in_mod() {
    check_fn_method_full_name(
        r#"
mod inner {
    pub fn $0helper() -> u32 { 1 }
}

use inner::helper as aliased;

fn main() {
    let f = aliased;
}
"#,
        "ra_test_fixture::inner::helper",
    );
}

#[test]
fn generic_trait_impl_method_called_via_qualified_path() {
    check_fn_method_full_name(
        r#"
trait Tr<T> {
    fn m(&self) -> T;
}
struct S<T>(T);
impl<T: Copy> Tr<T> for S<T> {
    fn $0m(&self) -> T { self.0 }
}
fn f(w: S<u32>) {
    let a = <S<u32> as Tr<u32>>::m(&w);
}
"#,
        "<ra_test_fixture::S<T> as ra_test_fixture::Tr<T>>::m",
    );
}

#[test]
fn trait_impl_for_lifetime_generic_struct() {
    check_fn_method_full_name(
        r#"
trait Tr {
    fn m(&self);
}

struct Mix<'a, T> {
    value: &'a T,
}

impl<'a, T> Tr for Mix<'a, T> {
    fn $0m(&self) {}
}

fn main() {}
"#,
        "<ra_test_fixture::Mix<'a, T> as ra_test_fixture::Tr>::m",
    );
}

#[test]
fn trait_impl_for_const_generic_struct() {
    check_fn_method_full_name(
        r#"
trait Tr {
    fn m(&self);
}

struct Foo<const N: usize>;

struct Pair<T, const N: usize> {
    value: T,
}

impl<const N: usize> Tr for Foo<N> {
    fn $0m(&self) {}
}

fn concrete(value: &Foo<3>) {}

fn expression(value: &Foo<{ 2 + 1 }>) {}

fn mixed<const N: usize>(value: &Pair<u32, N>) {}

fn main() {}
"#,
        "<ra_test_fixture::Foo<N> as ra_test_fixture::Tr>::m",
    );
}

#[test]
fn inherent_impl_with_elided_lifetime() {
    check_fn_method_full_name(
        r#"
struct P<'a>(&'a u8);

impl P<'_> {
    fn $0m(&self) {}
}
"#,
        "ra_test_fixture::P<'a>::m",
    );
}

#[test]
fn trait_impl_with_lifetime_type_and_const_args() {
    check_fn_method_full_name(
        r#"
trait Tr<'a, T, const N: usize> {
    fn m(&self);
}

struct S;

impl<'a> Tr<'a, u8, 3> for S {
    fn $0m(&self) {}
}

fn f(s: S) {
    s.m();
}

fn main() {}
"#,
        "<ra_test_fixture::S as ra_test_fixture::Tr<'a, u8, 3>>::m",
    );
}

#[test]
fn trait_impl_method_called_via_qualified_path() {
    check_fn_method_full_name(
        r#"
trait Tr {
    fn m(&self) -> i32;
}
struct S;
impl Tr for S {
    fn $0m(&self) -> i32 { 0 }
}
fn f(s: S) {
    let a = <S as Tr>::m(&s);
}
"#,
        "<ra_test_fixture::S as ra_test_fixture::Tr>::m",
    );
}

#[test]
fn trait_impl_method_called_via_trait() {
    check_fn_method_full_name(
        r#"
trait Tr {
    fn m(&self) -> i32;
}
struct S;
impl Tr for S {
    fn $0m(&self) -> i32 { 0 }
}
fn f(s: S) {
    let b = Tr::m(&s);
}
"#,
        "<ra_test_fixture::S as ra_test_fixture::Tr>::m",
    );
}

#[test]
fn trait_method_called_via_trait_on_dyn() {
    check_fn_method_full_name(
        r#"
trait Tr {
    fn $0m(&self) -> i32;
}
struct S;
impl Tr for S {
    fn m(&self) -> i32 { 0 }
}
fn f(g: &dyn Tr) {
    let c = Tr::m(g);
}
"#,
        "ra_test_fixture::Tr::m",
    );
}

#[test]
fn local_struct_in_sibling_anonymous_const_1() {
    check_fn_method_full_name(
        r#"
const _: () = {
    struct S { x: i32 }
    impl S { fn $0new() -> S { S { x: 1 } } }
    fn make_a() -> S { S::new() }
};
const _: () = {
    struct S { x: u8 }
    impl S { fn new() -> S { S { x: 2 } } }
    fn make_b() -> S { S::new() }
};

fn main() {}
"#,
        "ra_test_fixture::S#1::new",
    );
}

#[test]
fn local_struct_in_sibling_anonymous_const_2() {
    check_fn_method_full_name(
        r#"
const _: () = {
    struct S { x: i32 }
    impl S { fn new() -> S { S { x: 1 } } }
    fn make_a() -> S { S::new() }
};
const _: () = {
    struct S { x: u8 }
    impl S { fn $0new() -> S { S { x: 2 } } }
    fn make_b() -> S { S::new() }
};

fn main() {}
"#,
        "ra_test_fixture::S#2::new",
    );
}

#[test]
fn trait_impl_for_assoc_type() {
    check_fn_method_full_name(
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
    fn m(&self);
}
impl Sink for <S as Tr>::A {
    fn $0m(&self) {}
}
fn f(t: T) {
    t.m();
}
"#,
        "<ra_test_fixture::T as ra_test_fixture::Sink>::m",
    );
}

#[test]
fn inherent_impl_on_type_alias() {
    check_fn_method_full_name(
        r#"
struct S;

type A = S;

impl A {
    fn $0m(&self) {}
}
"#,
        "ra_test_fixture::S::m",
    );
}

#[test]
fn trait_impl_with_method() {
    check_fn_method_full_name(
        r#"
trait Tr {
    fn m(&self);
}

struct S;

impl Tr for S {
    fn $0m(&self) {}
}
"#,
        "<ra_test_fixture::S as ra_test_fixture::Tr>::m",
    );
}
