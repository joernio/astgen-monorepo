mod common;

use crate::common::{TestResult, no_sysroot_ast_json, struct_decl, sysroot_ast_json};

#[test]
fn emits_implemented_trait_for_struct() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr {}

struct S;

impl Tr for S {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct S;").implemented_traits(),
        vec!["rust2cpg::Tr"]
    );

    Ok(())
}

#[test]
fn emits_generic_args_of_implemented_trait() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr<T> {}

struct S;

impl Tr<u8> for S {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct S;").implemented_traits(),
        vec!["rust2cpg::Tr<u8>"]
    );

    Ok(())
}

#[test]
fn emits_one_entry_per_impl_of_same_trait() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr<T> {}

struct S;

impl Tr<u8> for S {}
impl Tr<u16> for S {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct S;").implemented_traits(),
        vec!["rust2cpg::Tr<u16>", "rust2cpg::Tr<u8>"]
    );

    Ok(())
}

#[test]
fn keeps_impl_scoped_type_params_by_name() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr<T> {}

struct S;

impl<T> Tr<T> for S {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct S;").implemented_traits(),
        vec!["rust2cpg::Tr<T>"]
    );

    Ok(())
}

#[test]
fn ignores_inherent_impls() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct S;

impl S {
    fn m(&self) {}
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct S;").implemented_traits(),
        Vec::<String>::new()
    );

    Ok(())
}

#[test]
fn excludes_negative_impls() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr {}

struct S;

impl !Tr for S {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct S;").implemented_traits(),
        Vec::<String>::new()
    );

    Ok(())
}

#[test]
fn finds_impls_in_other_modules() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[
            (
                "src/main.rs",
                r#"
mod m;

trait Tr {}

struct S;
"#,
            ),
            (
                "src/m.rs",
                r#"
use crate::{S, Tr};

impl Tr for S {}
"#,
            ),
        ],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct S;").implemented_traits(),
        vec!["rust2cpg::Tr"]
    );

    Ok(())
}

#[test]
fn emits_derived_trait_impls() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        r#"
#[derive(Clone)]
struct S;

fn main() {}
"#,
    )?;

    assert_eq!(
        struct_decl(&json, "#[derive(Clone)]\nstruct S;").implemented_traits(),
        vec!["core::clone::Clone"]
    );

    Ok(())
}
