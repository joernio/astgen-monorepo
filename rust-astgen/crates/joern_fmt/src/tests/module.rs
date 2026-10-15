use super::check_module_type_full_name;

#[test]
fn inline_module() {
    check_module_type_full_name(
        r#"
mod $0m {}
"#,
        Some("ra_test_fixture::m"),
    );
}

#[test]
fn nested_inline_module() {
    check_module_type_full_name(
        r#"
mod m {
    pub mod $0n {}
}
"#,
        Some("ra_test_fixture::m::n"),
    );
}

#[test]
fn cfg_inactive_module() {
    check_module_type_full_name(
        r#"
#[cfg(any())]
mod $0m {}

#[cfg(all())]
mod m {}
"#,
        None,
    );
}

#[test]
fn cfg_active_module() {
    check_module_type_full_name(
        r#"
#[cfg(any())]
mod m {}

#[cfg(all())]
mod $0m {}
"#,
        Some("ra_test_fixture::m"),
    );
}

#[test]
fn block_local_module() {
    check_module_type_full_name(
        r#"
fn f() {
    mod $0m {}
}
"#,
        Some("ra_test_fixture::f::m"),
    );
}

#[test]
fn same_named_block_local_modules() {
    check_module_type_full_name(
        r#"
fn f() {
    {
        mod m {}
    }
    {
        mod $0m {}
    }
}
"#,
        Some("ra_test_fixture::f::m#2"),
    );
}
