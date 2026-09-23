use ra_ap_hir::Semantics;
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::ast;

pub fn for_ident_pat(
    ident_pat: &ast::IdentPat,
    semantics: &Semantics<RootDatabase>,
) -> Option<bool> {
    semantics.resolve_bind_pat_to_const(ident_pat).map(|_| true)
}
