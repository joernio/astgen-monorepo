//! Rust-like name composer/formatter.
//!
//! Essentially, uses `::` for separators, and follows the same
//! naming conventions for generics and traits.

use super::method_full_names::format_function_full_name;
use ra_ap_hir::{GenericDef, InFile, Module, ModuleDef, ModuleSource, Name, Semantics};
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
    let crate_name = super::crate_name(krate, db)?;
    let canonical_path = def.canonical_path(db, krate.edition(db))?;
    Some(format_member_full_name(&crate_name, &canonical_path))
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
