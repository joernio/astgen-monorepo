//! Where we finally build `typeFullName` for each (relevant) SyntaxNode.

use super::{
    method_full_names::{format_enum_variant_full_name, format_generic_module_def_full_name},
    rust_name_formatter::{
        format_item_name, format_member_full_name, format_module_def_full_name,
        format_name_with_generic_args,
    },
    type_formatter,
};
use ra_ap_hir::{
    AsAssocItem, AssocItemContainer, GenericDef, Module, ModuleDef, PathResolution, Semantics,
    Type, TypeAlias,
};
use ra_ap_ide::RootDatabase;
use ra_ap_syntax::{AstNode, SyntaxNode, ast, ast::HasGenericArgs, match_ast};

pub(crate) fn type_full_name_for_node(
    node: &SyntaxNode,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    match_ast! {
        match node {
            ast::Enum(enum_) => resolve_enum_type_full_name(&enum_, semantics),
            ast::Expr(expr) => resolve_expr_type_full_name(&expr, semantics),
            ast::IdentPat(ident_pat) => resolve_ident_pat_type_full_name(&ident_pat, semantics),
            ast::NameRef(name_ref) => resolve_name_ref_type_full_name(&name_ref, semantics),
            ast::SelfParam(self_param) => resolve_self_param_type_full_name(&self_param, semantics),
            ast::Struct(struct_) => resolve_struct_type_full_name(&struct_, semantics),
            _ => None,
        }
    }
}

fn resolve_enum_type_full_name(
    enum_: &ast::Enum,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let enum_def = semantics.to_def(enum_)?;
    format_generic_module_def_full_name(
        ModuleDef::from(enum_def),
        GenericDef::from(enum_def),
        enum_def.module(semantics.db),
        semantics.db,
    )
}

fn resolve_struct_type_full_name(
    struct_: &ast::Struct,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let struct_def = semantics.to_def(struct_)?;
    format_generic_module_def_full_name(
        ModuleDef::from(struct_def),
        GenericDef::from(struct_def),
        struct_def.module(semantics.db),
        semantics.db,
    )
}

fn resolve_expr_type_full_name(
    expr: &ast::Expr,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let typ = semantics.type_of_expr(expr)?.original();
    format_node_type_full_name(typ, expr.syntax(), semantics)
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

pub(super) fn format_path_resolution_type_full_name<'db>(
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
        PathResolution::Def(ModuleDef::EnumVariant(enum_variant)) => {
            format_enum_variant_full_name(enum_variant, semantics.db)
        }
        PathResolution::Def(ModuleDef::TypeAlias(type_alias)) => {
            format_type_alias_type_full_name(type_alias, path, module, semantics)
        }
        PathResolution::Def(ModuleDef::Trait(trait_)) => format_module_def_type_full_name(
            ModuleDef::from(trait_),
            path,
            module,
            semantics.db,
            semantics,
        ),
        PathResolution::Def(ModuleDef::BuiltinType(builtin)) => {
            type_formatter::format_type(&builtin.ty(semantics.db), module, semantics.db)
        }
        PathResolution::TypeParam(type_param) => {
            type_formatter::format_type(&type_param.ty(semantics.db), module, semantics.db)
        }
        PathResolution::SelfType(impl_) => {
            type_formatter::format_impl_self_ty(impl_, module, semantics.db)
        }
        _ => None,
    }
}

fn format_type_alias_type_full_name<'db>(
    type_alias: TypeAlias,
    path: &ast::Path,
    module: Module,
    semantics: &Semantics<'db, RootDatabase>,
) -> Option<String> {
    if let Some(normalized) = type_formatter::normalize_assoc_type(path, type_alias, semantics) {
        return type_formatter::format_type(&normalized, module, semantics.db);
    }
    let base = format_type_alias_full_name(type_alias, semantics.db)?;
    let generic_args = generic_args_for_path(path, module, semantics);
    Some(format_name_with_generic_args(base, generic_args))
}

fn format_type_alias_full_name(type_alias: TypeAlias, db: &RootDatabase) -> Option<String> {
    let Some(AssocItemContainer::Trait(trait_)) =
        type_alias.as_assoc_item(db).map(|item| item.container(db))
    else {
        return format_module_def_full_name(ModuleDef::from(type_alias), db);
    };
    let trait_name = format_module_def_full_name(ModuleDef::from(trait_), db)?;
    let name = format_item_name(type_alias.name(db), type_alias.module(db), db);
    Some(format_member_full_name(&trait_name, &name))
}

fn format_module_def_type_full_name<'db>(
    def: ModuleDef,
    path: &ast::Path,
    module: Module,
    db: &'db RootDatabase,
    semantics: &Semantics<'db, RootDatabase>,
) -> Option<String> {
    let base = format_module_def_full_name(def, db)?;
    let generic_args = generic_args_for_path(path, module, semantics);
    Some(format_name_with_generic_args(base, generic_args))
}

fn generic_args_for_path<'db>(
    path: &ast::Path,
    module: Module,
    semantics: &Semantics<'db, RootDatabase>,
) -> Vec<String> {
    let Some(arg_list) = path
        .segment()
        .and_then(|segment| segment.generic_arg_list())
    else {
        return Vec::new();
    };
    arg_list
        .generic_args()
        .map(|arg| format_generic_arg(&arg, module, semantics))
        .collect()
}

fn format_generic_arg(
    arg: &ast::GenericArg,
    module: Module,
    semantics: &Semantics<RootDatabase>,
) -> String {
    match arg {
        ast::GenericArg::TypeArg(type_arg) => type_arg
            .ty()
            .and_then(|ty| resolve_type_full_name(&ty, module, semantics))
            .unwrap_or_else(|| type_arg.to_string()),
        ast::GenericArg::AssocTypeArg(assoc_type_arg) => {
            format_assoc_type_arg(assoc_type_arg, module, semantics)
                .unwrap_or_else(|| assoc_type_arg.to_string())
        }
        ast::GenericArg::LifetimeArg(lifetime_arg) => lifetime_arg.to_string(),
        ast::GenericArg::ConstArg(const_arg) => const_arg.to_string(),
    }
}

fn format_assoc_type_arg(
    assoc_type_arg: &ast::AssocTypeArg,
    module: Module,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let name = assoc_type_arg.name_ref()?;
    let typ = resolve_type_full_name(&assoc_type_arg.ty()?, module, semantics)?;
    Some(format!("{name} = {typ}"))
}

fn resolve_type_full_name(
    ty: &ast::Type,
    module: Module,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    if let Some(resolved) = semantics.resolve_type(ty)
        && let Some(formatted) = type_formatter::format_type(&resolved, module, semantics.db)
    {
        return Some(formatted);
    }
    resolve_const_param_name(ty, module, semantics)
}

fn resolve_const_param_name(
    ty: &ast::Type,
    module: Module,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let ast::Type::PathType(path_type) = ty else {
        return None;
    };
    match semantics.resolve_path(&path_type.path()?)? {
        PathResolution::ConstParam(const_param) => Some(format_item_name(
            const_param.name(semantics.db),
            module,
            semantics.db,
        )),
        _ => None,
    }
}

fn format_node_type_full_name<'db>(
    typ: Type,
    node: &SyntaxNode,
    semantics: &Semantics<'db, RootDatabase>,
) -> Option<String> {
    let module = semantics.scope(node)?.module();
    type_formatter::format_type(&typ, module, semantics.db)
}
