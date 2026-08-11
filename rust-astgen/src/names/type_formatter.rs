//! Type formatter based on [`super::rust_name_formatter`].
//! This way we have a single API for formatting names, taking into account generics as well.
//!
//! We need to traverse the type and format each case ourselves: the provided `display` and
//! `display_source_code` do not emit fully qualified paths.

use super::rust_name_formatter::{
    format_generic_args_for_def, format_item_name, format_module_def_full_name,
    format_name_with_generic_args,
};
use ra_ap_hir::{
    Adt, AssocItem, Callable, DisplayTarget, GenericDef, HirDisplay, Impl, Module, ModuleDef,
    Mutability, PathResolution, Semantics, Trait, Type,
};
use ra_ap_ide::RootDatabase;
use ra_ap_syntax::ast;

pub(crate) fn format_type(typ: &Type, module: Module, db: &RootDatabase) -> Option<String> {
    TypeFormatter::new(module, db).format(typ)
}

pub(crate) fn format_impl_self_ty(
    impl_: Impl,
    module: Module,
    db: &RootDatabase,
) -> Option<String> {
    let self_ty = impl_.self_ty(db);
    let self_ty_is_associated_type = self_ty.as_associated_type_parent_trait(db).is_some();
    if !self_ty_is_associated_type {
        return format_type(&self_ty, module, db);
    }

    let semantics = Semantics::new(db);
    if let Some(source) = semantics.source(impl_)
        && let Some(ast::Type::PathType(path_type)) = source.value.self_ty()
        && let Some(path) = path_type.path()
        && let Some(segment) = path.qualifier().and_then(|qualifier| qualifier.segment())
        && let Some(anchor) = segment.type_anchor().and_then(|anchor| anchor.ty())
        && let Some(anchor) = semantics.resolve_type(&anchor)
        && let Some(PathResolution::Def(ModuleDef::TypeAlias(assoc_type))) =
            semantics.resolve_path(&path)
        && let Some(normalized) = anchor.normalize_trait_assoc_type(db, &[], assoc_type)
    {
        return format_type(&normalized, module, db);
    }

    format_type(&self_ty, module, db)
}

struct TypeFormatter<'db> {
    module: Module,
    db: &'db RootDatabase,
    display_target: DisplayTarget,
}

impl<'db> TypeFormatter<'db> {
    fn new(module: Module, db: &'db RootDatabase) -> Self {
        let display_target = module.krate(db).to_display_target(db);
        Self {
            module,
            db,
            display_target,
        }
    }

    fn format(&self, typ: &Type) -> Option<String> {
        if typ.is_unknown() {
            return None;
        }
        if let Some((inner, mutability)) = typ.as_reference() {
            return self.format_reference(&inner, mutability);
        }
        if let Some((adt, hir_args)) = typ.as_adt_with_args() {
            return self.format_adt(adt, hir_args, typ);
        }
        if typ.is_tuple() {
            return self.format_tuple(typ.tuple_fields(self.db));
        }
        if let Some((inner, len)) = typ.as_array(self.db) {
            return self.format_array(&inner, len);
        }
        if let Some(inner) = typ.as_slice() {
            return self.format_slice(&inner);
        }
        if let Some((inner, mutability)) = typ.as_raw_ptr() {
            return self.format_raw_ptr(&inner, mutability);
        }
        if typ.is_fn()
            && let Some(callable) = typ.as_callable(self.db)
        {
            return self.format_fn(callable);
        }
        if let Some(traits) = typ.as_impl_traits(self.db) {
            return self.format_impl_trait(typ, traits);
        }
        if let Some(trait_) = typ.as_dyn_trait() {
            return self.format_dyn_trait(typ, trait_);
        }
        // Replace implicit `Self` with the trait's name.
        if let Some(type_param) = typ.as_type_param(self.db)
            && type_param.is_implicit(self.db)
            && let GenericDef::Trait(trait_) = type_param.parent(self.db)
        {
            return self.format_trait_self(trait_);
        }
        Some(self.format_fallback(typ))
    }

    fn format_trait_self(&self, trait_: Trait) -> Option<String> {
        let base = format_module_def_full_name(ModuleDef::from(trait_), self.db)?;
        let args =
            format_generic_args_for_def(GenericDef::from(trait_), trait_.module(self.db), self.db);
        Some(format_name_with_generic_args(base, args))
    }

    fn format_reference(&self, inner: &Type, mutability: Mutability) -> Option<String> {
        let prefix = match mutability {
            Mutability::Shared => "&",
            Mutability::Mut => "&mut ",
        };
        Some(format!("{prefix}{}", self.format(inner)?))
    }

    fn format_adt(&self, adt: Adt, hir_args: Vec<Option<Type>>, typ: &Type) -> Option<String> {
        let base = format_module_def_full_name(ModuleDef::from(adt), self.db)?;
        let generic_args = typ
            .generic_parameters(self.db, self.display_target)
            .zip(hir_args)
            .map(|(display_arg, hir_arg)| {
                hir_arg
                    .map(|hir_arg| self.format(&hir_arg))
                    .unwrap_or_else(|| Some(display_arg.to_string()))
            })
            .collect::<Option<Vec<_>>>()?;
        Some(format_name_with_generic_args(base, generic_args))
    }

    fn format_tuple(&self, fields: Vec<Type>) -> Option<String> {
        let parts = fields
            .into_iter()
            .map(|t| self.format(&t))
            .collect::<Option<Vec<_>>>()?;
        Some(match parts.as_slice() {
            [] => "()".to_string(),
            [single] => format!("({single},)"),
            _ => format!("({})", parts.join(", ")),
        })
    }

    fn format_array(&self, inner: &Type, len: usize) -> Option<String> {
        Some(format!("[{}; {len}]", self.format(inner)?))
    }

    fn format_slice(&self, inner: &Type) -> Option<String> {
        Some(format!("[{}]", self.format(inner)?))
    }

    fn format_raw_ptr(&self, inner: &Type, mutability: Mutability) -> Option<String> {
        let prefix = match mutability {
            Mutability::Shared => "*const",
            Mutability::Mut => "*mut",
        };
        Some(format!("{prefix} {}", self.format(inner)?))
    }

    fn format_fn(&self, callable: Callable<'db>) -> Option<String> {
        let params = callable
            .params()
            .into_iter()
            .map(|p| self.format(p.ty()))
            .collect::<Option<Vec<_>>>()?;
        let ret = self.format(&callable.return_type())?;
        Some(format!("fn({}) -> {ret}", params.join(", ")))
    }

    fn format_impl_trait(&self, typ: &Type, traits: impl Iterator<Item = Trait>) -> Option<String> {
        let bounds = traits
            .map(|trait_| self.format_trait_bound(typ, trait_))
            .collect::<Option<Vec<_>>>()?;
        Some(format!("impl {}", bounds.join(" + ")))
    }

    fn format_trait_bound(&self, typ: &Type, trait_: Trait) -> Option<String> {
        let base = format_module_def_full_name(ModuleDef::from(trait_), self.db)?;
        let bindings = trait_
            .items(self.db)
            .into_iter()
            .filter_map(|item| match item {
                AssocItem::TypeAlias(alias) => {
                    let value = typ.normalize_trait_assoc_type(self.db, &[], alias)?;
                    Some((alias, value))
                }
                _ => None,
            })
            .map(|(alias, value)| {
                let name = format_item_name(alias.name(self.db), self.module, self.db);
                Some(format!("{name} = {}", self.format(&value)?))
            })
            .collect::<Option<Vec<_>>>()?;
        Some(format_name_with_generic_args(base, bindings))
    }

    fn format_dyn_trait(&self, typ: &Type, trait_: Trait) -> Option<String> {
        Some(format!("dyn {}", self.format_trait_bound(typ, trait_)?))
    }

    fn format_fallback(&self, typ: &Type) -> String {
        typ.display_source_code(self.db, self.module.into(), true)
            .unwrap_or_else(|_| typ.display(self.db, self.display_target).to_string())
    }
}
