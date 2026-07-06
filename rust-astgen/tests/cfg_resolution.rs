mod common;

use crate::common::{TestResult, fn_decl, no_sysroot_resolve_cfg_ast_json, nodes_by_kind};

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
