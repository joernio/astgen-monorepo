use crate::names::{
    format_enum_variant_full_name, format_function_full_name, format_tuple_struct_ctor_full_name,
};
use crate::{cargo, config};
use anyhow::Context;
use ra_ap_hir::{
    Adt, AsAssocItem, AssocItem, AssocItemContainer, Crate, Enum, EnumVariant, Function,
    HasVisibility, Impl, Module, ModuleDef, StructKind, Trait, attach_db,
};
use ra_ap_ide::RootDatabase;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::io::{self, Write};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FunctionFullNameEntry {
    #[serde(rename = "methodFullName")]
    pub method_full_name: String,
    #[serde(rename = "hasSelfReceiver")]
    pub has_self_receiver: bool,
    #[serde(rename = "isTraitImpl")]
    pub is_trait_impl: bool,
    #[serde(rename = "isTraitMethodDef")]
    pub is_trait_method_def: bool,
    #[serde(rename = "isNightlyOnly")]
    pub is_nightly_only: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionFullNamesOutput {
    pub functions: Vec<FunctionFullNameEntry>,
}

pub fn run(config: &config::RustAstGenConfig) -> anyhow::Result<()> {
    let (root_db, _vfs) = cargo::load_workspace(config)?;

    let output = attach_db(&root_db, || collect_dependency_full_names(&root_db));

    let json = serde_json::to_string_pretty(&output)?;
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(json.as_bytes())
        .context("failed to write function fullnames JSON to stdout")?;
    stdout
        .write_all(b"\n")
        .context("failed to write trailing newline to stdout")?;

    Ok(())
}

fn collect_dependency_full_names(db: &RootDatabase) -> FunctionFullNamesOutput {
    let mut entries = BTreeMap::new();
    let workspace_roots = workspace_root_modules(db);

    for krate in dependency_crates(db) {
        for module in krate.modules(db) {
            collect_from_module(module, db, &workspace_roots, &mut entries);
        }
    }

    FunctionFullNamesOutput {
        functions: entries.into_values().collect(),
    }
}

fn workspace_root_modules(db: &RootDatabase) -> Vec<Module> {
    Crate::all(db)
        .into_iter()
        .filter(|krate| krate.origin(db).is_local())
        .map(|krate| krate.root_module(db))
        .collect()
}

fn is_available_from_workspace<T: HasVisibility>(
    item: &T,
    db: &RootDatabase,
    workspace_roots: &[Module],
) -> bool {
    workspace_roots
        .iter()
        .any(|module| item.is_visible_from(db, *module))
}

fn is_function_available_from_workspace(
    function: Function,
    db: &RootDatabase,
    workspace_roots: &[Module],
) -> bool {
    let Some(AssocItemContainer::Impl(impl_)) =
        function.as_assoc_item(db).map(|assoc| assoc.container(db))
    else {
        // Free functions and trait method definitions use their own visibility.
        return is_available_from_workspace(&function, db, workspace_roots);
    };

    let Some(trait_ref) = impl_.trait_ref(db) else {
        // Inherent impl methods can legitimately be private, so the function's own
        // visibility is the right gate.
        return is_available_from_workspace(&function, db, workspace_roots);
    };

    // Trait impl members are public via the trait, so `Function::is_visible_from`
    // under-reports them. A trait impl is reachable from the workspace iff both the
    // trait and the implementing (self) type are reachable.
    is_available_from_workspace(&trait_ref.trait_(), db, workspace_roots)
        && is_self_ty_available_from_workspace(impl_, db, workspace_roots)
}

fn is_self_ty_available_from_workspace(
    impl_: Impl,
    db: &RootDatabase,
    workspace_roots: &[Module],
) -> bool {
    match impl_.self_ty(db).as_adt_with_args() {
        // Primitives, arrays, slices, references, tuples, etc. carry no visibility
        // restriction of their own and are always nameable.
        None => true,
        Some((adt, _)) => is_available_from_workspace(&adt, db, workspace_roots),
    }
}

fn collect_from_module(
    module: Module,
    db: &RootDatabase,
    workspace_roots: &[Module],
    entries: &mut BTreeMap<String, FunctionFullNameEntry>,
) {
    for def in module.declarations(db) {
        match def {
            ModuleDef::Function(function) => {
                insert_function(function, db, workspace_roots, entries);
            }
            ModuleDef::Adt(Adt::Struct(struct_)) => {
                insert_tuple_struct_ctor(struct_, db, workspace_roots, entries);
            }
            ModuleDef::Adt(Adt::Enum(enum_)) => {
                collect_enum_variants(enum_, db, workspace_roots, entries);
            }
            ModuleDef::Trait(trait_) => {
                collect_trait_functions(trait_, db, workspace_roots, entries);
            }
            _ => {}
        }
    }

    for impl_ in module.impl_defs(db) {
        for item in impl_.items(db) {
            if let AssocItem::Function(function) = item {
                insert_function(function, db, workspace_roots, entries);
            }
        }
    }
}

fn collect_enum_variants(
    enum_: Enum,
    db: &RootDatabase,
    workspace_roots: &[Module],
    entries: &mut BTreeMap<String, FunctionFullNameEntry>,
) {
    for enum_variant in enum_.variants(db) {
        insert_enum_variant_ctor(enum_variant, db, workspace_roots, entries);
    }
}

fn collect_trait_functions(
    trait_: Trait,
    db: &RootDatabase,
    workspace_roots: &[Module],
    entries: &mut BTreeMap<String, FunctionFullNameEntry>,
) {
    for item in trait_.items(db) {
        if let AssocItem::Function(function) = item {
            insert_function(function, db, workspace_roots, entries);
        }
    }
}

fn dependency_crates(db: &RootDatabase) -> Vec<Crate> {
    let mut deps = Vec::new();
    let mut seen = HashSet::new();

    for krate in Crate::all(db) {
        if !krate.origin(db).is_local() {
            continue;
        }
        collect_transitive_dependency_crates(krate, db, &mut seen, &mut deps);
    }

    if deps.is_empty() {
        return Crate::all(db)
            .into_iter()
            .filter(|krate| !krate.origin(db).is_local())
            .collect();
    }

    deps
}

fn collect_transitive_dependency_crates(
    krate: Crate,
    db: &RootDatabase,
    seen: &mut HashSet<Crate>,
    deps: &mut Vec<Crate>,
) {
    for dep in krate.dependencies(db) {
        let dep_krate = dep.krate;
        if dep_krate.origin(db).is_local() {
            continue;
        }
        if seen.insert(dep_krate) {
            deps.push(dep_krate);
            collect_transitive_dependency_crates(dep_krate, db, seen, deps);
        }
    }
}

fn insert_function(
    function: Function,
    db: &RootDatabase,
    workspace_roots: &[Module],
    entries: &mut BTreeMap<String, FunctionFullNameEntry>,
) {
    if !is_function_available_from_workspace(function, db, workspace_roots) {
        return;
    }

    let Some(method_full_name) = format_function_full_name(function, db) else {
        return;
    };

    let (is_trait_impl, is_trait_method_def) = match function.as_assoc_item(db) {
        Some(assoc_item) => trait_flags(assoc_item, db),
        None => (false, false),
    };

    entries.insert(
        method_full_name.clone(),
        FunctionFullNameEntry {
            method_full_name,
            has_self_receiver: function.has_self_param(db),
            is_trait_impl,
            is_trait_method_def,
            is_nightly_only: function.is_unstable(db),
        },
    );
}

fn insert_tuple_struct_ctor(
    struct_: ra_ap_hir::Struct,
    db: &RootDatabase,
    workspace_roots: &[Module],
    entries: &mut BTreeMap<String, FunctionFullNameEntry>,
) {
    if !is_available_from_workspace(&struct_, db, workspace_roots) {
        return;
    }

    if struct_.kind(db) != StructKind::Tuple {
        return;
    }

    let Some(method_full_name) = format_tuple_struct_ctor_full_name(struct_, db) else {
        return;
    };

    entries.insert(
        method_full_name.clone(),
        FunctionFullNameEntry {
            method_full_name,
            has_self_receiver: false,
            is_trait_impl: false,
            is_trait_method_def: false,
            is_nightly_only: struct_.is_unstable(db),
        },
    );
}

fn insert_enum_variant_ctor(
    enum_variant: EnumVariant,
    db: &RootDatabase,
    workspace_roots: &[Module],
    entries: &mut BTreeMap<String, FunctionFullNameEntry>,
) {
    if !is_available_from_workspace(&enum_variant, db, workspace_roots) {
        return;
    }

    match enum_variant.kind(db) {
        StructKind::Tuple | StructKind::Unit => {}
        StructKind::Record => return,
    }

    let Some(method_full_name) = format_enum_variant_full_name(enum_variant, db) else {
        return;
    };

    entries.insert(
        method_full_name.clone(),
        FunctionFullNameEntry {
            method_full_name,
            has_self_receiver: false,
            is_trait_impl: false,
            is_trait_method_def: false,
            is_nightly_only: enum_variant.is_unstable(db),
        },
    );
}

fn trait_flags(assoc_item: AssocItem, db: &RootDatabase) -> (bool, bool) {
    match assoc_item.container(db) {
        AssocItemContainer::Trait(_) => (false, true),
        AssocItemContainer::Impl(impl_) => (impl_.trait_ref(db).is_some(), false),
    }
}
