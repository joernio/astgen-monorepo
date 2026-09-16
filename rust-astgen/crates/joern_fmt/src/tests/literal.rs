use super::check_literal_type_full_name;

#[test]
fn lhs_of_int_sum() {
    check_literal_type_full_name(
        r#"
fn main() {
    let sum = $01u32 + 2u32;
}
"#,
        "u32",
    );
}

#[test]
fn rhs_of_int_sum() {
    check_literal_type_full_name(
        r#"
fn main() {
    let sum = 1u32 + $02u32;
}
"#,
        "u32",
    );
}
