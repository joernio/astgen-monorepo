//! Rust-like name composer/formatter.
//!
//! Essentially, uses `::` for separators, and follows the same
//! naming conventions for generics and traits.

use super::type_formatter;
use ra_ap_hir::{
    AsAssocItem, AssocItemContainer, Crate, EnumVariant, Function, GenericDef, Impl, InFile,
    Module, ModuleDef, ModuleSource, Name, Semantics, Struct, TraitRef, TypeAlias,
};
use ra_ap_ide::RootDatabase;
use ra_ap_syntax::{AstNode, SyntaxNode, ast};

pub(crate) const PATH_SEPARATOR: &str = "::";
pub(crate) const DISAMBIGUATOR_SEPARATOR: &str = "#";

pub(crate) fn format_module_def_full_name(
    def: ModuleDef,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let module = def.module(semantics.db)?;
    let source = module.definition_source(semantics.db);
    if let ModuleSource::BlockExpr(block) = &source.value {
        return block_local_full_name(def, module, source.with_value(block.clone()), semantics);
    }
    format_module_member_full_name(def, module, semantics)
}

fn format_module_member_full_name(
    def: ModuleDef,
    module: Module,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let krate = module.krate(semantics.db);
    let crate_name = crate_name(krate, semantics)?;
    let canonical_path = def.canonical_path(semantics.db, krate.edition(semantics.db))?;
    Some(format_member_full_name(&crate_name, &canonical_path))
}

pub(crate) fn crate_name(krate: Crate, semantics: &Semantics<RootDatabase>) -> Option<String> {
    let display_name = krate.display_name(semantics.db)?.to_string();

    // Build scripts are named `build_script` regardless of the crate they belong to.
    // So, prefix it with the crate name to disambiguate.
    if display_name.starts_with("build_script_")
        && let Some(package_name) = krate.base().env(semantics.db).get("CARGO_PKG_NAME")
    {
        return Some(format!("{}_build_script", package_name.replace("-", "_")));
    }

    Some(display_name)
}

fn block_local_full_name(
    def: ModuleDef,
    module: Module,
    block: InFile<ast::BlockExpr>,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let Some(fn_) = enclosing_fn(block, semantics) else {
        return anonymous_const_local_full_name(def, module, semantics);
    };
    let parent = format_function_full_name(semantics.to_def(&fn_)?, semantics)?;
    let body = fn_.body()?;
    let name = format_item_name(def.name(semantics.db)?, module, semantics);
    let member = match block_local_disambiguator(def, body.syntax(), semantics) {
        Some(disambiguator) => format_disambiguated_full_name(&name, disambiguator),
        None => name,
    };
    Some(format_member_full_name(&parent, &member))
}

fn anonymous_const_local_full_name(
    def: ModuleDef,
    module: Module,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let full_name = format_module_member_full_name(def, module, semantics)?;
    let source = module
        .nearest_non_block_module(semantics.db)
        .definition_source(semantics.db);
    semantics.parse_or_expand(source.file_id);
    let scope = module_source_node(&source.value)?;
    match block_local_disambiguator(def, &scope, semantics) {
        Some(disambiguator) => Some(format_disambiguated_full_name(&full_name, disambiguator)),
        None => Some(full_name),
    }
}

fn module_source_node(source: &ModuleSource) -> Option<SyntaxNode> {
    match source {
        ModuleSource::SourceFile(it) => Some(it.syntax().clone()),
        ModuleSource::Module(it) => Some(it.item_list()?.syntax().clone()),
        ModuleSource::BlockExpr(it) => Some(it.syntax().clone()),
    }
}

fn enclosing_fn(
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

fn block_local_disambiguator(
    def: ModuleDef,
    scope: &SyntaxNode,
    semantics: &Semantics<RootDatabase>,
) -> Option<usize> {
    let name = def.name(semantics.db)?;
    let siblings = block_local_defs_named(&name, scope, semantics);
    let disambiguator = siblings.iter().position(|sibling| *sibling == def)? + 1;
    (siblings.len() > 1).then_some(disambiguator)
}

fn block_local_defs_named(
    name: &Name,
    scope: &SyntaxNode,
    semantics: &Semantics<RootDatabase>,
) -> Vec<ModuleDef> {
    let mut defs = Vec::new();
    collect_block_local_defs(semantics, scope, name, &mut defs);
    defs
}

fn collect_block_local_defs(
    semantics: &Semantics<RootDatabase>,
    node: &SyntaxNode,
    name: &Name,
    out: &mut Vec<ModuleDef>,
) {
    for child in node.children() {
        if let Some(macro_call) = ast::MacroCall::cast(child.clone()) {
            if let Some(expansion) = semantics.expand_macro_call(&macro_call) {
                collect_block_local_defs(semantics, &expansion.value, name, out);
            }
            continue;
        }
        if let Some(item) = ast::Item::cast(child.clone())
            && let Some(def) = block_item_def(semantics, &item)
            && def.name(semantics.db).as_ref() == Some(name)
        {
            out.push(def);
        }
        if !ast::Fn::can_cast(child.kind())
            && !ast::Module::can_cast(child.kind())
            && !ast::AssocItemList::can_cast(child.kind())
        {
            collect_block_local_defs(semantics, &child, name, out);
        }
    }
}

fn block_item_def(semantics: &Semantics<RootDatabase>, item: &ast::Item) -> Option<ModuleDef> {
    match item {
        ast::Item::Enum(it) => Some(semantics.to_def(it)?.into()),
        ast::Item::Fn(it) => Some(semantics.to_def(it)?.into()),
        ast::Item::Struct(it) => Some(semantics.to_def(it)?.into()),
        _ => None,
    }
}

pub(crate) fn format_item_name(
    name: ra_ap_hir::Name,
    module: Module,
    semantics: &Semantics<RootDatabase>,
) -> String {
    name.display(
        semantics.db,
        module.krate(semantics.db).edition(semantics.db),
    )
    .to_string()
}

fn format_member_full_name(parent: &str, member: &str) -> String {
    format!("{parent}{PATH_SEPARATOR}{member}")
}

fn format_disambiguated_full_name(name: &str, disambiguator: usize) -> String {
    format!("{name}{DISAMBIGUATOR_SEPARATOR}{disambiguator}")
}

fn format_trait_impl_full_name(impl_ty: &str, trait_name: &str) -> String {
    format!("<{impl_ty} as {trait_name}>")
}

pub(crate) fn format_name_with_generic_args(base: String, generic_args: Vec<String>) -> String {
    if generic_args.is_empty() {
        base
    } else {
        format!("{base}<{}>", generic_args.join(", "))
    }
}

fn format_generic_args_for_def(
    generic_def: GenericDef,
    module: Module,
    semantics: &Semantics<RootDatabase>,
) -> Vec<String> {
    let mut args = Vec::new();

    for param in generic_def.type_or_const_params(semantics.db) {
        if let Some(type_param) = param.as_type_param(semantics.db) {
            if type_param.is_implicit(semantics.db) {
                continue;
            }

            let name = format_item_name(type_param.name(semantics.db), module, semantics);
            args.push(name);
        } else if let Some(const_param) = param.as_const_param(semantics.db) {
            args.push(format_item_name(
                const_param.name(semantics.db),
                module,
                semantics,
            ));
        }
    }

    args
}

pub(super) fn format_impl_full_name(
    impl_: Impl,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let self_ty_name =
        type_formatter::format_impl_self_ty(impl_, impl_.module(semantics.db), semantics)?;
    let Some(trait_ref) = impl_.trait_ref(semantics.db) else {
        return Some(self_ty_name);
    };
    let trait_name = format_trait_ref_full_name(trait_ref, impl_.module(semantics.db), semantics)?;
    Some(format_trait_impl_full_name(&self_ty_name, &trait_name))
}

pub(crate) fn format_function_full_name(
    function: Function,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let Some(assoc_item) = function.as_assoc_item(semantics.db) else {
        return format_generic_module_def_full_name(function, semantics);
    };

    let method_name = format_generic_item_name(
        function.name(semantics.db),
        GenericDef::from(function),
        function.module(semantics.db),
        semantics,
    );
    match assoc_item.container(semantics.db) {
        AssocItemContainer::Impl(impl_) => Some(format_member_full_name(
            &format_impl_full_name(impl_, semantics)?,
            &method_name,
        )),
        AssocItemContainer::Trait(trait_) => {
            let trait_name = format_generic_module_def_full_name(trait_, semantics)?;
            Some(format_member_full_name(&trait_name, &method_name))
        }
    }
}

pub(super) fn format_generic_module_def_full_name<D>(
    def: D,
    semantics: &Semantics<RootDatabase>,
) -> Option<String>
where
    D: Into<ModuleDef> + Into<GenericDef> + Copy,
{
    let generic_def: GenericDef = def.into();
    let module_def: ModuleDef = def.into();
    let base = format_module_def_full_name(module_def, semantics)?;
    Some(format_generic_name(
        base,
        generic_def,
        generic_def.module(semantics.db),
        semantics,
    ))
}

fn format_generic_item_name(
    name: Name,
    generic_def: GenericDef,
    module: Module,
    semantics: &Semantics<RootDatabase>,
) -> String {
    let base = format_item_name(name, module, semantics);
    format_generic_name(base, generic_def, module, semantics)
}

fn format_generic_name(
    base: String,
    generic_def: GenericDef,
    module: Module,
    semantics: &Semantics<RootDatabase>,
) -> String {
    let generic_args = format_generic_args_for_def(generic_def, module, semantics);
    format_name_with_generic_args(base, generic_args)
}

pub(crate) fn format_tuple_struct_ctor_full_name(
    struct_: Struct,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    format_generic_module_def_full_name(struct_, semantics)
}

pub(crate) fn format_enum_variant_full_name(
    enum_variant: EnumVariant,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let enum_ = enum_variant.parent_enum(semantics.db);
    let enum_name = format_generic_module_def_full_name(enum_, semantics)?;
    let variant_name = format_item_name(
        enum_variant.name(semantics.db),
        enum_variant.module(semantics.db),
        semantics,
    );
    Some(format_member_full_name(&enum_name, &variant_name))
}

fn format_trait_ref_full_name(
    trait_ref: TraitRef,
    module: Module,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let trait_ = trait_ref.trait_();
    let base = format_module_def_full_name(ModuleDef::from(trait_), semantics)?;
    let arg_count = trait_.type_or_const_param_count(semantics.db, false);
    // Self is 0, type args are 1+
    let generic_args = (1..=arg_count)
        .map(|idx| {
            let arg = trait_ref.get_type_argument(idx)?;
            type_formatter::format_type(&arg.to_type(semantics.db), module, semantics)
        })
        .collect::<Option<Vec<_>>>()?;

    Some(format_name_with_generic_args(base, generic_args))
}

pub(super) fn format_type_alias_full_name(
    type_alias: TypeAlias,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let Some(AssocItemContainer::Trait(trait_)) = type_alias
        .as_assoc_item(semantics.db)
        .map(|item| item.container(semantics.db))
    else {
        return format_module_def_full_name(ModuleDef::from(type_alias), semantics);
    };
    let trait_name = format_module_def_full_name(ModuleDef::from(trait_), semantics)?;
    let name = format_item_name(
        type_alias.name(semantics.db),
        type_alias.module(semantics.db),
        semantics,
    );
    Some(format_member_full_name(&trait_name, &name))
}
