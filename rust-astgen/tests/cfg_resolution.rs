mod common;

use crate::common::{
    TestResult, call_expr, fn_decl, no_sysroot_resolve_cfg_ast_json, nodes_by_kind,
};

#[test]
fn cfg_test_module_is_dropped() -> TestResult<()> {
    let json = no_sysroot_resolve_cfg_ast_json(
        "rust2cpg",
        &[(
            "src/lib.rs",
            r#"
pub fn kept() {}

#[cfg(test)]
mod tests {
    fn helper() -> i32 { 0 }
}
"#,
        )],
        "src/lib.rs",
    )?;

    assert_eq!(nodes_by_kind(&json, "MODULE").len(), 0);
    assert_eq!(nodes_by_kind(&json, "FN").len(), 1);
    assert_eq!(
        fn_decl(&json, "pub fn kept() {}").method_full_name(),
        "rust2cpg::kept"
    );

    Ok(())
}

#[test]
fn inactive_feature_item_is_dropped() -> TestResult<()> {
    let json = no_sysroot_resolve_cfg_ast_json(
        "rust2cpg",
        &[(
            "src/lib.rs",
            r#"
#[cfg(not(test))]
pub fn present() {}

#[cfg(feature = "nonexistent")]
pub fn absent() {}
"#,
        )],
        "src/lib.rs",
    )?;

    assert_eq!(nodes_by_kind(&json, "FN").len(), 1);
    assert_eq!(
        fn_decl(&json, "#[cfg(not(test))]\npub fn present() {}").method_full_name(),
        "rust2cpg::present"
    );

    Ok(())
}

#[test]
fn always_false_cfg_is_dropped() -> TestResult<()> {
    let json = no_sysroot_resolve_cfg_ast_json(
        "rust2cpg",
        &[(
            "src/lib.rs",
            r#"
#[cfg(any())]
fn never_active() {}

fn plain() {}
"#,
        )],
        "src/lib.rs",
    )?;

    assert_eq!(nodes_by_kind(&json, "FN").len(), 1);
    assert_eq!(
        fn_decl(&json, "fn plain() {}").method_full_name(),
        "rust2cpg::plain"
    );

    Ok(())
}

#[test]
fn inactive_call_stmt_is_dropped_with_its_expr_stmt() -> TestResult<()> {
    let json = no_sysroot_resolve_cfg_ast_json(
        "rust2cpg",
        &[(
            "src/lib.rs",
            r#"
fn f() {}
fn g() {}

fn h() {
    #[cfg(any())]
    f();
    g();
}
"#,
        )],
        "src/lib.rs",
    )?;

    assert_eq!(nodes_by_kind(&json, "EXPR_STMT").len(), 1);
    assert!(call_expr(&json, "g()").exists());
    assert!(!call_expr(&json, "f()").exists());

    Ok(())
}

#[test]
fn active_cfg_call_stmt_is_kept() -> TestResult<()> {
    let json = no_sysroot_resolve_cfg_ast_json(
        "rust2cpg",
        &[(
            "src/lib.rs",
            r#"
fn f() {}

fn h() {
    #[cfg(all())]
    f();
}
"#,
        )],
        "src/lib.rs",
    )?;

    assert!(call_expr(&json, "#[cfg(all())]\n    f()").exists());

    Ok(())
}

#[test]
fn inactive_match_arm_is_dropped_from_kept_match() -> TestResult<()> {
    let json = no_sysroot_resolve_cfg_ast_json(
        "rust2cpg",
        &[(
            "src/lib.rs",
            r#"
fn f() {}
fn g() {}

fn h(x: i32) {
    match x {
        #[cfg(any())]
        0 => f(),
        _ => g(),
    }
}
"#,
        )],
        "src/lib.rs",
    )?;

    assert_eq!(nodes_by_kind(&json, "MATCH_EXPR").len(), 1);
    assert_eq!(nodes_by_kind(&json, "MATCH_ARM").len(), 1);
    assert_eq!(nodes_by_kind(&json, "CALL_EXPR").len(), 1);
    assert!(call_expr(&json, "g()").exists());

    Ok(())
}

#[test]
fn item_is_dropped_when_any_cfg_is_inactive() -> TestResult<()> {
    let json = no_sysroot_resolve_cfg_ast_json(
        "rust2cpg",
        &[(
            "src/lib.rs",
            r#"
#[cfg(all())]
#[cfg(any())]
fn dropped() {}

fn kept() {}
"#,
        )],
        "src/lib.rs",
    )?;

    assert_eq!(nodes_by_kind(&json, "FN").len(), 1);
    assert_eq!(
        fn_decl(&json, "fn kept() {}").method_full_name(),
        "rust2cpg::kept"
    );

    Ok(())
}
