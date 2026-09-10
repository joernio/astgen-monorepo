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

pub(crate) fn format_module_def_full_name(def: ModuleDef, db: &RootDatabase) -> Option<String> {
    let module = def.module(db)?;
    let source = module.definition_source(db);
    if let ModuleSource::BlockExpr(block) = &source.value {
        return block_local_full_name(def, module, source.with_value(block.clone()), db);
    }
    format_module_member_full_name(def, module, db)
}

fn format_module_member_full_name(
    def: ModuleDef,
    module: Module,
    db: &RootDatabase,
) -> Option<String> {
    let krate = module.krate(db);
    let crate_name = crate_name(krate, db)?;
    let canonical_path = def.canonical_path(db, krate.edition(db))?;
    Some(format_member_full_name(&crate_name, &canonical_path))
}

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

fn block_local_full_name(
    def: ModuleDef,
    module: Module,
    block: InFile<ast::BlockExpr>,
    db: &RootDatabase,
) -> Option<String> {
    let semantics = Semantics::new(db);
    let Some(fn_) = enclosing_fn(block, &semantics) else {
        return anonymous_const_local_full_name(def, module, &semantics);
    };
    let parent = format_function_full_name(semantics.to_def(&fn_)?, db)?;
    let body = fn_.body()?;
    let name = format_item_name(def.name(db)?, module, db);
    let member = match block_local_disambiguator(def, body.syntax(), &semantics) {
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
    let full_name = format_module_member_full_name(def, module, semantics.db)?;
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

pub(crate) fn format_item_name(name: ra_ap_hir::Name, module: Module, db: &RootDatabase) -> String {
    name.display(db, module.krate(db).edition(db)).to_string()
}

pub(crate) fn format_member_full_name(parent: &str, member: &str) -> String {
    format!("{parent}{PATH_SEPARATOR}{member}")
}

pub(crate) fn format_disambiguated_full_name(name: &str, disambiguator: usize) -> String {
    format!("{name}{DISAMBIGUATOR_SEPARATOR}{disambiguator}")
}

pub(crate) fn format_trait_impl_full_name(impl_ty: &str, trait_name: &str) -> String {
    format!("<{impl_ty} as {trait_name}>")
}

pub(crate) fn format_name_with_generic_args(base: String, generic_args: Vec<String>) -> String {
    if generic_args.is_empty() {
        base
    } else {
        format!("{base}<{}>", generic_args.join(", "))
    }
}

pub(super) fn format_generic_args_for_def(
    generic_def: GenericDef,
    module: Module,
    db: &RootDatabase,
) -> Vec<String> {
    let mut args = Vec::new();

    for param in generic_def.type_or_const_params(db) {
        if let Some(type_param) = param.as_type_param(db) {
            if type_param.is_implicit(db) {
                continue;
            }

            let name = format_item_name(type_param.name(db), module, db);
            args.push(name);
        } else if let Some(const_param) = param.as_const_param(db) {
            args.push(format_item_name(const_param.name(db), module, db));
        }
    }

    args
}

pub(super) fn format_impl_full_name(impl_: Impl, db: &RootDatabase) -> Option<String> {
    let self_ty_name = type_formatter::format_impl_self_ty(impl_, impl_.module(db), db)?;
    let Some(trait_ref) = impl_.trait_ref(db) else {
        return Some(self_ty_name);
    };
    let trait_name = format_trait_ref_full_name(trait_ref, impl_.module(db), db)?;
    Some(format_trait_impl_full_name(&self_ty_name, &trait_name))
}

pub fn format_function_full_name(function: Function, db: &RootDatabase) -> Option<String> {
    let Some(assoc_item) = function.as_assoc_item(db) else {
        return format_generic_module_def_full_name(
            ModuleDef::from(function),
            GenericDef::from(function),
            function.module(db),
            db,
        );
    };

    let method_name = format_generic_item_name(
        function.name(db),
        GenericDef::from(function),
        function.module(db),
        db,
    );
    match assoc_item.container(db) {
        AssocItemContainer::Impl(impl_) => Some(format_member_full_name(
            &format_impl_full_name(impl_, db)?,
            &method_name,
        )),
        AssocItemContainer::Trait(trait_) => {
            let trait_name = format_generic_module_def_full_name(
                ModuleDef::from(trait_),
                GenericDef::from(trait_),
                trait_.module(db),
                db,
            )?;
            Some(format_member_full_name(&trait_name, &method_name))
        }
    }
}

pub(super) fn format_generic_module_def_full_name(
    def: ModuleDef,
    generic_def: GenericDef,
    module: Module,
    db: &RootDatabase,
) -> Option<String> {
    let base = format_module_def_full_name(def, db)?;
    Some(format_generic_name(base, generic_def, module, db))
}

fn format_generic_item_name(
    name: Name,
    generic_def: GenericDef,
    module: Module,
    db: &RootDatabase,
) -> String {
    let base = format_item_name(name, module, db);
    format_generic_name(base, generic_def, module, db)
}

fn format_generic_name(
    base: String,
    generic_def: GenericDef,
    module: Module,
    db: &RootDatabase,
) -> String {
    let generic_args = format_generic_args_for_def(generic_def, module, db);
    format_name_with_generic_args(base, generic_args)
}

pub fn format_tuple_struct_ctor_full_name(struct_: Struct, db: &RootDatabase) -> Option<String> {
    format_generic_module_def_full_name(
        ModuleDef::from(struct_),
        GenericDef::from(struct_),
        struct_.module(db),
        db,
    )
}

pub fn format_enum_variant_full_name(
    enum_variant: EnumVariant,
    db: &RootDatabase,
) -> Option<String> {
    let enum_ = enum_variant.parent_enum(db);
    let enum_name = format_generic_module_def_full_name(
        ModuleDef::from(enum_),
        GenericDef::from(enum_),
        enum_.module(db),
        db,
    )?;
    let variant_name = format_item_name(enum_variant.name(db), enum_variant.module(db), db);
    Some(format_member_full_name(&enum_name, &variant_name))
}

fn format_trait_ref_full_name<'db>(
    trait_ref: TraitRef<'db>,
    module: Module,
    db: &'db RootDatabase,
) -> Option<String> {
    let trait_ = trait_ref.trait_();
    let base = format_module_def_full_name(ModuleDef::from(trait_), db)?;
    let arg_count = trait_.type_or_const_param_count(db, false);
    // Self is 0, type args are 1+
    let generic_args = (1..=arg_count)
        .map(|idx| {
            let arg = trait_ref.get_type_argument(idx)?;
            type_formatter::format_type(&arg.to_type(db), module, db)
        })
        .collect::<Option<Vec<_>>>()?;

    Some(format_name_with_generic_args(base, generic_args))
}

pub(super) fn format_type_alias_full_name(
    type_alias: TypeAlias,
    db: &RootDatabase,
) -> Option<String> {
    let Some(AssocItemContainer::Trait(trait_)) =
        type_alias.as_assoc_item(db).map(|item| item.container(db))
    else {
        return format_module_def_full_name(ModuleDef::from(type_alias), db);
    };
    let trait_name = format_module_def_full_name(ModuleDef::from(trait_), db)?;
    let name = format_item_name(type_alias.name(db), type_alias.module(db), db);
    Some(format_member_full_name(&trait_name, &name))
}
