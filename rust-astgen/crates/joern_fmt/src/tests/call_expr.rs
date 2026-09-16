use super::{check_call_has_self_receiver, check_call_method_full_name, check_call_type_full_name};

#[test]
fn free_fn() {
    check_call_method_full_name(
        r#"
fn foo() -> u32 { 1 }

fn main() {
    let value = $0foo();
}
"#,
        "ra_test_fixture::foo",
    );
}

#[test]
fn free_fn_in_mod() {
    check_call_method_full_name(
        r#"
mod inner {
    pub fn foo() -> u32 { 1 }
}

fn main() {
    let value = inner::$0foo();
}
"#,
        "ra_test_fixture::inner::foo",
    );
}

#[test]
fn generic_free_fn() {
    check_call_method_full_name(
        r#"
fn identity<T>(value: T) -> T { value }

fn main() {
    let value = $0identity::<u32>(1);
}
"#,
        "ra_test_fixture::identity<T>",
    );
}

#[test]
fn inherent_fn() {
    check_call_method_full_name(
        r#"
struct Foo;

impl Foo {
    fn new() -> Foo { Foo }
}

fn main() {
    let value = Foo::$0new();
}
"#,
        "ra_test_fixture::Foo::new",
    );
}

#[test]
fn local_fn() {
    check_call_method_full_name(
        r#"
fn main() {
    fn helper() -> u32 { 1 }
    let value = $0helper();
}
"#,
        "ra_test_fixture::main::helper",
    );
}

#[test]
fn local_fn_in_sibling_block_1() {
    check_call_method_full_name(
        r#"
fn f() {
    if true {
        fn g() -> u32 { 1 }
        let _a = $0g();
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
    check_call_method_full_name(
        r#"
fn f() {
    if true {
        fn g() -> u32 { 1 }
        let _a = g();
    }
    if false {
        fn g() -> u8 { 2 }
        let _b = $0g();
    }
}

fn main() { f(); }
"#,
        "ra_test_fixture::f::g#2",
    );
}

#[test]
fn local_fn_in_sibling_anonymous_const_1() {
    check_call_method_full_name(
        r#"
const _: () = {
    fn helper() -> u32 { 1 }
    fn __ctor() -> u32 { $0helper() }
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
    check_call_method_full_name(
        r#"
const _: () = {
    fn helper() -> u32 { 1 }
    fn __ctor() -> u32 { helper() }
};
const _: () = {
    fn helper() -> u8 { 2 }
    fn __ctor() -> u8 { $0helper() }
};

fn main() {}
"#,
        "ra_test_fixture::helper#2",
    );
}

#[test]
fn free_fn_as_callee() {
    check_call_method_full_name(
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
fn tuple_struct() {
    check_call_method_full_name(
        r#"
struct Plain(i32, bool);

fn main() {
    let _value = $0Plain(1, true);
}
"#,
        "ra_test_fixture::Plain",
    );
}

#[test]
fn generic_tuple_struct() {
    check_call_method_full_name(
        r#"
struct OneParam<T>(T);

fn main() {
    let _value = $0OneParam(1u32);
}
"#,
        "ra_test_fixture::OneParam<T>",
    );
}

#[test]
fn tuple_struct_with_two_type_params() {
    check_call_method_full_name(
        r#"
struct Multi<A, B>(A, B);

fn main() {
    let _value = $0Multi(1u32, true);
}
"#,
        "ra_test_fixture::Multi<A, B>",
    );
}

#[test]
fn tuple_struct_with_bounded_type_param() {
    check_call_method_full_name(
        r#"
struct Bounded<T: Clone>(T);

fn main() {
    let _value = $0Bounded(1u32);
}
"#,
        "ra_test_fixture::Bounded<T>",
    );
}

#[test]
fn tuple_struct_with_lifetime_param() {
    check_call_method_full_name(
        r#"
struct WithLife<'a>(&'a i32);

fn main() {
    let _value = $0WithLife(&1i32);
}
"#,
        "ra_test_fixture::WithLife",
    );
}

#[test]
fn tuple_struct_with_lifetime_and_type_params() {
    check_call_method_full_name(
        r#"
struct Mix<'a, T>(&'a T);

fn main() {
    let _value = $0Mix(&1u32);
}
"#,
        "ra_test_fixture::Mix<T>",
    );
}

#[test]
fn tuple_struct_with_defaulted_type_param() {
    check_call_method_full_name(
        r#"
struct WithDefault<T = i32>(T);

fn main() {
    let _value = $0WithDefault(1i32);
}
"#,
        "ra_test_fixture::WithDefault<T>",
    );
}

// TODO: `const` params have not been dealt with yet, just recording the status quo.
#[test]
fn tuple_struct_with_const_param() {
    check_call_method_full_name(
        r#"
struct WithConst<const N: usize>(usize);

fn main() {
    let _value = $0WithConst::<5>(0usize);
}
"#,
        "ra_test_fixture::WithConst<N>",
    );
}

#[test]
fn local_tuple_struct() {
    check_call_method_full_name(
        r#"
fn f() {
    struct S(i32);
    let _ = $0S(1);
}

fn main() { f(); }
"#,
        "ra_test_fixture::f::S",
    );
}

#[test]
fn inherent_method_called_as_fn() {
    check_call_has_self_receiver(
        r#"
struct Circle {
    radius: f64,
}

impl Circle {
    // A standard method that takes an immutable reference to self
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

fn main() {
    let my_circle = Circle { radius: 5.0 };

    // --- 1. Methods taking &self ---

    // Option A: Traditional method/receiver syntax
    let area1 = my_circle.area();

    // Option B: Regular function syntax (Fully Qualified Syntax)
    // We must explicitly pass the reference because the function expects `&self`
    let area2 = $0Circle::area(&my_circle);
}
"#,
        Some(true),
    );
}

#[test]
fn trait_method_called_via_trait() {
    check_call_has_self_receiver(
        r#"
struct Type;
trait Tr { fn m(&self) -> bool; }
impl Tr for Type { fn m(&self) -> bool { true } }
fn main() { let t = Type; let _ = $0Tr::m(&t); }
"#,
        Some(true),
    );
}

#[test]
fn trait_method_called_via_qualified_path() {
    check_call_has_self_receiver(
        r#"
struct Type;
trait Tr { fn m(&self) -> bool; }
impl Tr for Type { fn m(&self) -> bool { true } }
fn main() { let t = Type; let _ = $0<Type as Tr>::m(&t); }
"#,
        Some(true),
    );
}

#[test]
fn free_fn_with_args() {
    check_call_has_self_receiver(
        r#"
fn free(a: i32, b: i32) -> i32 { a + b }
fn main() { let _ = $0free(1, 2); }
"#,
        None,
    );
}

#[test]
fn inherent_fn_without_self() {
    check_call_has_self_receiver(
        r#"
struct Type;
impl Type { fn new() -> Type { Type } }
fn main() { let _ = $0Type::new(); }
"#,
        None,
    );
}

#[test]
fn tuple_struct_with_two_fields() {
    check_call_has_self_receiver(
        r#"
struct Point(i32, i32);
fn main() { let _ = $0Point(1, 2); }
"#,
        None,
    );
}

#[test]
fn tuple_enum_variant() {
    check_call_has_self_receiver(
        r#"
enum E { Wrap(i32) }
fn main() { let _ = $0E::Wrap(1); }
"#,
        None,
    );
}

#[test]
fn closure() {
    check_call_has_self_receiver(
        r#"
fn main() { let f = |x: i32| x; let _ = $0f(1); }
"#,
        None,
    );
}

#[test]
fn fn_returning_impl_trait_with_assoc_type() {
    check_call_type_full_name(
        r#"
trait Trait {
    type Assoc;
}

struct Foo;

impl Trait for Foo {
    type Assoc = i32;
}

fn make() -> impl Trait<Assoc = i32> {
    Foo
}

fn main() {
    $0make();
}
"#,
        "impl ra_test_fixture::Trait<Assoc = i32>",
    );
}

#[test]
fn fn_returning_impl_two_traits() {
    check_call_type_full_name(
        r#"
trait Sink {}
trait Make {}

struct Foo;

impl Sink for Foo {}
impl Make for Foo {}

fn make() -> impl Sink + Make {
    Foo
}

fn main() {
    $0make();
}
"#,
        "impl ra_test_fixture::Sink + ra_test_fixture::Make",
    );
}

#[test]
fn fn_returning_impl_generic_trait() {
    check_call_type_full_name(
        r#"
trait Extract<T> {}

struct Foo;

impl Extract<i32> for Foo {}

fn make() -> impl Extract<i32> {
    Foo
}

fn main() {
    $0make();
}
"#,
        "impl ra_test_fixture::Extract",
    );
}

#[test]
fn fn_returning_impl_trait_with_struct_assoc_type() {
    check_call_type_full_name(
        r#"
trait Trait {
    type Assoc;
}

struct Foo;
struct Bar;

impl Trait for Foo {
    type Assoc = Bar;
}

fn make() -> impl Trait<Assoc = Bar> {
    Foo
}

fn main() {
    $0make();
}
"#,
        "impl ra_test_fixture::Trait<Assoc = ra_test_fixture::Bar>",
    );
}

#[test]
fn fn_returning_impl_trait_with_method() {
    check_call_type_full_name(
        r#"
trait Trait {
    type Assoc;
    fn run(&self);
}

struct Foo;

impl Trait for Foo {
    type Assoc = i32;
    fn run(&self) {}
}

fn make() -> impl Trait<Assoc = i32> {
    Foo
}

fn main() {
    $0make();
}
"#,
        "impl ra_test_fixture::Trait<Assoc = i32>",
    );
}

#[test]
fn fn_returning_impl_trait_with_two_assoc_types() {
    check_call_type_full_name(
        r#"
trait Trait {
    type First;
    type Second;
}

struct Foo;

impl Trait for Foo {
    type First = i32;
    type Second = u32;
}

fn make() -> impl Trait<First = i32, Second = u32> {
    Foo
}

fn main() {
    $0make();
}
"#,
        "impl ra_test_fixture::Trait<First = i32, Second = u32>",
    );
}

#[test]
fn turbofish_free_fn_call_type_full_name() {
    check_call_type_full_name(
        r#"
fn identity<T>(value: T) -> T {
    value
}

fn main() {
    let identity_value = $0identity::<u32>(1);
}
"#,
        "u32",
    );
}

#[test]
fn turbofish_free_fn_call_method_full_name() {
    check_call_method_full_name(
        r#"
fn identity<T>(value: T) -> T {
    value
}

fn main() {
    let identity_value = $0identity::<u32>(1);
}
"#,
        "ra_test_fixture::identity<T>",
    );
}

#[test]
fn turbofish_inherent_fn_call_type_full_name() {
    check_call_type_full_name(
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
fn turbofish_inherent_fn_call_method_full_name() {
    check_call_method_full_name(
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
        "ra_test_fixture::Wrapper<T>::new",
    );
}

#[test]
fn generic_trait_impl_method_called_via_qualified_path() {
    check_call_method_full_name(
        r#"
trait Tr<T> {
    fn m(&self) -> T;
}
struct S<T>(T);
impl<T: Copy> Tr<T> for S<T> {
    fn m(&self) -> T { self.0 }
}
fn f(w: S<u32>) {
    let a = $0<S<u32> as Tr<u32>>::m(&w);
}
"#,
        "<ra_test_fixture::S<T> as ra_test_fixture::Tr<T>>::m",
    );
}

#[test]
fn generic_tuple_struct_in_impl_type_full_name() {
    check_call_type_full_name(
        r#"
struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        $0Wrapper(value)
    }
}

fn main() {}
"#,
        "ra_test_fixture::Wrapper<T>",
    );
}

#[test]
fn generic_tuple_struct_in_impl_method_full_name() {
    check_call_method_full_name(
        r#"
struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        $0Wrapper(value)
    }
}

fn main() {}
"#,
        "ra_test_fixture::Wrapper<T>",
    );
}

#[test]
fn trait_impls_per_const_arg_1() {
    check_call_method_full_name(
        r#"
trait Tr<const N: usize> {
    fn m(&self);
}

struct S;

impl Tr<3> for S {
    fn m(&self) {}
}

impl Tr<4> for S {
    fn m(&self) {}
}

fn f(s: S) {
    $0<S as Tr<3>>::m(&s);
    Tr::<4>::m(&s);
}

fn main() {}
"#,
        "<ra_test_fixture::S as ra_test_fixture::Tr<3>>::m",
    );
}

#[test]
fn trait_impls_per_const_arg_2() {
    check_call_method_full_name(
        r#"
trait Tr<const N: usize> {
    fn m(&self);
}

struct S;

impl Tr<3> for S {
    fn m(&self) {}
}

impl Tr<4> for S {
    fn m(&self) {}
}

fn f(s: S) {
    <S as Tr<3>>::m(&s);
    $0Tr::<4>::m(&s);
}

fn main() {}
"#,
        "<ra_test_fixture::S as ra_test_fixture::Tr<4>>::m",
    );
}

#[test]
fn free_fn_call_in_let_type_full_name() {
    check_call_type_full_name(
        r#"
fn foo() -> u32 {
    1
}

fn main() {
    let value = $0foo();
}
"#,
        "u32",
    );
}

#[test]
fn free_fn_call_in_let_method_full_name() {
    check_call_method_full_name(
        r#"
fn foo() -> u32 {
    1
}

fn main() {
    let value = $0foo();
}
"#,
        "ra_test_fixture::foo",
    );
}

#[test]
fn fn_pointer() {
    check_call_type_full_name(
        r#"
fn foo() -> u32 {
    1
}

fn main() {
    let ptr: fn() -> u32 = foo;
    let ptr_value = $0ptr();
}
"#,
        "u32",
    );
}

#[test]
fn closure_without_args() {
    check_call_type_full_name(
        r#"
fn main() {
    let closure = || 2u32;
    let closure_value = $0closure();
}
"#,
        "u32",
    );
}

#[test]
fn dyn_fn() {
    check_call_type_full_name(
        r#"
fn main() {
    let closure = || 2u32;
    let dyn_fn: &dyn Fn() -> u32 = &closure;
    let dyn_value = $0dyn_fn();
}
"#,
        "u32",
    );
}

#[test]
fn trait_impl_method_called_via_qualified_path() {
    check_call_method_full_name(
        r#"
trait Tr {
    fn m(&self) -> i32;
}
struct S;
impl Tr for S {
    fn m(&self) -> i32 { 0 }
}
fn f(s: S) {
    let a = $0<S as Tr>::m(&s);
}
"#,
        "<ra_test_fixture::S as ra_test_fixture::Tr>::m",
    );
}

#[test]
fn trait_impl_method_called_via_trait() {
    check_call_method_full_name(
        r#"
trait Tr {
    fn m(&self) -> i32;
}
struct S;
impl Tr for S {
    fn m(&self) -> i32 { 0 }
}
fn f(s: S) {
    let b = $0Tr::m(&s);
}
"#,
        "<ra_test_fixture::S as ra_test_fixture::Tr>::m",
    );
}

#[test]
fn trait_method_called_via_trait_on_dyn() {
    check_call_method_full_name(
        r#"
trait Tr {
    fn m(&self) -> i32;
}
struct S;
impl Tr for S {
    fn m(&self) -> i32 { 0 }
}
fn f(g: &dyn Tr) {
    let c = $0Tr::m(g);
}
"#,
        "ra_test_fixture::Tr::m",
    );
}

#[test]
fn fn_returning_const_raw_pointer() {
    check_call_type_full_name(
        r#"
fn returns_const() -> *const i32 {
    0 as *const i32
}

fn returns_mut() -> *mut i32 {
    0 as *mut i32
}

fn main() {
    let const_ptr = $0returns_const();
    let mut_ptr = returns_mut();
}
"#,
        "*const i32",
    );
}

#[test]
fn fn_returning_mut_raw_pointer() {
    check_call_type_full_name(
        r#"
fn returns_const() -> *const i32 {
    0 as *const i32
}

fn returns_mut() -> *mut i32 {
    0 as *mut i32
}

fn main() {
    let const_ptr = returns_const();
    let mut_ptr = $0returns_mut();
}
"#,
        "*mut i32",
    );
}

#[test]
fn self_assoc_fn_in_trait_default_method() {
    check_call_type_full_name(
        r#"
trait Tr {
    fn m() -> Self;
    fn d() -> Self {
        $0Self::m()
    }
}

fn main() {}
"#,
        "ra_test_fixture::Tr",
    );
}

#[test]
fn local_struct_in_sibling_block_1() {
    check_call_method_full_name(
        r#"
fn f() {
    if true {
        struct S { x: i32 }
        impl S { fn new() -> S { S { x: 1 } } }
        let a = $0S::new();
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
        "ra_test_fixture::f::S#1::new",
    );
}

#[test]
fn local_struct_in_sibling_block_2() {
    check_call_method_full_name(
        r#"
fn f() {
    if true {
        struct S { x: i32 }
        impl S { fn new() -> S { S { x: 1 } } }
        let a = S::new();
        let _ = a.x;
    }
    if false {
        struct S { x: u8 }
        impl S { fn new() -> S { S { x: 2 } } }
        let b = $0S::new();
        let _ = b.x;
    }
}

fn main() { f(); }
"#,
        "ra_test_fixture::f::S#2::new",
    );
}

#[test]
fn local_struct_in_sibling_anonymous_const_1() {
    check_call_method_full_name(
        r#"
const _: () = {
    struct S { x: i32 }
    impl S { fn new() -> S { S { x: 1 } } }
    fn make_a() -> S { $0S::new() }
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
    check_call_method_full_name(
        r#"
const _: () = {
    struct S { x: i32 }
    impl S { fn new() -> S { S { x: 1 } } }
    fn make_a() -> S { S::new() }
};
const _: () = {
    struct S { x: u8 }
    impl S { fn new() -> S { S { x: 2 } } }
    fn make_b() -> S { $0S::new() }
};

fn main() {}
"#,
        "ra_test_fixture::S#2::new",
    );
}
