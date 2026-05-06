//! Rust-like name composer/formatter.
//!
//! Essentially, uses `::` for separators, and follows the same
//! naming conventions for generics and traits.

use ra_ap_hir::{Module, ModuleDef};
use ra_ap_ide::RootDatabase;

pub(crate) const PATH_SEPARATOR: &str = "::";

pub(crate) fn format_module_def_full_name(def: ModuleDef, db: &RootDatabase) -> Option<String> {
    let module = def.module(db)?;
    let krate = module.krate(db);
    let crate_name = krate.display_name(db)?.to_string();
    let canonical_path = def.canonical_path(db, krate.edition(db))?;
    Some(format_member_full_name(&crate_name, &canonical_path))
}

pub(crate) fn format_item_name(name: ra_ap_hir::Name, module: Module, db: &RootDatabase) -> String {
    name.display(db, module.krate(db).edition(db)).to_string()
}

pub(crate) fn format_member_full_name(parent: &str, member: &str) -> String {
    format!("{parent}{PATH_SEPARATOR}{member}")
}

pub(crate) fn format_trait_impl_member_full_name(
    impl_ty: &str,
    trait_name: &str,
    member_name: &str,
) -> String {
    format!("<{impl_ty} as {trait_name}>{PATH_SEPARATOR}{member_name}")
}

pub(crate) fn format_name_with_generic_args(base: String, generic_args: Vec<String>) -> String {
    if generic_args.is_empty() {
        base
    } else {
        format!("{base}<{}>", generic_args.join(", "))
    }
}
