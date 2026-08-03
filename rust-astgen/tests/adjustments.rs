mod common;

use crate::common::{
    TestResult, closure_expr, no_sysroot_ast_json, path_expr, ref_expr, return_expr,
    sysroot_ast_json,
};
use serde_json::json;

#[test]
fn method_receiver_autoref() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct Type;
impl Type {
    fn value(&self) -> bool {
        true
    }
}
fn main() {
    let receiver = Type;
    let _ = receiver.value();
}
"#,
        )],
        "src/main.rs",
    )?;

    let receiver = path_expr(&json, "receiver").on_line("    let _ = receiver.value();");
    assert_eq!(
        receiver.adjustments(),
        vec![json!({
            "kind": "borrow",
            "source": "rust2cpg::Type",
            "target": "&rust2cpg::Type",
        })],
    );

    Ok(())
}

#[test]
fn reborrow_mut_to_shared_at_call_arg() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn take_ref(_s: &i32) {}
fn main() {
    let mut n = 1i32;
    take_ref(&mut n);
}
"#,
        )],
        "src/main.rs",
    )?;

    let arg = ref_expr(&json, "&mut n");
    assert_eq!(
        arg.adjustments(),
        vec![
            json!({"kind": "deref", "source": "&mut i32", "target": "i32"}),
            json!({"kind": "borrow", "source": "i32", "target": "&i32"}),
        ],
    );

    Ok(())
}

#[test]
fn type_full_name_is_the_unadjusted_source() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn take_ref(_s: &i32) {}
fn main() {
    let mut n = 1i32;
    take_ref(&mut n);
}
"#,
        )],
        "src/main.rs",
    )?;

    let arg = ref_expr(&json, "&mut n");
    assert_eq!(arg.type_full_name(), "&mut i32");
    assert_eq!(
        arg.adjustments(),
        vec![
            json!({"kind": "deref", "source": "&mut i32", "target": "i32"}),
            json!({"kind": "borrow", "source": "i32", "target": "&i32"})
        ]
    );

    Ok(())
}

#[test]
fn never_to_any() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() {
    let c = true;
    let _x: i32 = if c { 1 } else { return };
}
"#,
        )],
        "src/main.rs",
    )?;

    let ret = return_expr(&json, "return");
    assert_eq!(
        ret.adjustments(),
        vec![json!({"kind": "cast", "source": "!", "target": "i32"})],
    );

    Ok(())
}

#[test]
fn fn_item_to_fn_pointer_reify() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn g() {}
fn main() {
    let _f: fn() = g;
}
"#,
        )],
        "src/main.rs",
    )?;

    let callee = path_expr(&json, "g");
    assert_eq!(
        callee.adjustments(),
        vec![json!({"kind": "cast", "source": "fn() -> ()", "target": "fn() -> ()"})],
    );

    Ok(())
}

#[test]
fn closure_to_fn_pointer() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() {
    let _f: fn() = || {};
}
"#,
        )],
        "src/main.rs",
    )?;

    let closure = closure_expr(&json, "|| {}");
    assert_eq!(
        closure.adjustments(),
        vec![json!({"kind": "cast", "source": "impl Fn()", "target": "fn() -> ()"})],
    );

    Ok(())
}

#[test]
fn safe_to_unsafe_fn_pointer() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn g() {}
fn main() {
    let p: fn() = g;
    let _u: unsafe fn() = p;
}
"#,
        )],
        "src/main.rs",
    )?;

    let coerced = path_expr(&json, "p").on_line("    let _u: unsafe fn() = p;");
    assert_eq!(
        coerced.adjustments(),
        vec![json!({"kind": "cast", "source": "fn() -> ()", "target": "fn() -> ()"})],
    );

    Ok(())
}

#[test]
fn mut_ref_coerced_to_raw_ptr() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() {
    let mut x = 1i32;
    let _p: *mut i32 = &mut x;
}
"#,
        )],
        "src/main.rs",
    )?;

    let init = ref_expr(&json, "&mut x");
    assert_eq!(
        init.adjustments(),
        vec![
            json!({"kind": "deref", "source": "&mut i32", "target": "i32"}),
            json!({"kind": "borrow", "source": "i32", "target": "*mut i32"}),
        ],
    );

    Ok(())
}

#[test]
fn mut_raw_ptr_to_const_raw_ptr() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() {
    let mut x = 1i32;
    let p: *mut i32 = &mut x;
    let _c: *const i32 = p;
}
"#,
        )],
        "src/main.rs",
    )?;

    let coerced = path_expr(&json, "p").on_line("    let _c: *const i32 = p;");
    assert_eq!(
        coerced.adjustments(),
        vec![json!({"kind": "cast", "source": "*mut i32", "target": "*const i32"})],
    );

    Ok(())
}

#[test]
fn std_dependent_coercions() -> TestResult<()> {
    let json = sysroot_ast_json(
        "rust2cpg",
        r#"
use std::fmt::Display;
use std::ops::Deref;
use std::rc::Rc;

struct MyBox(String);
impl Deref for MyBox {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

struct Foo {
    field: u32,
}

fn take_str(_s: &str) {}
fn take_mut_str(_s: &mut str) {}
fn take_string(_s: &String) {}

fn through_generic<T: Deref<Target = str>>(x: &T) {
    let _ = x.len();
}

fn main() {
    let owned = String::from("x");
    take_str(&owned);

    let mut owned_mut = String::from("x");
    take_mut_str(&mut owned_mut);

    let b = MyBox(String::from("x"));
    take_string(&b);

    let r = Rc::new(1i32);
    let _n: &i32 = &r;

    let bx = Box::new(Foo { field: 1 });
    let _f = bx.field;

    let _xs: &[i32] = &[1, 2, 3];

    let disp = String::from("x");
    let _d: &dyn Display = &disp;
}
"#,
    )?;

    assert_eq!(
        ref_expr(&json, "&owned").adjustments(),
        vec![
            json!({"kind": "deref", "source": "&alloc::string::String", "target": "alloc::string::String"}),
            json!({
                "kind": "overloadedDeref",
                "source": "alloc::string::String",
                "target": "str",
                "mutable": false,
                "methodFullName": "<alloc::string::String as core::ops::deref::Deref>::deref",
            }),
            json!({"kind": "borrow", "source": "str", "target": "&str"}),
        ],
    );

    assert_eq!(
        ref_expr(&json, "&mut owned_mut").adjustments(),
        vec![
            json!({"kind": "deref", "source": "&mut alloc::string::String", "target": "alloc::string::String"}),
            json!({
                "kind": "overloadedDeref",
                "source": "alloc::string::String",
                "target": "str",
                "mutable": true,
                "methodFullName": "<alloc::string::String as core::ops::deref::DerefMut>::deref_mut",
            }),
            json!({"kind": "borrow", "source": "str", "target": "&mut str"}),
        ],
    );

    assert_eq!(
        ref_expr(&json, "&b").adjustments(),
        vec![
            json!({"kind": "deref", "source": "&rust2cpg::MyBox", "target": "rust2cpg::MyBox"}),
            json!({
                "kind": "overloadedDeref",
                "source": "rust2cpg::MyBox",
                "target": "alloc::string::String",
                "mutable": false,
                "methodFullName": "<rust2cpg::MyBox as core::ops::deref::Deref>::deref",
            }),
            json!({"kind": "borrow", "source": "alloc::string::String", "target": "&alloc::string::String"}),
        ],
    );

    assert_eq!(
        ref_expr(&json, "&r").adjustments(),
        vec![
            json!({"kind": "deref", "source": "&alloc::rc::Rc<i32, alloc::alloc::Global>", "target": "alloc::rc::Rc<i32, alloc::alloc::Global>"}),
            json!({
                "kind": "overloadedDeref",
                "source": "alloc::rc::Rc<i32, alloc::alloc::Global>",
                "target": "i32",
                "mutable": false,
                "methodFullName": "<alloc::rc::Rc<T, A> as core::ops::deref::Deref>::deref",
            }),
            json!({"kind": "borrow", "source": "i32", "target": "&i32"}),
        ],
    );

    assert_eq!(
        path_expr(&json, "x")
            .on_line("    let _ = x.len();")
            .adjustments(),
        vec![
            json!({"kind": "deref", "source": "&T", "target": "T"}),
            json!({
                "kind": "overloadedDeref",
                "source": "T",
                "target": "str",
                "mutable": false,
            }),
            json!({"kind": "borrow", "source": "str", "target": "&str"}),
        ],
    );

    assert_eq!(
        path_expr(&json, "bx")
            .on_line("    let _f = bx.field;")
            .adjustments(),
        vec![json!({
            "kind": "deref",
            "source": "alloc::boxed::Box<rust2cpg::Foo, alloc::alloc::Global>",
            "target": "rust2cpg::Foo",
        })],
    );

    assert_eq!(
        ref_expr(&json, "&[1, 2, 3]").adjustments(),
        vec![
            json!({"kind": "deref", "source": "&[i32; 3]", "target": "[i32; 3]"}),
            json!({"kind": "borrow", "source": "[i32; 3]", "target": "&[i32; 3]"}),
            json!({"kind": "cast", "source": "&[i32; 3]", "target": "&[i32]"}),
        ],
    );

    assert_eq!(
        ref_expr(&json, "&disp").adjustments(),
        vec![
            json!({"kind": "deref", "source": "&alloc::string::String", "target": "alloc::string::String"}),
            json!({"kind": "borrow", "source": "alloc::string::String", "target": "&alloc::string::String"}),
            json!({"kind": "cast", "source": "&alloc::string::String", "target": "&dyn core::fmt::Display"}),
        ],
    );

    Ok(())
}
