mod common;

use crate::common::{TestResult, call_expr, sysroot_ast_json};

#[test]
fn emits_names_for_async_fn_return_type() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        r#"
async fn f() -> i32 {
    1
}

fn main() {
    f();
}
"#,
    )?;

    assert_eq!(
        call_expr(&json, "f()").type_full_name(),
        "impl core::future::future::Future<Output = i32> + core::marker::Sized"
    );

    Ok(())
}
