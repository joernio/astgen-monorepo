use super::{check_trait_supertraits, check_trait_type_full_name};

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

#[test]
fn trait_type_full_name() {
    check_trait_type_full_name(
        r#"
trait $0Tr {}
"#,
        "ra_test_fixture::Tr",
    );
}

#[test]
fn generic_trait_type_full_name() {
    check_trait_type_full_name(
        r#"
trait $0Tr<T> {}
"#,
        "ra_test_fixture::Tr<T>",
    );
}

#[test]
fn trait_in_module_type_full_name() {
    check_trait_type_full_name(
        r#"
mod m {
    pub trait $0Tr {}
}
"#,
        "ra_test_fixture::m::Tr",
    );
}

#[test]
fn local_trait_type_full_name() {
    check_trait_type_full_name(
        r#"
fn f() {
    trait $0Tr {}
}
"#,
        "ra_test_fixture::f::Tr",
    );
}

#[test]
fn local_traits_in_sibling_blocks_type_full_name() {
    check_trait_type_full_name(
        r#"
fn f() {
    {
        trait Tr {}
    }
    {
        trait $0Tr {}
    }
}
"#,
        "ra_test_fixture::f::Tr#2",
    );
}

#[test]
fn trait_in_anonymous_const_in_fn_type_full_name() {
    check_trait_type_full_name(
        r#"
fn f() {
    const _: () = if false {
        trait $0Tr {}
    };
}
"#,
        "ra_test_fixture::f::Tr",
    );
}

#[test]
fn traits_in_sibling_anonymous_consts_type_full_name_1() {
    check_trait_type_full_name(
        r#"
const _: () = {
    trait $0Tr {}
};
const _: () = {
    trait Tr {}
};
"#,
        "ra_test_fixture::Tr#1",
    );
}

#[test]
fn traits_in_sibling_anonymous_consts_type_full_name_2() {
    check_trait_type_full_name(
        r#"
const _: () = {
    trait Tr {}
};
const _: () = {
    trait $0Tr {}
};
"#,
        "ra_test_fixture::Tr#2",
    );
}

#[test]
fn trait_in_attribute_expanded_anonymous_const_type_full_name() {
    check_trait_type_full_name(
        r#"
//- proc_macros: identity
#[proc_macros::identity]
const _: () = {
    trait $0Tr {}
};
const _: () = {
    trait Tr {}
};
"#,
        "ra_test_fixture::Tr#1",
    );
}

#[test]
fn trait_next_to_attribute_expanded_anonymous_const_type_full_name() {
    check_trait_type_full_name(
        r#"
//- proc_macros: identity
#[proc_macros::identity]
const _: () = {
    trait Tr {}
};
const _: () = {
    trait $0Tr {}
};
"#,
        "ra_test_fixture::Tr#2",
    );
}
