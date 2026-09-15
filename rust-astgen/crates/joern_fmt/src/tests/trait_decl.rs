use super::check_trait_supertraits;

#[test]
fn trait_with_supertrait() {
    check_trait_supertraits(
        r#"
trait Tr {}

trait $0Sub: Tr {}
"#,
        Some(vec!["ra_test_fixture::Tr"]),
    );
}

#[test]
fn trait_with_generic_supertrait() {
    check_trait_supertraits(
        r#"
trait Tr<T> {}

trait $0Sub: Tr<u8> {}
"#,
        Some(vec!["ra_test_fixture::Tr<u8>"]),
    );
}

#[test]
fn trait_with_two_supertraits() {
    check_trait_supertraits(
        r#"
trait Tr1 {}
trait Tr2 {}

trait $0Sub: Tr2 + Tr1 {}
"#,
        Some(vec!["ra_test_fixture::Tr1", "ra_test_fixture::Tr2"]),
    );
}

#[test]
fn trait_with_supertrait_and_lifetime_bound() {
    check_trait_supertraits(
        r#"
trait Tr {}

trait $0Sub: Tr + 'static {}
"#,
        Some(vec!["ra_test_fixture::Tr"]),
    );
}

#[test]
fn trait_without_supertraits() {
    check_trait_supertraits(
        r#"
trait $0Tr {}
"#,
        None,
    );
}
