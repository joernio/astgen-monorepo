mod common;

use crate::common::{
    TestResult, closure_expr, no_sysroot_ast_json, path_expr, ref_expr, return_expr,
};
use serde_json::json;

#[test]
fn method_receiver_autoref() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct Type;
impl Type {
    fn value(&self) -> bool {
        true
    }
}
fn main() {
    let receiver = Type;
    let _ = receiver.value();
}
"#,
        )],
        "src/main.rs",
    )?;

    let receiver = path_expr(&json, "receiver").on_line("    let _ = receiver.value();");
    assert_eq!(
        receiver.adjustments(),
        vec![json!({
            "kind": "borrow",
            "source": "rust2cpg::Type",
            "target": "&rust2cpg::Type",
        })],
    );

    Ok(())
}

#[test]
fn reborrow_mut_to_shared_at_call_arg() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn take_ref(_s: &i32) {}
fn main() {
    let mut n = 1i32;
    take_ref(&mut n);
}
"#,
        )],
        "src/main.rs",
    )?;

    let arg = ref_expr(&json, "&mut n");
    assert_eq!(
        arg.adjustments(),
        vec![
            json!({"kind": "deref", "source": "&mut i32", "target": "i32"}),
            json!({"kind": "borrow", "source": "i32", "target": "&i32"}),
        ],
    );

    Ok(())
}

#[test]
fn type_full_name_is_the_unadjusted_source() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn take_ref(_s: &i32) {}
fn main() {
    let mut n = 1i32;
    take_ref(&mut n);
}
"#,
        )],
        "src/main.rs",
    )?;

    let arg = ref_expr(&json, "&mut n");
    assert_eq!(arg.type_full_name(), "&mut i32");
    assert_eq!(
        arg.adjustments(),
        vec![
            json!({"kind": "deref", "source": "&mut i32", "target": "i32"}),
            json!({"kind": "borrow", "source": "i32", "target": "&i32"})
        ]
    );

    Ok(())
}

#[test]
fn never_to_any() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() {
    let c = true;
    let _x: i32 = if c { 1 } else { return };
}
"#,
        )],
        "src/main.rs",
    )?;

    let ret = return_expr(&json, "return");
    assert_eq!(
        ret.adjustments(),
        vec![json!({"kind": "cast", "source": "!", "target": "i32"})],
    );

    Ok(())
}

#[test]
fn fn_item_to_fn_pointer_reify() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn g() {}
fn main() {
    let _f: fn() = g;
}
"#,
        )],
        "src/main.rs",
    )?;

    let callee = path_expr(&json, "g");
    assert_eq!(
        callee.adjustments(),
        vec![json!({"kind": "cast", "source": "fn() -> ()", "target": "fn() -> ()"})],
    );

    Ok(())
}

#[test]
fn closure_to_fn_pointer() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() {
    let _f: fn() = || {};
}
"#,
        )],
        "src/main.rs",
    )?;

    let closure = closure_expr(&json, "|| {}");
    assert_eq!(
        closure.adjustments(),
        vec![json!({"kind": "cast", "source": "impl Fn()", "target": "fn() -> ()"})],
    );

    Ok(())
}

#[test]
fn safe_to_unsafe_fn_pointer() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn g() {}
fn main() {
    let p: fn() = g;
    let _u: unsafe fn() = p;
}
"#,
        )],
        "src/main.rs",
    )?;

    let coerced = path_expr(&json, "p").on_line("    let _u: unsafe fn() = p;");
    assert_eq!(
        coerced.adjustments(),
        vec![json!({"kind": "cast", "source": "fn() -> ()", "target": "fn() -> ()"})],
    );

    Ok(())
}

#[test]
fn mut_ref_coerced_to_raw_ptr() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() {
    let mut x = 1i32;
    let _p: *mut i32 = &mut x;
}
"#,
        )],
        "src/main.rs",
    )?;

    let init = ref_expr(&json, "&mut x");
    assert_eq!(
        init.adjustments(),
        vec![
            json!({"kind": "deref", "source": "&mut i32", "target": "i32"}),
            json!({"kind": "borrow", "source": "i32", "target": "*mut i32"}),
        ],
    );

    Ok(())
}

#[test]
fn mut_raw_ptr_to_const_raw_ptr() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() {
    let mut x = 1i32;
    let p: *mut i32 = &mut x;
    let _c: *const i32 = p;
}
"#,
        )],
        "src/main.rs",
    )?;

    let coerced = path_expr(&json, "p").on_line("    let _c: *const i32 = p;");
    assert_eq!(
        coerced.adjustments(),
        vec![json!({"kind": "cast", "source": "*mut i32", "target": "*const i32"})],
    );

    Ok(())
}
