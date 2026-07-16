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
use std::collections::HashSet;
use std::io::{self, Write};
use std::rc::Rc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

    attach_db(&root_db, || {
        let mut stdout = io::stdout().lock();
        write_function_fullnames_by_crate(&mut stdout, &root_db)
    })
}

pub fn dependency_full_names<'db>(
    db: &'db RootDatabase,
) -> impl Iterator<Item = FunctionFullNameEntry> + 'db {
    let workspace_roots = workspace_root_modules_rc(db);
    unique_by_method_full_name(dependency_crates(db).into_iter().flat_map({
        let workspace_roots = Rc::clone(&workspace_roots);
        move |krate| {
            let workspace_roots = Rc::clone(&workspace_roots);
            modules_in_crate(db, krate).flat_map(move |module| {
                module_full_names(db, module, Rc::clone(&workspace_roots))
            })
        }
    }))
}

pub fn load_sysroot_workspace(
    input_dir: std::path::PathBuf,
) -> anyhow::Result<RootDatabase> {
    let input_dir = input_dir.canonicalize()?;
    let config = config::RustAstGenConfig::new(input_dir.clone(), input_dir, 1, true, false)?;
    Ok(cargo::load_workspace(&config)?.0)
}

pub fn workspace_root_modules_rc(db: &RootDatabase) -> Rc<[Module]> {
    workspace_root_modules(db).into()
}

pub fn dependency_crates(db: &RootDatabase) -> Vec<Crate> {
    let mut deps = Vec::new();
    let mut seen = HashSet::new();

    for krate in Crate::all(db) {
        if !krate.origin(db).is_local() {
            continue;
        }
        collect_transitive_dependency_crates(krate, db, &mut seen, &mut deps);
    }

    deps
}

pub fn dependency_crate_named(db: &RootDatabase, name: &str) -> Option<Crate> {
    dependency_crates(db).into_iter().find(|krate| {
        krate
            .display_name(db)
            .is_some_and(|crate_name| crate_name.as_str() == name)
    })
}

pub fn modules_in_crate(db: &RootDatabase, krate: Crate) -> impl Iterator<Item = Module> + '_ {
    krate.modules(db).into_iter()
}

pub fn module_full_names<'db>(
    db: &'db RootDatabase,
    module: Module,
    workspace_roots: Rc<[Module]>,
) -> impl Iterator<Item = FunctionFullNameEntry> + 'db {
    let decl_roots = Rc::clone(&workspace_roots);
    let decls = module
        .declarations(db)
        .into_iter()
        .flat_map(move |def| module_def_full_names(db, def, Rc::clone(&decl_roots)));

    let impls = module.impl_defs(db).into_iter().flat_map(move |impl_| {
        impl_full_names(db, impl_, Rc::clone(&workspace_roots))
    });

    decls.chain(impls)
}

fn module_def_full_names<'db>(
    db: &'db RootDatabase,
    def: ModuleDef,
    workspace_roots: Rc<[Module]>,
) -> Box<dyn Iterator<Item = FunctionFullNameEntry> + 'db> {
    match def {
        ModuleDef::Function(function) => option_entry(function_entry(
            db,
            function,
            workspace_roots.as_ref(),
        )),
        ModuleDef::Adt(Adt::Struct(struct_)) => option_entry(tuple_struct_ctor_entry(
            db,
            struct_,
            workspace_roots.as_ref(),
        )),
        ModuleDef::Adt(Adt::Enum(enum_)) => Box::new(enum_full_names(db, enum_, workspace_roots)),
        ModuleDef::Trait(trait_) => Box::new(trait_full_names(db, trait_, workspace_roots)),
        _ => Box::new(std::iter::empty()),
    }
}

fn enum_full_names<'db>(
    db: &'db RootDatabase,
    enum_: Enum,
    workspace_roots: Rc<[Module]>,
) -> impl Iterator<Item = FunctionFullNameEntry> + 'db {
    enum_.variants(db).into_iter().filter_map(move |variant| {
        enum_variant_ctor_entry(db, variant, workspace_roots.as_ref())
    })
}

fn trait_full_names<'db>(
    db: &'db RootDatabase,
    trait_: Trait,
    workspace_roots: Rc<[Module]>,
) -> impl Iterator<Item = FunctionFullNameEntry> + 'db {
    trait_.items(db).into_iter().filter_map(move |item| match item {
        AssocItem::Function(function) => {
            function_entry(db, function, workspace_roots.as_ref())
        }
        _ => None,
    })
}

fn impl_full_names<'db>(
    db: &'db RootDatabase,
    impl_: Impl,
    workspace_roots: Rc<[Module]>,
) -> impl Iterator<Item = FunctionFullNameEntry> + 'db {
    impl_.items(db).into_iter().filter_map(move |item| match item {
        AssocItem::Function(function) => function_entry(db, function, workspace_roots.as_ref()),
        _ => None,
    })
}

fn function_entry(
    db: &RootDatabase,
    function: Function,
    workspace_roots: &[Module],
) -> Option<FunctionFullNameEntry> {
    if !is_function_available_from_workspace(function, db, workspace_roots) {
        return None;
    }

    let method_full_name = format_function_full_name(function, db)?;

    let (is_trait_impl, is_trait_method_def) = match function.as_assoc_item(db) {
        Some(assoc_item) => trait_flags(assoc_item, db),
        None => (false, false),
    };

    Some(FunctionFullNameEntry {
        method_full_name,
        has_self_receiver: function.has_self_param(db),
        is_trait_impl,
        is_trait_method_def,
        is_nightly_only: function.is_unstable(db),
    })
}

fn tuple_struct_ctor_entry(
    db: &RootDatabase,
    struct_: ra_ap_hir::Struct,
    workspace_roots: &[Module],
) -> Option<FunctionFullNameEntry> {
    if !is_available_from_workspace(&struct_, db, workspace_roots) {
        return None;
    }

    if struct_.kind(db) != StructKind::Tuple {
        return None;
    }

    let method_full_name = format_tuple_struct_ctor_full_name(struct_, db)?;

    Some(FunctionFullNameEntry {
        method_full_name,
        has_self_receiver: false,
        is_trait_impl: false,
        is_trait_method_def: false,
        is_nightly_only: struct_.is_unstable(db),
    })
}

fn enum_variant_ctor_entry(
    db: &RootDatabase,
    enum_variant: EnumVariant,
    workspace_roots: &[Module],
) -> Option<FunctionFullNameEntry> {
    if !is_available_from_workspace(&enum_variant, db, workspace_roots) {
        return None;
    }

    match enum_variant.kind(db) {
        StructKind::Tuple | StructKind::Unit => {}
        StructKind::Record => return None,
    }

    let method_full_name = format_enum_variant_full_name(enum_variant, db)?;

    Some(FunctionFullNameEntry {
        method_full_name,
        has_self_receiver: false,
        is_trait_impl: false,
        is_trait_method_def: false,
        is_nightly_only: enum_variant.is_unstable(db),
    })
}

fn option_entry(
    entry: Option<FunctionFullNameEntry>,
) -> Box<dyn Iterator<Item = FunctionFullNameEntry>> {
    Box::new(entry.into_iter())
}

pub fn unique_by_method_full_name<I>(iter: I) -> UniqueByMethodFullName<I>
where
    I: Iterator<Item = FunctionFullNameEntry>,
{
    UniqueByMethodFullName {
        inner: iter,
        seen: HashSet::new(),
    }
}

pub struct UniqueByMethodFullName<I> {
    inner: I,
    seen: HashSet<String>,
}

impl<I> Iterator for UniqueByMethodFullName<I>
where
    I: Iterator<Item = FunctionFullNameEntry>,
{
    type Item = FunctionFullNameEntry;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entry = self.inner.next()?;
            if self.seen.insert(entry.method_full_name.clone()) {
                return Some(entry);
            }
        }
    }
}

fn write_function_fullnames_by_crate<W: Write>(
    writer: &mut W,
    db: &RootDatabase,
) -> anyhow::Result<()> {
    writer
        .write_all(b"{\n")
        .context("failed to write JSON opening brace")?;

    let workspace_roots = workspace_root_modules_rc(db);
    let mut first_crate = true;

    for krate in dependency_crates(db) {
        let crate_name = match krate.display_name(db) {
            Some(name) => name.to_string(),
            None => continue,
        };

        if !first_crate {
            writer
                .write_all(b",\n")
                .context("failed to write crate separator")?;
        }
        first_crate = false;

        serde_json::to_writer(&mut *writer, &crate_name)
            .context("failed to serialize crate name")?;
        writer
            .write_all(b":[\n")
            .context("failed to write array opening")?;

        let entries = unique_by_method_full_name(
            modules_in_crate(db, krate).flat_map(|module| {
                module_full_names(db, module, Rc::clone(&workspace_roots))
            })
        );

        let mut first_entry = true;
        for entry in entries {
            if !first_entry {
                writer
                    .write_all(b",\n")
                    .context("failed to write entry separator")?;
            }
            first_entry = false;
            serde_json::to_writer(&mut *writer, &entry)
                .context("failed to serialize function fullname entry")?;
        }

        writer
            .write_all(b"\n]")
            .context("failed to write array closing")?;
    }

    writer
        .write_all(b"\n}\n")
        .context("failed to write JSON closing brace")?;
    Ok(())
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

fn trait_flags(assoc_item: AssocItem, db: &RootDatabase) -> (bool, bool) {
    match assoc_item.container(db) {
        AssocItemContainer::Trait(_) => (false, true),
        AssocItemContainer::Impl(impl_) => (impl_.trait_ref(db).is_some(), false),
    }
}
