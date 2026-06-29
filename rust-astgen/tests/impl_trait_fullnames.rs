mod common;

use crate::common::{TestResult, call_expr, no_sysroot_ast_json};

#[test]
fn emits_names_for_impl_trait_with_associated_type() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
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
    make();
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        call_expr(&json, "make()").type_full_name(),
        "impl rust2cpg::Trait<Assoc = i32>"
    );

    Ok(())
}

#[test]
fn emits_names_for_multiple_impl_trait_bounds() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
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
    make();
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        call_expr(&json, "make()").type_full_name(),
        "impl rust2cpg::Sink + rust2cpg::Make"
    );

    Ok(())
}

#[test]
fn impl_trait_type_arguments_are_dropped() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Extract<T> {}

struct Foo;

impl Extract<i32> for Foo {}

fn make() -> impl Extract<i32> {
    Foo
}

fn main() {
    make();
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        call_expr(&json, "make()").type_full_name(),
        "impl rust2cpg::Extract"
    );

    Ok(())
}

#[test]
fn emits_qualified_name_for_associated_type_value() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
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
    make();
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        call_expr(&json, "make()").type_full_name(),
        "impl rust2cpg::Trait<Assoc = rust2cpg::Bar>"
    );

    Ok(())
}

#[test]
fn ignores_trait_methods_when_formatting_impl_trait() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
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
    make();
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        call_expr(&json, "make()").type_full_name(),
        "impl rust2cpg::Trait<Assoc = i32>"
    );

    Ok(())
}

#[test]
fn emits_names_for_multiple_associated_types() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
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
    make();
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        call_expr(&json, "make()").type_full_name(),
        "impl rust2cpg::Trait<First = i32, Second = u32>"
    );

    Ok(())
}
