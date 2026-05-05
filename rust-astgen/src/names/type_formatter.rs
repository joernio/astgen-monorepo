//! Type formatter based on [`super::rust_name_formatter`].
//! This way we have a single API for formatting names, taking into account generics as well.

use super::rust_name_formatter::{format_module_def_full_name, format_name_with_generic_args};
use ra_ap_hir::{HirDisplay, Module, ModuleDef, Mutability, Type};
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
}

impl<'db> TypeFormatter<'db> {
    fn new(module: Module, db: &'db RootDatabase) -> Self {
        Self { module, db }
    }

    fn format(&self, typ: Type<'db>) -> Option<String> {
        if typ.is_unknown() {
            return None;
        }

        if let Some((inner, mutability)) = typ.as_reference() {
            let prefix = match mutability {
                Mutability::Shared => "&",
                Mutability::Mut => "&mut ",
            };
            let inner = self.format(inner)?;
            return Some(format!("{prefix}{inner}"));
        }

        let display_target = self.module.krate(self.db).to_display_target(self.db);
        if let Some((adt, hir_args)) = typ.as_adt_with_args() {
            let base = format_module_def_full_name(ModuleDef::from(adt), self.db)?;
            let generic_args = typ
                .generic_parameters(self.db, display_target)
                .zip(hir_args)
                .map(|(display_arg, hir_arg)| {
                    hir_arg
                        .map(|hir_arg| self.format(hir_arg))
                        .unwrap_or_else(|| Some(display_arg.to_string()))
                })
                .collect::<Option<Vec<_>>>()?;

            Some(format_name_with_generic_args(base, generic_args))
        } else {
            // TODO(xavierp): ideally recursive
            // fine for a 1st iteration, but ideally we would keep
            // recursively traversing, because if there are ADTs inside
            // non-ADTS (tuples, arrays, function pointers, at least),
            // then it's not guaranteed that display target will still
            // be fully qualified like we wish.
            Some(typ.display(self.db, display_target).to_string())
        }
    }
}
