use super::check_self_param_type_full_name;

#[test]
fn by_ref_self_in_imported_trait() {
    check_self_param_type_full_name(
        r#"
mod imported {
    pub struct Type;

    pub trait Trait {
        fn trait_value($0&self) -> bool;
    }
}

use imported::{Trait, Type};

impl Type {
    fn value(&self) -> bool {
        true
    }

    fn by_mut(&mut self) -> bool {
        true
    }
}

impl Trait for Type {
    fn trait_value(&self) -> bool {
        true
    }
}

fn main() {}
"#,
        "&ra_test_fixture::imported::Trait",
    );
}

#[test]
fn by_ref_self_in_inherent_impl_of_imported_struct() {
    check_self_param_type_full_name(
        r#"
mod imported {
    pub struct Type;

    pub trait Trait {
        fn trait_value(&self) -> bool;
    }
}

use imported::{Trait, Type};

impl Type {
    fn value($0&self) -> bool {
        true
    }

    fn by_mut(&mut self) -> bool {
        true
    }
}

impl Trait for Type {
    fn trait_value(&self) -> bool {
        true
    }
}

fn main() {}
"#,
        "&ra_test_fixture::imported::Type",
    );
}

#[test]
fn by_ref_self_in_trait_impl_of_imported_struct() {
    check_self_param_type_full_name(
        r#"
mod imported {
    pub struct Type;

    pub trait Trait {
        fn trait_value(&self) -> bool;
    }
}

use imported::{Trait, Type};

impl Type {
    fn value(&self) -> bool {
        true
    }

    fn by_mut(&mut self) -> bool {
        true
    }
}

impl Trait for Type {
    fn trait_value($0&self) -> bool {
        true
    }
}

fn main() {}
"#,
        "&ra_test_fixture::imported::Type",
    );
}

#[test]
fn by_mut_ref_self_in_inherent_impl_of_imported_struct() {
    check_self_param_type_full_name(
        r#"
mod imported {
    pub struct Type;

    pub trait Trait {
        fn trait_value(&self) -> bool;
    }
}

use imported::{Trait, Type};

impl Type {
    fn value(&self) -> bool {
        true
    }

    fn by_mut($0&mut self) -> bool {
        true
    }
}

impl Trait for Type {
    fn trait_value(&self) -> bool {
        true
    }
}

fn main() {}
"#,
        "&mut ra_test_fixture::imported::Type",
    );
}

#[test]
fn by_ref_self_in_trait() {
    check_self_param_type_full_name(
        r#"
trait Tr {
    fn m($0&self);
    fn n(&mut self);
    fn o(self);
}

fn main() {}
"#,
        "&ra_test_fixture::Tr",
    );
}

#[test]
fn by_mut_ref_self_in_trait() {
    check_self_param_type_full_name(
        r#"
trait Tr {
    fn m(&self);
    fn n($0&mut self);
    fn o(self);
}

fn main() {}
"#,
        "&mut ra_test_fixture::Tr",
    );
}

#[test]
fn by_value_self_in_trait() {
    check_self_param_type_full_name(
        r#"
trait Tr {
    fn m(&self);
    fn n(&mut self);
    fn o($0self);
}

fn main() {}
"#,
        "ra_test_fixture::Tr",
    );
}
