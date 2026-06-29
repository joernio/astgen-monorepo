mod common;

use crate::common::{TestResult, sysroot_function_fullnames_json};
use rust_ast_gen::function_fullnames_gen::FunctionFullNameEntry;

fn find_by_method_full_name<'a>(
    entries: &'a [FunctionFullNameEntry],
    method_full_name: &str,
) -> Option<&'a FunctionFullNameEntry> {
    entries
        .iter()
        .find(|entry| entry.method_full_name == method_full_name)
}

#[test]
fn dumps_public_dependency_callables_and_excludes_workspace_items() -> TestResult<()> {
    let output = sysroot_function_fullnames_json(
        "rust2cpg",
        &[(
            "src/main.rs",
            r#"
struct LocalTuple(i32);

fn workspace_fn() {}

fn main() {
}
"#,
        )],
    )?;

    let vec_new = FunctionFullNameEntry {
        method_full_name: "alloc::vec::Vec<T, alloc::alloc::Global>::new".to_owned(),
        has_self_receiver: false,
        is_trait_impl: false,
        is_trait_method_def: false,
        is_nightly_only: false,
    };
    assert_eq!(
        find_by_method_full_name(&output.functions, &vec_new.method_full_name),
        Some(&vec_new)
    );

    let option_some = FunctionFullNameEntry {
        method_full_name: "core::option::Option<T>::Some".to_owned(),
        has_self_receiver: false,
        is_trait_impl: false,
        is_trait_method_def: false,
        is_nightly_only: false,
    };
    assert_eq!(
        find_by_method_full_name(&output.functions, &option_some.method_full_name),
        Some(&option_some)
    );

    let result_unwrap_or_else = FunctionFullNameEntry {
        method_full_name: "core::result::Result<T, E>::unwrap_or_else<F>".to_owned(),
        has_self_receiver: true,
        is_trait_impl: false,
        is_trait_method_def: false,
        is_nightly_only: false,
    };
    assert_eq!(
        find_by_method_full_name(&output.functions, &result_unwrap_or_else.method_full_name),
        Some(&result_unwrap_or_else)
    );

    let iterator_next = FunctionFullNameEntry {
        method_full_name: "core::iter::traits::iterator::Iterator::next".to_owned(),
        has_self_receiver: true,
        is_trait_impl: false,
        is_trait_method_def: true,
        is_nightly_only: false,
    };
    let clone_clone = FunctionFullNameEntry {
        method_full_name: "core::clone::Clone::clone".to_owned(),
        has_self_receiver: true,
        is_trait_impl: false,
        is_trait_method_def: true,
        is_nightly_only: false,
    };
    assert!(
        find_by_method_full_name(&output.functions, &iterator_next.method_full_name)
            == Some(&iterator_next)
            || find_by_method_full_name(&output.functions, &clone_clone.method_full_name)
                == Some(&clone_clone),
        "expected Iterator::next or Clone::clone in dependency dump"
    );

    assert_eq!(
        find_by_method_full_name(&output.functions, "rust2cpg::workspace_fn"),
        None,
        "workspace free function should be excluded"
    );
    assert_eq!(
        find_by_method_full_name(&output.functions, "rust2cpg::LocalTuple"),
        None,
        "workspace tuple struct ctor should be excluded"
    );

    Ok(())
}

#[test]
fn includes_std_trait_impl_and_inherent_callables() -> TestResult<()> {
    let output = sysroot_function_fullnames_json("rust2cpg", &[("src/main.rs", "fn main() {}\n")])?;

    let string_deref = FunctionFullNameEntry {
        method_full_name: "<alloc::string::String as core::ops::deref::Deref>::deref".to_owned(),
        has_self_receiver: true,
        is_trait_impl: true,
        is_trait_method_def: false,
        is_nightly_only: false,
    };
    assert_eq!(
        find_by_method_full_name(&output.functions, &string_deref.method_full_name),
        Some(&string_deref)
    );

    let array_clone = FunctionFullNameEntry {
        method_full_name: "<[T; N] as core::clone::Clone>::clone".to_owned(),
        has_self_receiver: true,
        is_trait_impl: true,
        is_trait_method_def: false,
        is_nightly_only: false,
    };
    assert_eq!(
        find_by_method_full_name(&output.functions, &array_clone.method_full_name),
        Some(&array_clone)
    );

    let str_as_bytes = FunctionFullNameEntry {
        method_full_name: "str::as_bytes".to_owned(),
        has_self_receiver: true,
        is_trait_impl: false,
        is_trait_method_def: false,
        is_nightly_only: false,
    };
    assert_eq!(
        find_by_method_full_name(&output.functions, &str_as_bytes.method_full_name),
        Some(&str_as_bytes)
    );

    Ok(())
}
