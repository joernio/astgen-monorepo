mod common;

use crate::common::{TestResult, path_expr, ref_expr, sysroot_ast_json};
use serde_json::json;

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
            json!({"kind": "cast", "source": "&alloc::string::String", "target": "&dyn Display + 'static"}),
        ],
    );

    Ok(())
}
