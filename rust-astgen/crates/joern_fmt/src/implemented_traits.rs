use ra_ap_hir::{Adt, Impl, Semantics};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::ast;
use crate::rust_name_formatter::format_impl_trait;

// NB: This is approximate (cf. all_for_type's doc). In particular, `impl<T> Trait for T` are
// excluded, as well as compiler marker traits (Send, Sync, Unpin, UnwindSafe, etc.), and
// negative impls (no use for them).
pub fn for_struct(
    struct_: &ast::Struct,
    semantics: &Semantics<RootDatabase>,
) -> Option<Vec<String>> {
    let adt = Adt::from(semantics.to_def(struct_)?);
    implemented_traits(adt, semantics)
}

pub fn for_enum(
    enum_: &ast::Enum,
    semantics: &Semantics<RootDatabase>,
) -> Option<Vec<String>> {
    let adt = Adt::from(semantics.to_def(enum_)?);
    implemented_traits(adt, semantics)
}

fn implemented_traits(adt: Adt, semantics: &Semantics<RootDatabase>) -> Option<Vec<String>> {
    let module = adt.module(semantics.db);

    let mut names: Vec<String> = Impl::all_for_type(semantics.db, adt.ty(semantics.db))
        .into_iter()
        .filter(|impl_| !impl_.is_negative(semantics.db))
        .filter_map(|impl_| format_impl_trait(impl_, module, semantics))
        .collect();

    names.sort();
    names.dedup();
    if names.is_empty() { None } else { Some(names) }
}
