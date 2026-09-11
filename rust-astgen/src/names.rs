pub(crate) mod method_full_names;
pub(crate) mod rust_name_formatter;
mod trait_full_names;
pub(crate) mod type_formatter;
mod type_full_names;

use ra_ap_hir::{EnumVariant, Function, Semantics, Struct};
use ra_ap_ide::RootDatabase;

pub(crate) use method_full_names::{has_self_receiver_for_node, method_full_name_for_node};
pub(crate) use rust_name_formatter::crate_name;
pub(crate) use trait_full_names::{implemented_traits_for_node, supertraits_for_node};
pub(crate) use type_full_names::type_full_name_for_node;

// TODO(xavierp): Only here so we don't touch rust_ast_function_fullnames yet. Remove later.
pub fn format_function_full_name(function: Function, db: &RootDatabase) -> Option<String> {
    rust_name_formatter::format_function_full_name(function, &Semantics::new(db))
}

// TODO(xavierp): Only here so we don't touch rust_ast_function_fullnames yet. Remove later.
pub fn format_tuple_struct_ctor_full_name(struct_: Struct, db: &RootDatabase) -> Option<String> {
    rust_name_formatter::format_tuple_struct_ctor_full_name(struct_, &Semantics::new(db))
}

// TODO(xavierp): Only here so we don't touch rust_ast_function_fullnames yet. Remove later.
pub fn format_enum_variant_full_name(
    enum_variant: EnumVariant,
    db: &RootDatabase,
) -> Option<String> {
    rust_name_formatter::format_enum_variant_full_name(enum_variant, &Semantics::new(db))
}
