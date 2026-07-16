mod common;

use crate::common::{TestResult, no_sysroot_ast_json, trait_decl};

#[test]
fn emits_supertrait_for_trait() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr {}

trait Sub: Tr {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        trait_decl(&json, "trait Sub: Tr {}").supertraits(),
        vec!["rust2cpg::Tr"]
    );

    Ok(())
}

#[test]
fn emits_generic_args_of_supertrait() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr<T> {}

trait Sub: Tr<u8> {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        trait_decl(&json, "trait Sub: Tr<u8> {}").supertraits(),
        vec!["rust2cpg::Tr<u8>"]
    );

    Ok(())
}

#[test]
fn keeps_supertraits_in_declaration_order() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr1 {}
trait Tr2 {}

trait Sub: Tr2 + Tr1 {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        trait_decl(&json, "trait Sub: Tr2 + Tr1 {}").supertraits(),
        vec!["rust2cpg::Tr1", "rust2cpg::Tr2"]
    );

    Ok(())
}

#[test]
fn skips_lifetime_bounds() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr {}

trait Sub: Tr + 'static {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        trait_decl(&json, "trait Sub: Tr + 'static {}").supertraits(),
        vec!["rust2cpg::Tr"]
    );

    Ok(())
}

#[test]
fn omits_supertraits_when_trait_has_none() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
trait Tr {}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        trait_decl(&json, "trait Tr {}").supertraits(),
        Vec::<String>::new()
    );

    Ok(())
}
