use super::rust_name_formatter::format_generic_module_def_full_name;
use ra_ap_hir::{Adt, ModuleDef, PathResolution, Semantics, StructKind};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::ast;

pub fn for_path_expr(
    path_expr: &ast::PathExpr,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let path = path_expr.path()?;
    match semantics.resolve_path(&path)? {
        PathResolution::Def(ModuleDef::Adt(Adt::Struct(struct_)))
            if struct_.kind(semantics.db) == StructKind::Unit =>
        {
            format_generic_module_def_full_name(struct_, semantics)
        }
        _ => None,
    }
}
