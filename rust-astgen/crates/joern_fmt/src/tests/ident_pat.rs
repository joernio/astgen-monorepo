use super::check_ident_pat_type_full_name;

#[test]
fn turbofish_inherent_fn_call() {
    check_ident_pat_type_full_name(
        r#"
struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new(value: T) -> Wrapper<T> {
        Wrapper(value)
    }
}

fn main() {
    let $0wrapped = Wrapper::<u32>::new(1);
}
"#,
        "ra_test_fixture::Wrapper<u32>",
    );
}

#[test]
fn param_of_generic_inherent_fn() {
    check_ident_pat_type_full_name(
        r#"
trait Extract<T> {
    fn extract(&self) -> T;
}

struct Wrapper<T>(T);

impl<T: Copy> Wrapper<T> {
    fn new($0value: T) -> Wrapper<T> {
        Wrapper(value)
    }
}

impl<T: Copy> Extract<T> for Wrapper<T> {
    fn extract(&self) -> T {
        self.0
    }
}

fn main() {}
"#,
        "T",
    );
}

#[test]
fn let_of_generic_tuple_struct() {
    check_ident_pat_type_full_name(
        r#"
struct Wrapper<T>(T);

fn main() {
    let $0plain = Wrapper(3u32);
    let copied = plain;
}
"#,
        "ra_test_fixture::Wrapper<u32>",
    );
}

#[test]
fn let_of_moved_generic_tuple_struct() {
    check_ident_pat_type_full_name(
        r#"
struct Wrapper<T>(T);

fn main() {
    let plain = Wrapper(3u32);
    let $0copied = plain;
}
"#,
        "ra_test_fixture::Wrapper<u32>",
    );
}

#[test]
fn inherent_method_of_imported_struct() {
    check_ident_pat_type_full_name(
        r#"
mod imported {
    pub struct Type;
}

use imported::Type;

impl Type {
    fn value(&self) -> bool {
        true
    }
}

fn main() {
    let receiver = Type;
    let $0method_value = receiver.value();
}
"#,
        "bool",
    );
}

#[test]
fn let_of_str_ref() {
    check_ident_pat_type_full_name(
        r#"
fn main() {
    let text: &str = "hello";
    let $0text_copy = text;
    let mut number = 1u32;
    let number_ref: &mut u32 = &mut number;
    let number_ref_copy = number_ref;
}
"#,
        "&str",
    );
}

#[test]
fn mut_ref() {
    check_ident_pat_type_full_name(
        r#"
fn main() {
    let text: &str = "hello";
    let text_copy = text;
    let mut number = 1u32;
    let $0number_ref: &mut u32 = &mut number;
    let number_ref_copy = number_ref;
}
"#,
        "&mut u32",
    );
}

#[test]
fn let_of_mut_ref() {
    check_ident_pat_type_full_name(
        r#"
fn main() {
    let text: &str = "hello";
    let text_copy = text;
    let mut number = 1u32;
    let number_ref: &mut u32 = &mut number;
    let $0number_ref_copy = number_ref;
}
"#,
        "&mut u32",
    );
}

#[test]
fn let_of_int_sum() {
    check_ident_pat_type_full_name(
        r#"
fn main() {
    let $0sum = 1u32 + 2u32;
}
"#,
        "u32",
    );
}

#[test]
fn const_raw_pointer_param() {
    check_ident_pat_type_full_name(
        r#"
fn takes($0p_const: *const i32, p_mut: *mut i32) {
    let _ = p_const;
    let _ = p_mut;
}

fn main() {}
"#,
        "*const i32",
    );
}

#[test]
fn mut_raw_pointer_param() {
    check_ident_pat_type_full_name(
        r#"
fn takes(p_const: *const i32, $0p_mut: *mut i32) {
    let _ = p_const;
    let _ = p_mut;
}

fn main() {}
"#,
        "*mut i32",
    );
}

#[test]
fn let_of_const_raw_pointer() {
    check_ident_pat_type_full_name(
        r#"
fn returns_const() -> *const i32 {
    0 as *const i32
}

fn returns_mut() -> *mut i32 {
    0 as *mut i32
}

fn main() {
    let $0const_ptr = returns_const();
    let mut_ptr = returns_mut();
}
"#,
        "*const i32",
    );
}

#[test]
fn let_of_mut_raw_pointer() {
    check_ident_pat_type_full_name(
        r#"
fn returns_const() -> *const i32 {
    0 as *const i32
}

fn returns_mut() -> *mut i32 {
    0 as *mut i32
}

fn main() {
    let const_ptr = returns_const();
    let $0mut_ptr = returns_mut();
}
"#,
        "*mut i32",
    );
}

#[test]
fn const_raw_pointer_to_struct_param() {
    check_ident_pat_type_full_name(
        r#"
struct Type;

fn takes($0const_p: *const Type, mut_p: *mut Type) {
    let _ = const_p;
    let _ = mut_p;
}

fn main() {}
"#,
        "*const ra_test_fixture::Type",
    );
}

#[test]
fn mut_raw_pointer_to_struct_param() {
    check_ident_pat_type_full_name(
        r#"
struct Type;

fn takes(const_p: *const Type, $0mut_p: *mut Type) {
    let _ = const_p;
    let _ = mut_p;
}

fn main() {}
"#,
        "*mut ra_test_fixture::Type",
    );
}

#[test]
fn nested_raw_pointer_param() {
    check_ident_pat_type_full_name(
        r#"
fn takes($0p: *const *mut i32) {
    let _ = p;
}

fn main() {}
"#,
        "*const *mut i32",
    );
}

#[test]
fn raw_pointer_to_slice_param() {
    check_ident_pat_type_full_name(
        r#"
fn takes($0p: *const [i32]) {
    let _ = p;
}

fn main() {}
"#,
        "*const [i32]",
    );
}

#[test]
fn let_of_unit() {
    check_ident_pat_type_full_name(
        r#"
fn main() {
    let $0unit = ();
    let single = (1i32,);
    let multi = (1i32, 2u32, true);
}
"#,
        "()",
    );
}

#[test]
fn let_of_single_element_tuple() {
    check_ident_pat_type_full_name(
        r#"
fn main() {
    let unit = ();
    let $0single = (1i32,);
    let multi = (1i32, 2u32, true);
}
"#,
        "(i32,)",
    );
}

#[test]
fn let_of_tuple() {
    check_ident_pat_type_full_name(
        r#"
fn main() {
    let unit = ();
    let single = (1i32,);
    let $0multi = (1i32, 2u32, true);
}
"#,
        "(i32, u32, bool)",
    );
}

#[test]
fn let_of_array() {
    check_ident_pat_type_full_name(
        r#"
fn main() {
    let $0arr = [0, 1, 2, 3];
}
"#,
        "[i32; 4]",
    );
}

#[test]
fn let_of_slice_ref() {
    check_ident_pat_type_full_name(
        r#"
fn main() {
    let arr: [i32; 3] = [1, 2, 3];
    let $0slice: &[i32] = &arr;
}
"#,
        "&[i32]",
    );
}

#[test]
fn let_of_fn_pointer_without_args() {
    check_ident_pat_type_full_name(
        r#"
fn no_args() -> u32 {
    1
}

fn with_args(a: u32, b: bool) -> u32 {
    if b { a + 1 } else { a }
}

fn main() {
    let $0no_arg_ptr: fn() -> u32 = no_args;
    let multi_arg_ptr: fn(u32, bool) -> u32 = with_args;
}
"#,
        "fn() -> u32",
    );
}

#[test]
fn let_of_fn_pointer_with_args() {
    check_ident_pat_type_full_name(
        r#"
fn no_args() -> u32 {
    1
}

fn with_args(a: u32, b: bool) -> u32 {
    if b { a + 1 } else { a }
}

fn main() {
    let no_arg_ptr: fn() -> u32 = no_args;
    let $0multi_arg_ptr: fn(u32, bool) -> u32 = with_args;
}
"#,
        "fn(u32, bool) -> u32",
    );
}

#[test]
fn local_struct_in_sibling_block_1() {
    check_ident_pat_type_full_name(
        r#"
fn f() {
    if true {
        struct S { x: i32 }
        impl S { fn new() -> S { S { x: 1 } } }
        let $0a = S::new();
        let _ = a.x;
    }
    if false {
        struct S { x: u8 }
        impl S { fn new() -> S { S { x: 2 } } }
        let b = S::new();
        let _ = b.x;
    }
}

fn main() { f(); }
"#,
        "ra_test_fixture::f::S#1",
    );
}

#[test]
fn local_struct_in_sibling_block_2() {
    check_ident_pat_type_full_name(
        r#"
fn f() {
    if true {
        struct S { x: i32 }
        impl S { fn new() -> S { S { x: 1 } } }
        let a = S::new();
        let _ = a.x;
    }
    if false {
        struct S { x: u8 }
        impl S { fn new() -> S { S { x: 2 } } }
        let $0b = S::new();
        let _ = b.x;
    }
}

fn main() { f(); }
"#,
        "ra_test_fixture::f::S#2",
    );
}
