//! Where we build `implementedTraits` for struct/enum declarations and
//! `supertraits` for trait declarations.

use super::{
    rust_name_formatter::format_impl_trait, type_full_names::format_path_resolution_type_full_name,
};
use ra_ap_hir::{Adt, Impl, ModuleDef, PathResolution, Semantics};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::{ast, ast::HasTypeBounds};

// NB: This is approximate (cf. all_for_type's doc). In particular, `impl<T> Trait for T` are
// excluded, as well as compiler marker traits (Send, Sync, Unpin, UnwindSafe, etc.), and
// negative impls (no use for them).
pub fn implemented_traits_for_struct(
    struct_: &ast::Struct,
    semantics: &Semantics<RootDatabase>,
) -> Option<Vec<String>> {
    let adt = Adt::from(semantics.to_def(struct_)?);
    implemented_traits(adt, semantics)
}

pub fn implemented_traits_for_enum(
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

// TODO: `where Self: Tr` is in essence also a supertrait, but not currently handled.
pub fn supertraits(
    trait_decl: &ast::Trait,
    semantics: &Semantics<RootDatabase>,
) -> Option<Vec<String>> {
    let mut names: Vec<String> = trait_decl
        .type_bound_list()?
        .bounds()
        .filter_map(|bound| supertrait_full_name(&bound, semantics))
        .collect();

    names.sort();
    names.dedup();
    if names.is_empty() { None } else { Some(names) }
}

fn supertrait_full_name(
    bound: &ast::TypeBound,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let ast::Type::PathType(path_type) = bound.ty()? else {
        return None;
    };
    let path = path_type.path()?;
    let resolution = semantics.resolve_path(&path)?;
    match resolution {
        PathResolution::Def(ModuleDef::Trait(_)) => {
            format_path_resolution_type_full_name(resolution, &path, semantics)
        }
        _ => None,
    }
}
