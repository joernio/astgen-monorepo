mod common;

use crate::common::{
    TestResult, call_expr, ident_pat, method_call_expr, name_ref, no_sysroot_ast_json,
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
