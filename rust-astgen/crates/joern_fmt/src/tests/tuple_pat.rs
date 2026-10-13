use super::check_tuple_pat_total_elements;

#[test]
fn tuple_with_rest() {
    check_tuple_pat_total_elements(
        r#"
fn f(t: (u8, u8, u8)) {
    let $0(a, .., b) = t;
}
"#,
        Some(3),
    );
}

#[test]
fn tuple_ref_with_rest() {
    check_tuple_pat_total_elements(
        r#"
fn f(t: &(u8, u8, u8)) {
    let $0(a, .., b) = t;
}
"#,
        Some(3),
    );
}

#[test]
fn tuple_without_rest() {
    check_tuple_pat_total_elements(
        r#"
fn f(t: (u8, u8, u8)) {
    let $0(a, b, c) = t;
}
"#,
        None,
    );
}
