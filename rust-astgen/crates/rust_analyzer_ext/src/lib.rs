use ra_ap_hir::{
    Crate, HirFileId, Impl, InFile, Semantics, Type, TypeAlias, db::DefDatabase,
    db::ExpandDatabase, db::HirDatabase, next_solver::GenericArgs,
};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::{AstNode, SyntaxNode, ast};
use ra_ap_vfs::FileId;

pub fn is_include_target(
    file_id: FileId,
    target_crate: Crate,
    semantics: &Semantics<RootDatabase>,
) -> bool {
    semantics
        .db
        .include_macro_invoc(target_crate.base())
        .iter()
        .any(|(_, included_file_id)| included_file_id.file_id(semantics.db) == file_id)
}

pub fn crate_for_file(
    syntax_tree: &SyntaxNode,
    semantics: &Semantics<RootDatabase>,
) -> Option<Crate> {
    semantics
        .scope(syntax_tree)?
        .module()
        .krate(semantics.db)
        .into()
}

pub fn expansion_has_no_errors(
    hir_file_id: HirFileId,
    semantics: &Semantics<RootDatabase>,
) -> bool {
    let Some(macro_file) = hir_file_id.macro_file() else {
        return true;
    };

    let (parse, _) = &semantics.db.parse_macro_expansion(macro_file).value;
    parse.errors().is_empty()
}

pub fn enclosing_fn(
    block: InFile<ast::BlockExpr>,
    semantics: &Semantics<RootDatabase>,
) -> Option<ast::Fn> {
    semantics
        .ancestors_with_macros_file(block.with_value(block.value.syntax().clone()))
        .find_map(|ancestor| {
            let fn_ = ast::Fn::cast(ancestor.value)?;
            semantics.parse_or_expand(ancestor.file_id);
            Some(fn_)
        })
}

pub fn normalize_assoc_type<'db>(
    path: &ast::Path,
    assoc_type: TypeAlias,
    semantics: &Semantics<'db, RootDatabase>,
) -> Option<Type<'db>> {
    let segment = path.qualifier().and_then(|qualifier| qualifier.segment())?;
    let anchor = segment.type_anchor().and_then(|anchor| anchor.ty())?;
    let anchor = semantics.resolve_type(&anchor)?;
    let normalized = anchor.normalize_trait_assoc_type(semantics.db, &[], assoc_type)?;
    normalized
        .as_associated_type_parent_trait(semantics.db)
        .is_none()
        .then_some(normalized)
}

pub fn impl_trait_args<'db>(
    impl_: Impl,
    semantics: &Semantics<'db, RootDatabase>,
) -> Option<GenericArgs<'db>> {
    let trait_ref = semantics.db.impl_trait(impl_.try_into().ok()?)?;
    Some(trait_ref.skip_binder().args)
}
