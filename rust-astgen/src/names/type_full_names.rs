//! Where we finally build `typeFullName` for each (relevant) SyntaxNode.

use super::{
    rust_name_formatter::{format_module_def_full_name, format_name_with_generic_args},
    type_formatter,
};
use ra_ap_hir::{Module, ModuleDef, PathResolution, Semantics, Type};
use ra_ap_ide::RootDatabase;
use ra_ap_syntax::{AstNode, SyntaxNode, ast, ast::HasGenericArgs, match_ast};

pub(crate) fn type_full_name_for_node(
    node: &SyntaxNode,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    match_ast! {
        match node {
            ast::Expr(expr) => resolve_expr_type_full_name(&expr, semantics),
            ast::IdentPat(ident_pat) => resolve_ident_pat_type_full_name(&ident_pat, semantics),
            ast::NameRef(name_ref) => resolve_name_ref_type_full_name(&name_ref, semantics),
            ast::SelfParam(self_param) => resolve_self_param_type_full_name(&self_param, semantics),
            _ => None,
        }
    }
}

fn resolve_expr_type_full_name(
    expr: &ast::Expr,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    if let Some(type_name) = semantics
        .type_of_expr(expr)
        .and_then(|typ| format_node_type_full_name(typ.adjusted(), expr.syntax(), semantics))
    {
        return Some(type_name);
    }

    if let ast::Expr::MethodCallExpr(method_call_expr) = expr {
        resolve_method_call_return_type_full_name(method_call_expr, semantics)
    } else {
        None
    }
}

fn resolve_method_call_return_type_full_name<'db>(
    method_call_expr: &ast::MethodCallExpr,
    semantics: &Semantics<'db, RootDatabase>,
) -> Option<String> {
    let typ = semantics
        .resolve_method_call_as_callable(method_call_expr)?
        .return_type();
    format_node_type_full_name(typ, method_call_expr.syntax(), semantics)
}

fn resolve_ident_pat_type_full_name(
    ident_pat: &ast::IdentPat,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let typ = semantics.type_of_binding_in_pat(ident_pat)?;
    format_node_type_full_name(typ, ident_pat.syntax(), semantics)
}

fn resolve_self_param_type_full_name(
    self_param: &ast::SelfParam,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let typ = semantics.type_of_self(self_param)?;
    format_node_type_full_name(typ, self_param.syntax(), semantics)
}

fn resolve_name_ref_type_full_name(
    name_ref: &ast::NameRef,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let path = path_for_name_ref(name_ref)?;
    if let Some(path_expr) = terminal_path_expr(&path) {
        // if it's the rightmost (whole/terminal expression), resolve the type
        resolve_expr_type_full_name(&ast::Expr::PathExpr(path_expr), semantics)
    } else {
        // otherwise, resolve the path instead.
        let resolution = semantics.resolve_path(&path)?;
        format_path_resolution_type_full_name(resolution, &path, semantics)
    }
}

fn path_for_name_ref(name_ref: &ast::NameRef) -> Option<ast::Path> {
    let path_segment = name_ref
        .syntax()
        .parent()
        .and_then(ast::PathSegment::cast)?;
    path_segment.syntax().parent().and_then(ast::Path::cast)
}

fn terminal_path_expr(path: &ast::Path) -> Option<ast::PathExpr> {
    path.syntax().parent().and_then(ast::PathExpr::cast)
}

fn format_path_resolution_type_full_name<'db>(
    resolution: PathResolution,
    path: &ast::Path,
    semantics: &Semantics<'db, RootDatabase>,
) -> Option<String> {
    let module = semantics.scope(path.syntax())?.module();
    match resolution {
        PathResolution::Def(ModuleDef::Adt(adt)) => format_module_def_type_full_name(
            ModuleDef::from(adt),
            path,
            module,
            semantics.db,
            semantics,
        ),
        PathResolution::Def(ModuleDef::TypeAlias(type_alias)) => format_module_def_type_full_name(
            ModuleDef::from(type_alias),
            path,
            module,
            semantics.db,
            semantics,
        ),
        PathResolution::Def(ModuleDef::BuiltinType(builtin)) => {
            type_formatter::format_type(builtin.ty(semantics.db), module, semantics.db)
        }
        PathResolution::TypeParam(type_param) => {
            type_formatter::format_type(type_param.ty(semantics.db), module, semantics.db)
        }
        PathResolution::SelfType(impl_) => {
            type_formatter::format_type(impl_.self_ty(semantics.db), module, semantics.db)
        }
        _ => None,
    }
}

fn format_module_def_type_full_name<'db>(
    def: ModuleDef,
    path: &ast::Path,
    module: Module,
    db: &'db RootDatabase,
    semantics: &Semantics<'db, RootDatabase>,
) -> Option<String> {
    let base = format_module_def_full_name(def, db)?;
    let generic_args = generic_args_for_path(path, module, semantics)?;
    Some(format_name_with_generic_args(base, generic_args))
}

fn generic_args_for_path<'db>(
    path: &ast::Path,
    module: Module,
    semantics: &Semantics<'db, RootDatabase>,
) -> Option<Vec<String>> {
    path.segment()?
        .generic_arg_list()
        .map(|arg_list| {
            arg_list
                .generic_args()
                .map(|arg| format_generic_arg(arg, module, semantics))
                .collect()
        })
        .unwrap_or_else(|| Some(Vec::new()))
}

fn format_generic_arg(
    arg: ast::GenericArg,
    module: Module,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    match arg {
        ast::GenericArg::TypeArg(type_arg) => {
            let typ = semantics.resolve_type(&type_arg.ty()?)?;
            type_formatter::format_type(typ, module, semantics.db)
        }
        ast::GenericArg::ConstArg(const_arg) => Some(const_arg.syntax().text().to_string()),
        _ => None,
    }
}

fn format_node_type_full_name<'db>(
    typ: Type<'db>,
    node: &SyntaxNode,
    semantics: &Semantics<'db, RootDatabase>,
) -> Option<String> {
    let module = semantics.scope(node)?.module();
    type_formatter::format_type(typ, module, semantics.db)
}
