use super::rust_name_formatter::{
    format_generic_module_def_full_name, format_impl_full_name, format_module_full_name,
};
use ra_ap_hir::{AsAssocItem, AssocItemContainer, ModuleDef, PathResolution, Semantics};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::ast;

pub fn for_path_expr(
    path_expr: &ast::PathExpr,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let path = path_expr.path()?;
    match semantics.resolve_path(&path)? {
        PathResolution::Def(ModuleDef::Const(const_)) => match const_.as_assoc_item(semantics.db) {
            None => format_module_full_name(const_.module(semantics.db), semantics),
            Some(assoc_item) => match assoc_item.container(semantics.db) {
                AssocItemContainer::Impl(impl_) => format_impl_full_name(impl_, semantics),
                AssocItemContainer::Trait(trait_) => {
                    format_generic_module_def_full_name(trait_, semantics)
                }
            },
        },
        PathResolution::Def(ModuleDef::Static(static_)) => {
            format_module_full_name(static_.module(semantics.db), semantics)
        }
        _ => None,
    }
}
