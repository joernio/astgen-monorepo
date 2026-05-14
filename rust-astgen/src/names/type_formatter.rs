//! Type formatter based on [`super::rust_name_formatter`].
//! This way we have a single API for formatting names, taking into account generics as well.
//!
//! We need to traverse the type and format each case ourselves: the provided `display` and
//! `display_source_code` do not emit fully qualified paths.

use super::rust_name_formatter::{format_module_def_full_name, format_name_with_generic_args};
use ra_ap_hir::{Adt, Callable, DisplayTarget, HirDisplay, Module, ModuleDef, Mutability, Type};
use ra_ap_ide::RootDatabase;

pub(crate) fn format_type<'db>(
    typ: Type<'db>,
    module: Module,
    db: &'db RootDatabase,
) -> Option<String> {
    TypeFormatter::new(module, db).format(typ)
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

    fn format(&self, typ: Type<'db>) -> Option<String> {
        if typ.is_unknown() {
            return None;
        }
        if let Some((inner, mutability)) = typ.as_reference() {
            return self.format_reference(inner, mutability);
        }
        if let Some((adt, hir_args)) = typ.as_adt_with_args() {
            return self.format_adt(adt, hir_args, &typ);
        }
        if typ.is_tuple() {
            return self.format_tuple(typ.tuple_fields(self.db));
        }
        if let Some((inner, len)) = typ.as_array(self.db) {
            return self.format_array(inner, len);
        }
        if let Some(inner) = typ.as_slice() {
            return self.format_slice(inner);
        }
        if let Some(inner) = typ.remove_raw_ptr() {
            return self.format_raw_ptr(inner);
        }
        if typ.is_fn()
            && let Some(callable) = typ.as_callable(self.db)
        {
            return self.format_fn(callable);
        }
        Some(self.format_fallback(&typ))
    }

    fn format_reference(&self, inner: Type<'db>, mutability: Mutability) -> Option<String> {
        let prefix = match mutability {
            Mutability::Shared => "&",
            Mutability::Mut => "&mut ",
        };
        Some(format!("{prefix}{}", self.format(inner)?))
    }

    fn format_adt(
        &self,
        adt: Adt,
        hir_args: Vec<Option<Type<'db>>>,
        typ: &Type<'db>,
    ) -> Option<String> {
        let base = format_module_def_full_name(ModuleDef::from(adt), self.db)?;
        let generic_args = typ
            .generic_parameters(self.db, self.display_target)
            .zip(hir_args)
            .map(|(display_arg, hir_arg)| {
                hir_arg
                    .map(|hir_arg| self.format(hir_arg))
                    .unwrap_or_else(|| Some(display_arg.to_string()))
            })
            .collect::<Option<Vec<_>>>()?;
        Some(format_name_with_generic_args(base, generic_args))
    }

    fn format_tuple(&self, fields: Vec<Type<'db>>) -> Option<String> {
        let parts = fields
            .into_iter()
            .map(|t| self.format(t))
            .collect::<Option<Vec<_>>>()?;
        Some(match parts.as_slice() {
            [] => "()".to_string(),
            [single] => format!("({single},)"),
            _ => format!("({})", parts.join(", ")),
        })
    }

    fn format_array(&self, inner: Type<'db>, len: usize) -> Option<String> {
        Some(format!("[{}; {len}]", self.format(inner)?))
    }

    fn format_slice(&self, inner: Type<'db>) -> Option<String> {
        Some(format!("[{}]", self.format(inner)?))
    }

    fn format_raw_ptr(&self, inner: Type<'db>) -> Option<String> {
        Some(format!("*{}", self.format(inner)?))
    }

    fn format_fn(&self, callable: Callable<'db>) -> Option<String> {
        let params = callable
            .params()
            .into_iter()
            .map(|p| self.format(p.ty().clone()))
            .collect::<Option<Vec<_>>>()?;
        let ret = self.format(callable.return_type())?;
        Some(format!("fn({}) -> {ret}", params.join(", ")))
    }

    fn format_fallback(&self, typ: &Type<'db>) -> String {
        typ.display_source_code(self.db, self.module.into(), true)
            .unwrap_or_else(|_| typ.display(self.db, self.display_target).to_string())
    }
}
