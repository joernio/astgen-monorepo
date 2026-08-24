mod common;

use crate::common::{
    TestResult, call_expr, enum_decl, fn_decl, ident_pat, method_call_expr, name_ref,
    no_sysroot_ast_json, sysroot_ast_json,
};

#[test]
fn emits_names_for_generic_free_function_call() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn identity<T>(value: T) -> T {
    value
}

fn main() {
    let identity_value = identity::<u32>(1);
}
"#,
        )],
        "src/main.rs",
    )?;

    let identity_call = call_expr(&json, "identity::<u32>(1)");
    assert_eq!(identity_call.type_full_name(), "u32");
    assert_eq!(identity_call.method_full_name(), "rust2cpg::identity<T>");

    Ok(())
}

#[test]
fn emits_names_for_generic_associated_function_call() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        Wrapper(value)
    }
}

fn main() {
    let wrapped = Wrapper::<u32>::new(1);
}
"#,
        )],
        "src/main.rs",
    )?;

    let new_call = call_expr(&json, "Wrapper::<u32>::new(1)");
    assert_eq!(new_call.type_full_name(), "rust2cpg::Wrapper<u32>");
    assert_eq!(new_call.method_full_name(), "rust2cpg::Wrapper<T>::new");
    assert_eq!(
        name_ref(&json, "Wrapper")
            .on_line("let wrapped = Wrapper::<u32>::new(1);")
            .type_full_name(),
        "rust2cpg::Wrapper<u32>"
    );
    assert_eq!(
        name_ref(&json, "new")
            .on_line("let wrapped = Wrapper::<u32>::new(1);")
            .type_full_name(),
        "fn(u32) -> rust2cpg::Wrapper<u32>"
    );
    assert_eq!(
        ident_pat(&json, "wrapped").type_full_name(),
        "rust2cpg::Wrapper<u32>"
    );

    Ok(())
}

#[test]
fn emits_names_for_methods_on_generic_type() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
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
    let passthrough = wrapped.passthrough::<bool>(true);
}
"#,
        )],
        "src/main.rs",
    )?;

    let direct_call = method_call_expr(&json, "wrapped.value()");
    assert_eq!(direct_call.type_full_name(), "u32");
    assert_eq!(
        direct_call.method_full_name(),
        "rust2cpg::Wrapper<T>::value"
    );

    let by_ref_call = method_call_expr(&json, "(&wrapped).value()");
    assert_eq!(by_ref_call.type_full_name(), "u32");
    assert_eq!(
        by_ref_call.method_full_name(),
        "rust2cpg::Wrapper<T>::value"
    );

    let by_mut_call = method_call_expr(&json, "(&mut mutable).value_mut()");
    assert_eq!(by_mut_call.type_full_name(), "u32");
    assert_eq!(
        by_mut_call.method_full_name(),
        "rust2cpg::Wrapper<T>::value_mut"
    );

    let passthrough_call = method_call_expr(&json, "wrapped.passthrough::<bool>(true)");
    assert_eq!(passthrough_call.type_full_name(), "bool");
    assert_eq!(
        passthrough_call.method_full_name(),
        "rust2cpg::Wrapper<T>::passthrough<U>"
    );

    Ok(())
}

#[test]
fn emits_names_for_trait_impl_method_call() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
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
    let extracted = wrapped.extract();
}
"#,
        )],
        "src/main.rs",
    )?;

    let extract_call = method_call_expr(&json, "wrapped.extract()");
    assert_eq!(extract_call.type_full_name(), "u32");
    assert_eq!(
        extract_call.method_full_name(),
        "<rust2cpg::Wrapper<T> as rust2cpg::Extract<T>>::extract"
    );

    Ok(())
}

#[test]
fn emits_concrete_impl_method_for_generic_trait_path_call() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr<T> {
    fn m(&self) -> T;
}
struct S<T>(T);
impl<T: Copy> Tr<T> for S<T> {
    fn m(&self) -> T { self.0 }
}
fn f(w: S<u32>) {
    let a = <S<u32> as Tr<u32>>::m(&w);
}
"#,
        )],
        "src/main.rs",
    )?;

    let m_call = call_expr(&json, "<S<u32> as Tr<u32>>::m(&w)");
    assert_eq!(
        m_call.method_full_name(),
        "<rust2cpg::S<T> as rust2cpg::Tr<T>>::m"
    );
    assert_eq!(
        fn_decl(&json, "fn m(&self) -> T { self.0 }").method_full_name(),
        "<rust2cpg::S<T> as rust2cpg::Tr<T>>::m"
    );

    Ok(())
}

#[test]
fn emits_names_for_bound_trait_method_call() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Sink<T> {
    fn sink(&self) -> T;
}

fn call_bound<S: Sink<u32>>(s: S) {
    let bound_value = s.sink();
}

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

    let bound_sink_call =
        method_call_expr(&json, "s.sink()").on_line("let bound_value = s.sink();");
    assert_eq!(bound_sink_call.type_full_name(), "u32");
    assert_eq!(
        bound_sink_call.method_full_name(),
        "rust2cpg::Sink<T>::sink"
    );

    Ok(())
}

#[test]
fn emits_names_for_tuple_struct_constructor() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        Wrapper(value)
    }
}

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

    let tuple_constructor = call_expr(&json, "Wrapper(value)");
    assert_eq!(tuple_constructor.type_full_name(), "rust2cpg::Wrapper<T>");
    assert_eq!(tuple_constructor.method_full_name(), "rust2cpg::Wrapper<T>");

    assert_eq!(
        name_ref(&json, "Wrapper")
            .on_line("Wrapper(value)")
            .type_full_name(),
        "fn(T) -> rust2cpg::Wrapper<T>"
    );

    Ok(())
}

#[test]
fn emits_names_for_generic_adt_in_source_positions() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
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

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        ident_pat(&json, "value")
            .on_line("fn new(value: T) -> Wrapper<T> {")
            .type_full_name(),
        "T"
    );
    assert_eq!(
        name_ref(&json, "Wrapper")
            .on_line("impl<T: Copy> Wrapper<T> {")
            .type_full_name(),
        "rust2cpg::Wrapper<T>"
    );
    assert_eq!(
        name_ref(&json, "Wrapper")
            .on_line("fn new(value: T) -> Wrapper<T> {")
            .type_full_name(),
        "rust2cpg::Wrapper<T>"
    );
    assert_eq!(
        name_ref(&json, "Wrapper")
            .on_line("impl<T: Copy> Extract<T> for Wrapper<T> {")
            .type_full_name(),
        "rust2cpg::Wrapper<T>"
    );

    Ok(())
}

#[test]
fn emits_names_for_generic_adt_after_move() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct Wrapper<T>(T);

fn main() {
    let plain = Wrapper(3u32);
    let copied = plain;
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        ident_pat(&json, "plain").type_full_name(),
        "rust2cpg::Wrapper<u32>"
    );
    assert_eq!(
        ident_pat(&json, "copied").type_full_name(),
        "rust2cpg::Wrapper<u32>"
    );
    assert_eq!(
        name_ref(&json, "plain").type_full_name(),
        "rust2cpg::Wrapper<u32>"
    );

    Ok(())
}

#[test]
fn emits_trait_full_name_for_self_in_generic_trait_decl() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr<T> {
    fn m() -> Self;
}

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(name_ref(&json, "Self").type_full_name(), "rust2cpg::Tr<T>");

    Ok(())
}

#[test]
fn emits_names_for_vec_and_dyn_trait() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
use std::vec::Vec;

trait Sink<T> {
    fn sink(&self) -> T;
}

fn call_dyn(s: &dyn Sink<u32>) {
    let dyn_value = s.sink();
}

fn main() {
    let numbers = Vec::<u32>::new();
    let numbers_copy = numbers;
}
"#,
        )],
        "src/main.rs",
    )?;

    let vec_new_call = call_expr(&json, "Vec::<u32>::new()");
    assert_eq!(
        vec_new_call.type_full_name(),
        "alloc::vec::Vec<u32, alloc::alloc::Global>"
    );
    assert_eq!(
        vec_new_call.method_full_name(),
        "alloc::vec::Vec<T, alloc::alloc::Global>::new"
    );
    assert_eq!(
        name_ref(&json, "Vec")
            .on_line("let numbers = Vec::<u32>::new();")
            .type_full_name(),
        "alloc::vec::Vec<u32>"
    );
    assert_eq!(
        name_ref(&json, "new")
            .on_line("let numbers = Vec::<u32>::new();")
            .type_full_name(),
        "fn() -> alloc::vec::Vec<u32, alloc::alloc::Global>"
    );
    assert_eq!(
        ident_pat(&json, "numbers").type_full_name(),
        "alloc::vec::Vec<u32, alloc::alloc::Global>"
    );
    assert_eq!(
        name_ref(&json, "numbers").type_full_name(),
        "alloc::vec::Vec<u32, alloc::alloc::Global>"
    );

    let dyn_sink_call = method_call_expr(&json, "s.sink()").on_line("let dyn_value = s.sink();");
    assert_eq!(dyn_sink_call.type_full_name(), "u32");
    assert_eq!(dyn_sink_call.method_full_name(), "rust2cpg::Sink<T>::sink");

    Ok(())
}

#[test]
fn emits_type_full_name_for_lifetime_parameterized_impl_self_type() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr {
    fn m(&self);
}

struct Mix<'a, T> {
    value: &'a T,
}

impl<'a, T> Tr for Mix<'a, T> {
    fn m(&self) {}
}

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        fn_decl(&json, "fn m(&self) {}").method_full_name(),
        "<rust2cpg::Mix<'a, T> as rust2cpg::Tr>::m"
    );
    assert_eq!(
        name_ref(&json, "Mix")
            .on_line("impl<'a, T> Tr for Mix<'a, T> {")
            .type_full_name(),
        "rust2cpg::Mix<'a, T>"
    );

    Ok(())
}

#[test]
fn emits_type_full_name_for_const_param_impl_self_type() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
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

fn mixed<const N: usize>(value: &Pair<u32, N>) {}

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        name_ref(&json, "Foo")
            .on_line("fn concrete(value: &Foo<3>) {}")
            .type_full_name(),
        "rust2cpg::Foo<3>"
    );
    assert_eq!(
        name_ref(&json, "Foo")
            .on_line("fn expression(value: &Foo<{ 2 + 1 }>) {}")
            .type_full_name(),
        "rust2cpg::Foo<{ 2 + 1 }>"
    );
    assert_eq!(
        fn_decl(&json, "fn m(&self) {}").method_full_name(),
        "<rust2cpg::Foo<N> as rust2cpg::Tr>::m"
    );
    assert_eq!(
        name_ref(&json, "Foo")
            .on_line("impl<const N: usize> Tr for Foo<N> {")
            .type_full_name(),
        "rust2cpg::Foo<N>"
    );
    assert_eq!(
        name_ref(&json, "Pair")
            .on_line("fn mixed<const N: usize>(value: &Pair<u32, N>) {}")
            .type_full_name(),
        "rust2cpg::Pair<u32, N>"
    );

    Ok(())
}

#[test]
fn emits_type_full_name_for_path_with_associated_type_binding() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr {
    type Assoc;
}

struct Foo;

impl Tr for Foo {
    type Assoc = u32;
}

fn bare(value: &dyn Tr) {}

fn bound(value: &dyn Tr<Assoc = u32>) {}

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        name_ref(&json, "Tr")
            .on_line("fn bare(value: &dyn Tr) {}")
            .type_full_name(),
        "rust2cpg::Tr"
    );
    assert_eq!(
        name_ref(&json, "Tr")
            .on_line("fn bound(value: &dyn Tr<Assoc = u32>) {}")
            .type_full_name(),
        "rust2cpg::Tr<Assoc = u32>"
    );

    Ok(())
}

#[test]
fn emits_type_full_name_for_lifetime_parameterized_path_outside_impl() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct Foo<'a> {
    value: &'a str,
}

fn named<'a>(value: &Foo<'a>) {}

fn elided(value: &Foo<'_>) {}

fn borrowed_static(value: &'static Foo<'static>) {}

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        name_ref(&json, "Foo")
            .on_line("fn named<'a>(value: &Foo<'a>) {}")
            .type_full_name(),
        "rust2cpg::Foo<'a>"
    );
    assert_eq!(
        name_ref(&json, "Foo")
            .on_line("fn elided(value: &Foo<'_>) {}")
            .type_full_name(),
        "rust2cpg::Foo<'_>"
    );
    assert_eq!(
        name_ref(&json, "Foo")
            .on_line("fn borrowed_static(value: &'static Foo<'static>) {}")
            .type_full_name(),
        "rust2cpg::Foo<'static>"
    );

    Ok(())
}

#[test]
fn emits_distinct_type_full_names_for_distinct_lifetime_parameterized_impls() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
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

impl<'a> Tr for Beta<'a> {
    fn m(&self) {}
}

fn main() {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        name_ref(&json, "Alpha")
            .on_line("impl<'a> Tr for Alpha<'a> {")
            .type_full_name(),
        "rust2cpg::Alpha<'a>"
    );
    assert_eq!(
        name_ref(&json, "Beta")
            .on_line("impl<'a> Tr for Beta<'a> {")
            .type_full_name(),
        "rust2cpg::Beta<'a>"
    );

    Ok(())
}

#[test]
fn emits_type_full_name_for_generic_enum_declaration() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
enum E<T> { A(T) }

fn main() {
    let _ = E::A(1);
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        enum_decl(&json, "enum E<T> { A(T) }").type_full_name(),
        "rust2cpg::E<T>"
    );

    Ok(())
}
