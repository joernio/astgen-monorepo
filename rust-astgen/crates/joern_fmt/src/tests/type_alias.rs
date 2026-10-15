use super::check_type_alias_type_full_name;

#[test]
fn alias() {
    check_type_alias_type_full_name(
        r#"
type $0A = u8;
"#,
        "ra_test_fixture::A",
    );
}

#[test]
fn generic_alias() {
    check_type_alias_type_full_name(
        r#"
struct S<T>(T);
type $0A<T> = S<T>;
"#,
        "ra_test_fixture::A<T>",
    );
}

#[test]
fn alias_in_module() {
    check_type_alias_type_full_name(
        r#"
mod m {
    pub type $0A = u8;
}
"#,
        "ra_test_fixture::m::A",
    );
}

#[test]
fn assoc_type_in_trait() {
    check_type_alias_type_full_name(
        r#"
trait Tr {
    type $0A;
}
"#,
        "ra_test_fixture::Tr::A",
    );
}

#[test]
fn assoc_type_in_trait_impl() {
    check_type_alias_type_full_name(
        r#"
struct S;
trait Tr {
    type A;
}
impl Tr for S {
    type $0A = u8;
}
"#,
        "<ra_test_fixture::S as ra_test_fixture::Tr>::A",
    );
}

#[test]
fn local_alias() {
    check_type_alias_type_full_name(
        r#"
fn f() {
    type $0A = u8;
}
"#,
        "ra_test_fixture::f::A",
    );
}

#[test]
fn local_alias_in_sibling_block_1() {
    check_type_alias_type_full_name(
        r#"
fn f() {
    {
        type $0A = u8;
    }
    {
        type A = u16;
    }
}
"#,
        "ra_test_fixture::f::A#1",
    );
}

#[test]
fn local_alias_in_sibling_block_2() {
    check_type_alias_type_full_name(
        r#"
fn f() {
    {
        type A = u8;
    }
    {
        type $0A = u16;
    }
}
"#,
        "ra_test_fixture::f::A#2",
    );
}

#[test]
fn local_alias_in_block_module_inside_fn() {
    check_type_alias_type_full_name(
        r#"
fn f() {
    mod util {
        pub type $0Mask = u8;
    }
}
"#,
        "ra_test_fixture::f::util::Mask",
    );
}

#[test]
fn local_alias_in_sibling_block_modules_1() {
    check_type_alias_type_full_name(
        r#"
fn f() {
    {
        mod util {
            pub type $0Mask = u8;
        }
    }
    {
        mod util {
            pub type Mask = u16;
        }
    }
}
"#,
        "ra_test_fixture::f::util#1::Mask",
    );
}

#[test]
fn local_alias_in_sibling_block_modules_2() {
    check_type_alias_type_full_name(
        r#"
fn f() {
    {
        mod util {
            pub type Mask = u8;
        }
    }
    {
        mod util {
            pub type $0Mask = u16;
        }
    }
}
"#,
        "ra_test_fixture::f::util#2::Mask",
    );
}
