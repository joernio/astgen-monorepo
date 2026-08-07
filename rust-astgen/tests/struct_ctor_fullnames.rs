mod common;

use crate::common::{TestResult, call_expr, no_sysroot_ast_json, struct_decl};

#[test]
fn plain_tuple_struct() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpgtest",
        &[(
            "src/main.rs",
            r#"
struct Plain(i32, bool);

fn main() {
    let _value = Plain(1, true);
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct Plain(i32, bool);").method_full_name(),
        "rust2cpgtest::Plain"
    );
    assert_eq!(
        call_expr(&json, "Plain(1, true)").method_full_name(),
        "rust2cpgtest::Plain"
    );

    Ok(())
}

#[test]
fn single_type_param() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpgtest",
        &[(
            "src/main.rs",
            r#"
struct OneParam<T>(T);

fn main() {
    let _value = OneParam(1u32);
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct OneParam<T>(T);").method_full_name(),
        "rust2cpgtest::OneParam<T>"
    );
    assert_eq!(
        call_expr(&json, "OneParam(1u32)").method_full_name(),
        "rust2cpgtest::OneParam<T>"
    );

    Ok(())
}

#[test]
fn multiple_type_params() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpgtest",
        &[(
            "src/main.rs",
            r#"
struct Multi<A, B>(A, B);

fn main() {
    let _value = Multi(1u32, true);
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct Multi<A, B>(A, B);").method_full_name(),
        "rust2cpgtest::Multi<A, B>"
    );
    assert_eq!(
        call_expr(&json, "Multi(1u32, true)").method_full_name(),
        "rust2cpgtest::Multi<A, B>"
    );

    Ok(())
}

#[test]
fn type_param_bounds_are_stripped() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpgtest",
        &[(
            "src/main.rs",
            r#"
struct Bounded<T: Clone>(T);

fn main() {
    let _value = Bounded(1u32);
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct Bounded<T: Clone>(T);").method_full_name(),
        "rust2cpgtest::Bounded<T>"
    );
    assert_eq!(
        call_expr(&json, "Bounded(1u32)").method_full_name(),
        "rust2cpgtest::Bounded<T>"
    );

    Ok(())
}

#[test]
fn lifetime_only_struct_has_no_generic_suffix() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpgtest",
        &[(
            "src/main.rs",
            r#"
struct WithLife<'a>(&'a i32);

fn main() {
    let _value = WithLife(&1i32);
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct WithLife<'a>(&'a i32);").method_full_name(),
        "rust2cpgtest::WithLife"
    );
    assert_eq!(
        call_expr(&json, "WithLife(&1i32)").method_full_name(),
        "rust2cpgtest::WithLife"
    );

    Ok(())
}

#[test]
fn lifetimes_dropped_but_type_params_kept() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpgtest",
        &[(
            "src/main.rs",
            r#"
struct Mix<'a, T>(&'a T);

fn main() {
    let _value = Mix(&1u32);
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct Mix<'a, T>(&'a T);").method_full_name(),
        "rust2cpgtest::Mix<T>"
    );
    assert_eq!(
        call_expr(&json, "Mix(&1u32)").method_full_name(),
        "rust2cpgtest::Mix<T>"
    );

    Ok(())
}

#[test]
fn type_param_defaults_are_dropped() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpgtest",
        &[(
            "src/main.rs",
            r#"
struct WithDefault<T = i32>(T);

fn main() {
    let _value = WithDefault(1i32);
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct WithDefault<T = i32>(T);").method_full_name(),
        "rust2cpgtest::WithDefault<T>"
    );
    assert_eq!(
        call_expr(&json, "WithDefault(1i32)").method_full_name(),
        "rust2cpgtest::WithDefault<T>"
    );

    Ok(())
}

// TODO: `const` params have not been dealt with yet, just recording the status quo.
#[test]
fn const_param_renders_in_suffix() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpgtest",
        &[(
            "src/main.rs",
            r#"
struct WithConst<const N: usize>(usize);

fn main() {
    let _value = WithConst::<5>(0usize);
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct WithConst<const N: usize>(usize);").method_full_name(),
        "rust2cpgtest::WithConst<N>"
    );
    assert_eq!(
        call_expr(&json, "WithConst::<5>(0usize)").method_full_name(),
        "rust2cpgtest::WithConst<N>"
    );

    Ok(())
}

#[test]
fn record_struct_has_no_ctor_fullname() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpgtest",
        &[(
            "src/main.rs",
            r#"
struct Record { x: i32 }

fn main() {
    let _r = Record { x: 1 };
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct Record { x: i32 }").method_full_name_opt(),
        None
    );

    Ok(())
}

#[test]
fn unit_struct_has_no_ctor_fullname() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpgtest",
        &[(
            "src/main.rs",
            r#"
struct Unit;

fn main() {
    let _u = Unit;
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct Unit;").method_full_name_opt(),
        None
    );

    Ok(())
}

#[test]
fn block_local_struct_is_named_through_enclosing_fn() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpgtest",
        &[(
            "src/main.rs",
            r#"
fn f() {
    struct S(i32);
    let _ = S(1);
}

fn main() { f(); }
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        struct_decl(&json, "struct S(i32);").type_full_name(),
        "rust2cpgtest::f::S"
    );
    assert_eq!(
        call_expr(&json, "S(1)").method_full_name(),
        "rust2cpgtest::f::S"
    );

    Ok(())
}
