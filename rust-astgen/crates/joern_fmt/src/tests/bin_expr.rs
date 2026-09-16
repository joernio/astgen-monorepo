use super::check_bin_type_full_name;

#[test]
fn int_sum() {
    check_bin_type_full_name(
        r#"
fn main() {
    let sum = $01u32 + 2u32;
}
"#,
        "u32",
    );
}
