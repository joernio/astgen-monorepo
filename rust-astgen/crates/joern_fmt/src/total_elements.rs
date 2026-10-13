use ra_ap_hir::Semantics;
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::ast;

pub fn for_tuple_pat(
    tuple_pat: &ast::TuplePat,
    semantics: &Semantics<RootDatabase>,
) -> Option<usize> {
    if !has_rest_pat(tuple_pat.fields()) {
        return None;
    }
    let typ = semantics
        .type_of_pat(&tuple_pat.to_owned().into())?
        .original;
    typ.is_tuple().then(|| typ.tuple_fields(semantics.db).len())
}

fn has_rest_pat(mut pats: impl Iterator<Item = ast::Pat>) -> bool {
    pats.find(|it| matches!(it, ast::Pat::RestPat(_))).is_some()
}
