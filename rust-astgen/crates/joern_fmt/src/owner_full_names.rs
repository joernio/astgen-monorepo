use super::rust_name_formatter::format_module_full_name;
use ra_ap_hir::{AsAssocItem, ModuleDef, PathResolution, Semantics};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::ast;

pub fn for_path_expr(
    path_expr: &ast::PathExpr,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let path = path_expr.path()?;
    let module = match semantics.resolve_path(&path)? {
        PathResolution::Def(ModuleDef::Const(const_))
            if const_.as_assoc_item(semantics.db).is_none() =>
        {
            const_.module(semantics.db)
        }
        PathResolution::Def(ModuleDef::Static(static_)) => static_.module(semantics.db),
        _ => return None,
    };
    format_module_full_name(module, semantics)
}
