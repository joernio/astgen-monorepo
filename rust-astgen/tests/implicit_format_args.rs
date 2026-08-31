mod common;

use crate::common::{
    TestResult, format_args_arg, name_ref, nodes_by_kind, path_expr, sysroot_ast_json,
};

#[test]
fn implicit_capture() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() { let n = "a"; let _ = format!("p/{n}"); }
"#,
        )],
        "src/main.rs",
    )?;

    assert!(format_args_arg(&json, "n").exists());
    assert_eq!(path_expr(&json, "n").type_full_name(), "&str");
    assert_eq!(name_ref(&json, "n").type_full_name(), "&str");

    Ok(())
}

#[test]
fn one_placeholder_capturing_two_names() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() { let n = 1u8; let w = 3usize; let _ = format!("{n:>w$}"); }
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(path_expr(&json, "n").type_full_name(), "u8");
    assert_eq!(path_expr(&json, "w").type_full_name(), "usize");

    Ok(())
}

#[test]
fn named_arg_isnt_capture() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() { let n = 1u8; let _ = format!("{a}", a = n); }
"#,
        )],
        "src/main.rs",
    )?;

    assert!(!format_args_arg(&json, "a").exists());
    assert!(format_args_arg(&json, "a = n").exists());

    Ok(())
}

#[test]
fn duplicate_capture_one_format_arg() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() { let n = 1u8; let _ = format!("{n} {n}"); }
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(nodes_by_kind(&json, "FORMAT_ARGS_ARG").len(), 1);

    Ok(())
}

#[test]
fn const_capture() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
const C: u8 = 1;
fn main() { let _ = format!("{C}"); }
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(path_expr(&json, "C").type_full_name(), "u8");

    Ok(())
}

#[test]
fn static_capture() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
static S: u8 = 1;
fn main() { let _ = format!("{S}"); }
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(path_expr(&json, "S").type_full_name(), "u8");

    Ok(())
}

#[test]
fn asm_captures_nothing() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() {
    let n = 1u64;
    unsafe { core::arch::asm!("mov {r}, {r}", r = inout(reg) _, in(reg) n); }
}
"#,
        )],
        "src/main.rs",
    )?;

    assert!(nodes_by_kind(&json, "FORMAT_ARGS_ARG").is_empty());

    Ok(())
}
