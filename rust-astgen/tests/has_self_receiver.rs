mod common;

use crate::common::{TestResult, call_expr, method_call_expr, no_sysroot_ast_json};

#[test]
fn inherent_method_tagged_only_in_function_call_form() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct Circle {
    radius: f64,
}

impl Circle {
    // A standard method that takes an immutable reference to self
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

fn main() {
    let my_circle = Circle { radius: 5.0 };

    // --- 1. Methods taking &self ---

    // Option A: Traditional method/receiver syntax
    let area1 = my_circle.area();

    // Option B: Regular function syntax (Fully Qualified Syntax)
    // We must explicitly pass the reference because the function expects `&self`
    let area2 = Circle::area(&my_circle);
}
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        method_call_expr(&json, "my_circle.area()").has_self_receiver(),
        None
    );
    assert_eq!(
        call_expr(&json, "Circle::area(&my_circle)").has_self_receiver(),
        Some(true)
    );

    Ok(())
}

#[test]
fn trait_method_via_trait_name() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct Type;
trait Tr { fn m(&self) -> bool; }
impl Tr for Type { fn m(&self) -> bool { true } }
fn main() { let t = Type; let _ = Tr::m(&t); }
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        call_expr(&json, "Tr::m(&t)").has_self_receiver(),
        Some(true)
    );

    Ok(())
}

#[test]
fn qualified_trait_method() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct Type;
trait Tr { fn m(&self) -> bool; }
impl Tr for Type { fn m(&self) -> bool { true } }
fn main() { let t = Type; let _ = <Type as Tr>::m(&t); }
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(
        call_expr(&json, "<Type as Tr>::m(&t)").has_self_receiver(),
        Some(true)
    );

    Ok(())
}

#[test]
fn free_function() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn free(a: i32, b: i32) -> i32 { a + b }
fn main() { let _ = free(1, 2); }
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(call_expr(&json, "free(1, 2)").has_self_receiver(), None);

    Ok(())
}

#[test]
fn assoc_fn_without_self() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct Type;
impl Type { fn new() -> Type { Type } }
fn main() { let _ = Type::new(); }
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(call_expr(&json, "Type::new()").has_self_receiver(), None);

    Ok(())
}

#[test]
fn tuple_struct_ctor() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct Point(i32, i32);
fn main() { let _ = Point(1, 2); }
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(call_expr(&json, "Point(1, 2)").has_self_receiver(), None);

    Ok(())
}

#[test]
fn tuple_enum_variant() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
enum E { Wrap(i32) }
fn main() { let _ = E::Wrap(1); }
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(call_expr(&json, "E::Wrap(1)").has_self_receiver(), None);

    Ok(())
}

#[test]
fn closure_value_call() -> TestResult<()> {
    let json = no_sysroot_ast_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
fn main() { let f = |x: i32| x; let _ = f(1); }
"#,
        )],
        "src/main.rs",
    )?;

    assert_eq!(call_expr(&json, "f(1)").has_self_receiver(), None);

    Ok(())
}
