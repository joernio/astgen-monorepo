mod bin_expr;
mod call_expr;
mod enum_decl;
mod fn_decl;
mod ident_pat;
mod impl_decl;
mod literal;
mod method_call_expr;
mod name_ref;
mod path_expr;
mod self_param;
mod struct_decl;
mod trait_decl;

use crate::{method_full_names, trait_full_names, type_full_names};
use ra_ap_hir::{Semantics, attach_db};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::{AstNode, algo::find_node_at_offset, ast};
use ra_ap_test_fixture::WithFixture;
use std::fmt::Debug;

#[track_caller]
fn check<N: AstNode, T: PartialEq + Debug>(
    ra_fixture: &str,
    resolve: impl FnOnce(&N, &Semantics<RootDatabase>) -> T,
    expected: T,
) {
    let (db, position) = RootDatabase::with_position(ra_fixture);
    let semantics = Semantics::new(&db);
    let source_file = semantics.parse(position.file_id);
    let node = find_node_at_offset::<N>(source_file.syntax(), position.offset).unwrap();
    let actual = attach_db(semantics.db, || resolve(&node, &semantics));
    assert_eq!(actual, expected);
}

#[track_caller]
fn check_fn_method_full_name(ra_fixture: &str, expected: &str) {
    check(
        ra_fixture,
        method_full_names::resolve_fn_def_full_name,
        Some(expected.to_owned()),
    );
}

#[track_caller]
fn check_call_method_full_name(ra_fixture: &str, expected: &str) {
    check(
        ra_fixture,
        method_full_names::resolve_call_expr_full_name,
        Some(expected.to_owned()),
    );
}

#[track_caller]
fn check_method_call_method_full_name(ra_fixture: &str, expected: &str) {
    check(
        ra_fixture,
        method_full_names::resolve_method_call_expr_full_name,
        Some(expected.to_owned()),
    );
}

#[track_caller]
fn check_path_method_full_name(ra_fixture: &str, expected: &str) {
    check(
        ra_fixture,
        method_full_names::resolve_path_expr_full_name,
        Some(expected.to_owned()),
    );
}

#[track_caller]
fn check_struct_method_full_name(ra_fixture: &str, expected: Option<&str>) {
    check(
        ra_fixture,
        method_full_names::resolve_struct_ctor_full_name,
        expected.map(str::to_owned),
    );
}

#[track_caller]
fn check_struct_type_full_name(ra_fixture: &str, expected: &str) {
    check(
        ra_fixture,
        type_full_names::resolve_struct_type_full_name,
        Some(expected.to_owned()),
    );
}

#[track_caller]
fn check_trait_supertraits(ra_fixture: &str, expected: Option<Vec<&str>>) {
    check(
        ra_fixture,
        trait_full_names::supertraits,
        expected.map(|names| names.into_iter().map(str::to_owned).collect()),
    );
}

#[track_caller]
fn check_call_has_self_receiver(ra_fixture: &str, expected: Option<bool>) {
    check(ra_fixture, method_full_names::has_self_receiver, expected);
}

#[track_caller]
fn check_struct_implemented_traits(ra_fixture: &str, expected: Option<Vec<&str>>) {
    check(
        ra_fixture,
        trait_full_names::implemented_traits_for_struct,
        expected.map(|names| names.into_iter().map(str::to_owned).collect()),
    );
}

#[track_caller]
fn check_enum_implemented_traits(ra_fixture: &str, expected: Option<Vec<&str>>) {
    check(
        ra_fixture,
        trait_full_names::implemented_traits_for_enum,
        expected.map(|names| names.into_iter().map(str::to_owned).collect()),
    );
}

#[track_caller]
fn check_call_type_full_name(ra_fixture: &str, expected: &str) {
    check(
        ra_fixture,
        |call: &ast::CallExpr, semantics| {
            type_full_names::resolve_expr_type_full_name(
                &ast::Expr::CallExpr(call.clone()),
                semantics,
            )
        },
        Some(expected.to_owned()),
    );
}

#[track_caller]
fn check_name_ref_type_full_name(ra_fixture: &str, expected: &str) {
    check(
        ra_fixture,
        type_full_names::resolve_name_ref_type_full_name,
        Some(expected.to_owned()),
    );
}

#[track_caller]
fn check_method_call_type_full_name(ra_fixture: &str, expected: &str) {
    check(
        ra_fixture,
        |method_call: &ast::MethodCallExpr, semantics| {
            type_full_names::resolve_expr_type_full_name(
                &ast::Expr::MethodCallExpr(method_call.clone()),
                semantics,
            )
        },
        Some(expected.to_owned()),
    );
}

#[track_caller]
fn check_ident_pat_type_full_name(ra_fixture: &str, expected: &str) {
    check(
        ra_fixture,
        type_full_names::resolve_ident_pat_type_full_name,
        Some(expected.to_owned()),
    );
}

#[track_caller]
fn check_impl_type_full_name(ra_fixture: &str, expected: &str) {
    check(
        ra_fixture,
        type_full_names::resolve_impl_type_full_name,
        Some(expected.to_owned()),
    );
}

#[track_caller]
fn check_enum_type_full_name(ra_fixture: &str, expected: &str) {
    check(
        ra_fixture,
        type_full_names::resolve_enum_type_full_name,
        Some(expected.to_owned()),
    );
}

#[track_caller]
fn check_path_type_full_name(ra_fixture: &str, expected: &str) {
    check(
        ra_fixture,
        |path: &ast::PathExpr, semantics| {
            type_full_names::resolve_expr_type_full_name(
                &ast::Expr::PathExpr(path.clone()),
                semantics,
            )
        },
        Some(expected.to_owned()),
    );
}

#[track_caller]
fn check_self_param_type_full_name(ra_fixture: &str, expected: &str) {
    check(
        ra_fixture,
        type_full_names::resolve_self_param_type_full_name,
        Some(expected.to_owned()),
    );
}

#[track_caller]
fn check_bin_type_full_name(ra_fixture: &str, expected: &str) {
    check(
        ra_fixture,
        |bin: &ast::BinExpr, semantics| {
            type_full_names::resolve_expr_type_full_name(
                &ast::Expr::BinExpr(bin.clone()),
                semantics,
            )
        },
        Some(expected.to_owned()),
    );
}

#[track_caller]
fn check_literal_type_full_name(ra_fixture: &str, expected: &str) {
    check(
        ra_fixture,
        |literal: &ast::Literal, semantics| {
            type_full_names::resolve_expr_type_full_name(
                &ast::Expr::Literal(literal.clone()),
                semantics,
            )
        },
        Some(expected.to_owned()),
    );
}
