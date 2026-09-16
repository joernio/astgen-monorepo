use super::{check_method_call_method_full_name, check_method_call_type_full_name};

#[test]
fn inherent_method() {
    check_method_call_method_full_name(
        r#"
struct Foo;

impl Foo {
    fn foo(&self) -> u32 { 1 }
}

fn main() {
    let value = Foo.$0foo();
}
"#,
        "ra_test_fixture::Foo::foo",
    );
}

#[test]
fn trait_impl_method() {
    check_method_call_method_full_name(
        r#"
struct Foo;

trait Greet {
    fn hello(&self) -> bool;
}

impl Greet for Foo {
    fn hello(&self) -> bool { true }
}

fn main() {
    let foo = Foo;
    let value = foo.$0hello();
}
"#,
        "<ra_test_fixture::Foo as ra_test_fixture::Greet>::hello",
    );
}

#[test]
fn method_of_generic_struct_type_full_name() {
    check_method_call_type_full_name(
        r#"
struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        Wrapper(value)
    }

    fn value(&self) -> T {
        self.0
    }

    fn value_mut(&mut self) -> T {
        self.0
    }

    fn passthrough<U>(&self, value: U) -> U {
        value
    }
}

fn main() {
    let wrapped = Wrapper::<u32>::new(1);
    let direct = $0wrapped.value();
    let by_ref = (&wrapped).value();
    let mut mutable = Wrapper::<u32>::new(2);
    let by_mut = (&mut mutable).value_mut();
    let passthrough = wrapped.passthrough::<bool>(true);
}
"#,
        "u32",
    );
}

#[test]
fn method_of_generic_struct_method_full_name() {
    check_method_call_method_full_name(
        r#"
struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        Wrapper(value)
    }

    fn value(&self) -> T {
        self.0
    }

    fn value_mut(&mut self) -> T {
        self.0
    }

    fn passthrough<U>(&self, value: U) -> U {
        value
    }
}

fn main() {
    let wrapped = Wrapper::<u32>::new(1);
    let direct = $0wrapped.value();
    let by_ref = (&wrapped).value();
    let mut mutable = Wrapper::<u32>::new(2);
    let by_mut = (&mut mutable).value_mut();
    let passthrough = wrapped.passthrough::<bool>(true);
}
"#,
        "ra_test_fixture::Wrapper<T>::value",
    );
}

#[test]
fn method_of_generic_struct_on_ref_type_full_name() {
    check_method_call_type_full_name(
        r#"
struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        Wrapper(value)
    }

    fn value(&self) -> T {
        self.0
    }

    fn value_mut(&mut self) -> T {
        self.0
    }

    fn passthrough<U>(&self, value: U) -> U {
        value
    }
}

fn main() {
    let wrapped = Wrapper::<u32>::new(1);
    let direct = wrapped.value();
    let by_ref = $0(&wrapped).value();
    let mut mutable = Wrapper::<u32>::new(2);
    let by_mut = (&mut mutable).value_mut();
    let passthrough = wrapped.passthrough::<bool>(true);
}
"#,
        "u32",
    );
}

#[test]
fn method_of_generic_struct_on_ref_method_full_name() {
    check_method_call_method_full_name(
        r#"
struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        Wrapper(value)
    }

    fn value(&self) -> T {
        self.0
    }

    fn value_mut(&mut self) -> T {
        self.0
    }

    fn passthrough<U>(&self, value: U) -> U {
        value
    }
}

fn main() {
    let wrapped = Wrapper::<u32>::new(1);
    let direct = wrapped.value();
    let by_ref = $0(&wrapped).value();
    let mut mutable = Wrapper::<u32>::new(2);
    let by_mut = (&mut mutable).value_mut();
    let passthrough = wrapped.passthrough::<bool>(true);
}
"#,
        "ra_test_fixture::Wrapper<T>::value",
    );
}

#[test]
fn mut_method_of_generic_struct_on_mut_ref_type_full_name() {
    check_method_call_type_full_name(
        r#"
struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        Wrapper(value)
    }

    fn value(&self) -> T {
        self.0
    }

    fn value_mut(&mut self) -> T {
        self.0
    }

    fn passthrough<U>(&self, value: U) -> U {
        value
    }
}

fn main() {
    let wrapped = Wrapper::<u32>::new(1);
    let direct = wrapped.value();
    let by_ref = (&wrapped).value();
    let mut mutable = Wrapper::<u32>::new(2);
    let by_mut = $0(&mut mutable).value_mut();
    let passthrough = wrapped.passthrough::<bool>(true);
}
"#,
        "u32",
    );
}

#[test]
fn mut_method_of_generic_struct_on_mut_ref_method_full_name() {
    check_method_call_method_full_name(
        r#"
struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        Wrapper(value)
    }

    fn value(&self) -> T {
        self.0
    }

    fn value_mut(&mut self) -> T {
        self.0
    }

    fn passthrough<U>(&self, value: U) -> U {
        value
    }
}

fn main() {
    let wrapped = Wrapper::<u32>::new(1);
    let direct = wrapped.value();
    let by_ref = (&wrapped).value();
    let mut mutable = Wrapper::<u32>::new(2);
    let by_mut = $0(&mut mutable).value_mut();
    let passthrough = wrapped.passthrough::<bool>(true);
}
"#,
        "ra_test_fixture::Wrapper<T>::value_mut",
    );
}

#[test]
fn generic_method_of_generic_struct_type_full_name() {
    check_method_call_type_full_name(
        r#"
struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        Wrapper(value)
    }

    fn value(&self) -> T {
        self.0
    }

    fn value_mut(&mut self) -> T {
        self.0
    }

    fn passthrough<U>(&self, value: U) -> U {
        value
    }
}

fn main() {
    let wrapped = Wrapper::<u32>::new(1);
    let direct = wrapped.value();
    let by_ref = (&wrapped).value();
    let mut mutable = Wrapper::<u32>::new(2);
    let by_mut = (&mut mutable).value_mut();
    let passthrough = $0wrapped.passthrough::<bool>(true);
}
"#,
        "bool",
    );
}

#[test]
fn generic_method_of_generic_struct_method_full_name() {
    check_method_call_method_full_name(
        r#"
struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        Wrapper(value)
    }

    fn value(&self) -> T {
        self.0
    }

    fn value_mut(&mut self) -> T {
        self.0
    }

    fn passthrough<U>(&self, value: U) -> U {
        value
    }
}

fn main() {
    let wrapped = Wrapper::<u32>::new(1);
    let direct = wrapped.value();
    let by_ref = (&wrapped).value();
    let mut mutable = Wrapper::<u32>::new(2);
    let by_mut = (&mut mutable).value_mut();
    let passthrough = $0wrapped.passthrough::<bool>(true);
}
"#,
        "ra_test_fixture::Wrapper<T>::passthrough<U>",
    );
}

#[test]
fn generic_trait_impl_method_type_full_name() {
    check_method_call_type_full_name(
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

impl<T: Copy> Extract<T> for Wrapper<T> {
    fn extract(&self) -> T {
        self.0
    }
}

fn main() {
    let wrapped = Wrapper::<u32>::new(1);
    let extracted = $0wrapped.extract();
}
"#,
        "u32",
    );
}

#[test]
fn generic_trait_impl_method_method_full_name() {
    check_method_call_method_full_name(
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

impl<T: Copy> Extract<T> for Wrapper<T> {
    fn extract(&self) -> T {
        self.0
    }
}

fn main() {
    let wrapped = Wrapper::<u32>::new(1);
    let extracted = $0wrapped.extract();
}
"#,
        "<ra_test_fixture::Wrapper<T> as ra_test_fixture::Extract<T>>::extract",
    );
}

#[test]
fn trait_bound_method_type_full_name() {
    check_method_call_type_full_name(
        r#"
trait Sink<T> {
    fn sink(&self) -> T;
}

fn call_bound<S: Sink<u32>>(s: S) {
    let bound_value = $0s.sink();
}

fn main() {}
"#,
        "u32",
    );
}

#[test]
fn trait_bound_method_method_full_name() {
    check_method_call_method_full_name(
        r#"
trait Sink<T> {
    fn sink(&self) -> T;
}

fn call_bound<S: Sink<u32>>(s: S) {
    let bound_value = $0s.sink();
}

fn main() {}
"#,
        "ra_test_fixture::Sink<T>::sink",
    );
}

#[test]
fn trait_impl_with_lifetime_type_and_const_args() {
    check_method_call_method_full_name(
        r#"
trait Tr<'a, T, const N: usize> {
    fn m(&self);
}

struct S;

impl<'a> Tr<'a, u8, 3> for S {
    fn m(&self) {}
}

fn f(s: S) {
    $0s.m();
}

fn main() {}
"#,
        "<ra_test_fixture::S as ra_test_fixture::Tr<'a, u8, 3>>::m",
    );
}

#[test]
fn inherent_method_of_imported_struct_type_full_name() {
    check_method_call_type_full_name(
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
    let method_value = $0receiver.value();
}
"#,
        "bool",
    );
}

#[test]
fn inherent_method_of_imported_struct_method_full_name() {
    check_method_call_method_full_name(
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
    let method_value = $0receiver.value();
}
"#,
        "ra_test_fixture::imported::Type::value",
    );
}

#[test]
fn trait_method_of_imported_struct_type_full_name() {
    check_method_call_type_full_name(
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
    let trait_value = $0trait_receiver.trait_value();
}
"#,
        "bool",
    );
}

#[test]
fn trait_method_of_imported_struct_method_full_name() {
    check_method_call_method_full_name(
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
    let trait_value = $0trait_receiver.trait_value();
}
"#,
        "<ra_test_fixture::imported::Type as ra_test_fixture::imported::Trait>::trait_value",
    );
}

#[test]
fn inherent_method_on_ref_type_full_name() {
    check_method_call_type_full_name(
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
    let ref_value = $0(&ref_receiver).value();
    let mut mut_receiver = Type;
    let mut_value = (&mut mut_receiver).by_mut();
}
"#,
        "bool",
    );
}

#[test]
fn inherent_method_on_ref_method_full_name() {
    check_method_call_method_full_name(
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
    let ref_value = $0(&ref_receiver).value();
    let mut mut_receiver = Type;
    let mut_value = (&mut mut_receiver).by_mut();
}
"#,
        "ra_test_fixture::imported::Type::value",
    );
}

#[test]
fn inherent_mut_method_on_mut_ref_type_full_name() {
    check_method_call_type_full_name(
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
    let mut_value = $0(&mut mut_receiver).by_mut();
}
"#,
        "bool",
    );
}

#[test]
fn inherent_mut_method_on_mut_ref_method_full_name() {
    check_method_call_method_full_name(
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
    let mut_value = $0(&mut mut_receiver).by_mut();
}
"#,
        "ra_test_fixture::imported::Type::by_mut",
    );
}

#[test]
fn trait_impl_for_assoc_type() {
    check_method_call_method_full_name(
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
    fn m(&self) {}
}
fn f(t: T) {
    $0t.m();
}
"#,
        "<ra_test_fixture::T as ra_test_fixture::Sink>::m",
    );
}
