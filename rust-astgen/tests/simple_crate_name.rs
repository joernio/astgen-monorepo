use crate::common::{TestResult, ast_json};

mod common;

#[test]
fn emits_crate_name() -> TestResult<()> {
    let json = ast_json("my_crate_name", "fn foo() {}")?;

    // crateName being in the wrapped AST
    assert_eq!(json["crateName"].as_str().unwrap(), "my_crate_name");

    Ok(())
}
