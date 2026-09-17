use super::type_full_names::format_path_resolution_type_full_name;
use ra_ap_hir::{ModuleDef, PathResolution, Semantics};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::{ast, ast::HasTypeBounds};

// TODO: `where Self: Tr` is in essence also a supertrait, but not currently handled.
pub fn for_trait(
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
