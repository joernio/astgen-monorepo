//! Where we finally build `methodFullName` for each call.

use super::rust_name_formatter::{
    format_enum_variant_full_name, format_function_full_name, format_tuple_struct_ctor_full_name,
};
use ra_ap_hir::{CallableKind, ModuleDef, PathResolution, Semantics};
use ra_ap_ide::RootDatabase;
use ra_ap_syntax::{AstNode, SyntaxNode, ast, match_ast};

pub(crate) fn method_full_name_for_node(
    node: &SyntaxNode,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    match_ast! {
        match node {
            ast::CallExpr(call_expr) => resolve_call_expr_full_name(&call_expr, semantics),
            ast::MethodCallExpr(method_call_expr) => resolve_method_call_expr_full_name(&method_call_expr, semantics),
            ast::Struct(struct_) => resolve_struct_ctor_full_name(&struct_, semantics),
            ast::Fn(fn_) => resolve_fn_def_full_name(&fn_, semantics),
            ast::PathExpr(path_expr) => resolve_path_expr_full_name(&path_expr, semantics),
            _ => None,
        }
    }
}

/// Whether a `CallExpr`'s first argument is a `self` receiver.
pub(crate) fn has_self_receiver_for_node(
    node: &SyntaxNode,
    semantics: &Semantics<RootDatabase>,
) -> Option<bool> {
    let call_expr = ast::CallExpr::cast(node.clone())?;
    let callee = call_expr.expr()?;
    match semantics.resolve_expr_as_callable(&callee)?.kind() {
        CallableKind::Function(function) if function.has_self_param(semantics.db) => Some(true),
        _ => None,
    }
}

fn resolve_method_call_expr_full_name(
    method_call_expr: &ast::MethodCallExpr,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let function = semantics.resolve_method_call(method_call_expr)?;
    format_function_full_name(function, semantics.db)
}

fn resolve_path_expr_full_name(
    path_expr: &ast::PathExpr,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let path = path_expr.path()?;
    match semantics.resolve_path(&path)? {
        PathResolution::Def(ModuleDef::Function(f)) => format_function_full_name(f, semantics.db),
        _ => None,
    }
}

fn resolve_call_expr_full_name(
    call_expr: &ast::CallExpr,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let callee_expr = call_expr.expr()?;
    if let ast::Expr::PathExpr(path_expr) = &callee_expr
        && let Some(name) = resolve_path_expr_full_name(path_expr, semantics)
    {
        return Some(name);
    }
    match semantics.resolve_expr_as_callable(&callee_expr)?.kind() {
        CallableKind::Function(function) => format_function_full_name(function, semantics.db),
        CallableKind::TupleStruct(tuple_struct) => {
            format_tuple_struct_ctor_full_name(tuple_struct, semantics.db)
        }
        CallableKind::TupleEnumVariant(enum_variant) => {
            format_enum_variant_full_name(enum_variant, semantics.db)
        }
        // TODO(xavierp): need more time to understand what these should be named as
        CallableKind::Closure(_) | CallableKind::FnPtr | CallableKind::FnImpl(_) => None,
    }
}

fn resolve_struct_ctor_full_name(
    struct_: &ast::Struct,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    // We provide a `methodFullName` at struct definition to match its constructor name.
    // Only tuple structs have a callable constructor. Record/Unit structs have RecordExpr
    // and IdentExpr at call-site.
    // So, when we see "MyTuple(...)" (call-site), the `methodFullName` for this call shall
    // match the `methodFullName` for the struct ctor which we synthesize in Joern.
    let Some(ast::FieldList::TupleFieldList(_)) = struct_.field_list() else {
        return None;
    };
    let struct_def = semantics.to_def(struct_)?;
    format_tuple_struct_ctor_full_name(struct_def, semantics.db)
}

fn resolve_fn_def_full_name(fn_: &ast::Fn, semantics: &Semantics<RootDatabase>) -> Option<String> {
    let function = semantics.to_def(fn_)?;
    format_function_full_name(function, semantics.db)
}
