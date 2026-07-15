//! Where we build `implementedTraits` for struct declarations and
//! `supertraits` for trait declarations.

use super::{
    rust_name_formatter::{format_module_def_full_name, format_name_with_generic_args},
    type_formatter,
    type_full_names::format_path_resolution_type_full_name,
};
use ra_ap_hir::{Adt, GenericDef, Impl, Module, ModuleDef, PathResolution, Semantics, TraitRef};
use ra_ap_ide::RootDatabase;
use ra_ap_syntax::{AstNode, SyntaxNode, ast, ast::HasTypeBounds};

// NB: This is approximate (cf. all_for_type's doc). In particular, `impl<T> Trait for T` are
// excluded, as well as compiler marker traits (Send, Sync, Unpin, UnwindSafe, etc.), and
// negative impls (no use for them).
pub(crate) fn implemented_traits_for_node(
    node: &SyntaxNode,
    semantics: &Semantics<RootDatabase>,
) -> Option<Vec<String>> {
    let struct_ = ast::Struct::cast(node.clone())?;
    let adt = Adt::from(semantics.to_def(&struct_)?);
    let module = semantics.scope(node)?.module();

    let mut names: Vec<String> = Impl::all_for_type(semantics.db, adt.ty(semantics.db))
        .into_iter()
        .filter(|impl_| !impl_.is_negative(semantics.db))
        .filter_map(|impl_| impl_.trait_ref(semantics.db))
        .filter_map(|trait_ref| format_trait_ref_full_name(&trait_ref, module, semantics))
        .collect();

    names.sort();
    names.dedup();
    if names.is_empty() { None } else { Some(names) }
}

// TODO: `where Self: Tr` is in essence also a supertrait, but not currently handled.
pub(crate) fn supertraits_for_node(
    node: &SyntaxNode,
    semantics: &Semantics<RootDatabase>,
) -> Option<Vec<String>> {
    let trait_decl = ast::Trait::cast(node.clone())?;
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

fn format_trait_ref_full_name(
    trait_ref: &TraitRef,
    module: Module,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let trait_ = trait_ref.trait_();
    let base = format_module_def_full_name(ModuleDef::from(trait_), semantics.db)?;

    // Parameter 0 is the `Self` (trait) type. Actual parameters start at 1.
    // E.g. trait Tr<'a, A, const N: usize> has parameters: Self, 'a, A, N.
    let param_count = GenericDef::from(trait_).params(semantics.db).len();

    // get_type_argument already skips lifetime and const parameters.
    let args = (1..=param_count)
        .filter_map(|idx| trait_ref.get_type_argument(idx))
        .map(|arg| type_formatter::format_type(&arg.to_type(semantics.db), module, semantics.db))
        .collect::<Option<Vec<_>>>()?;

    Some(format_name_with_generic_args(base, args))
}
