mod common;

use crate::common::{TestResult, struct_decl, sysroot_ast_json};

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
