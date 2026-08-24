pub(crate) mod method_full_names;
mod rust_name_formatter;
mod trait_full_names;
pub(crate) mod type_formatter;
mod type_full_names;

pub(crate) use method_full_names::{
    format_enum_variant_full_name, format_function_full_name, format_tuple_struct_ctor_full_name,
    method_full_name_for_node,
};
pub(crate) use trait_full_names::{implemented_traits_for_node, supertraits_for_node};
pub(crate) use type_full_names::type_full_name_for_node;

use ra_ap_hir::Crate;
use ra_ap_ide::RootDatabase;

pub(crate) fn crate_name(krate: Crate, db: &RootDatabase) -> Option<String> {
    let display_name = krate.display_name(db)?.to_string();

    // Build scripts are named `build_script` regardless of the crate they belong to.
    // So, prefix it with the crate name to disambiguate.
    if display_name.starts_with("build_script_")
        && let Some(package_name) = krate.base().env(db).get("CARGO_PKG_NAME")
    {
        return Some(format!("{}_build_script", package_name.replace("-", "_")));
    }

    Some(display_name)
}
