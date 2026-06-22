//! Where we attach implicit dereferences/borrows/etc ("adjustments") to each expression.
//!
//! Adjustments can be chained, so there's a list of them, and the order is relevant.

use crate::names::method_full_names;
use crate::names::type_formatter;
use log::debug;
use ra_ap_hir::{Adjust, AssocItem, Impl, LangItem, Module, Mutability, Semantics, Trait, Type};
use ra_ap_ide::RootDatabase;
use ra_ap_syntax::{AstNode, SyntaxNode, ast};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(rename_all_fields = "camelCase")]
#[serde(tag = "kind")]
pub(crate) enum Adjustment {
    // Built-in dereferences, e.g. `&T -> T`.
    Deref {
        source: String,
        target: String,
    },
    // Data type-specific dereferences, e.g. `Vec<T> -> &[T]`.
    OverloadedDeref {
        source: String,
        target: String,
        mutable: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        method_full_name: Option<String>,
    },
    Borrow {
        source: String,
        target: String,
    },
    Cast {
        source: String,
        target: String,
    },
}

pub(crate) fn adjustments_for_node(
    node: &SyntaxNode,
    semantics: &Semantics<RootDatabase>,
) -> Option<Vec<Adjustment>> {
    let expr = ast::Expr::cast(node.clone())?;
    let steps = semantics.expr_adjustments(&expr)?;
    let module = semantics.scope(node)?.module();
    let mut adjustments = Vec::with_capacity(steps.len());
    for (index, step) in steps.into_iter().enumerate() {
        let Some(adjustment) = convert_adjustment(&step, module, semantics.db) else {
            debug!("failed to convert adjustment {} in {:?}", index, step);
            return None;
        };
        adjustments.push(adjustment);
    }
    Some(adjustments)
}

fn convert_adjustment<'db>(
    step: &ra_ap_hir::Adjustment<'db>,
    module: Module,
    db: &'db RootDatabase,
) -> Option<Adjustment> {
    let source = type_formatter::format_type(&step.source, module, db)?;
    let target = type_formatter::format_type(&step.target, module, db)?;
    let adjust = match step.kind {
        Adjust::NeverToAny => Adjustment::Cast { source, target },
        Adjust::Deref(None) => Adjustment::Deref { source, target },
        Adjust::Deref(Some(deref)) => Adjustment::OverloadedDeref {
            source,
            target,
            mutable: deref.0 == Mutability::Mut,
            method_full_name: overloaded_deref_method_full_name(&step.source, deref.0, module, db),
        },
        Adjust::Borrow(_) => Adjustment::Borrow { source, target },
        Adjust::Pointer(_) => Adjustment::Cast { source, target },
    };
    Some(adjust)
}

fn overloaded_deref_method_full_name<'db>(
    source: &Type<'db>,
    mutability: Mutability,
    module: Module,
    db: &'db RootDatabase,
) -> Option<String> {
    let resolved = resolve_deref_method(source, mutability, module, db);
    if resolved.is_none() && source.as_type_param(db).is_none() {
        debug!("no deref method found for {:?}", source);
    }
    resolved
}

fn resolve_deref_method<'db>(
    source: &Type<'db>,
    mutability: Mutability,
    module: Module,
    db: &'db RootDatabase,
) -> Option<String> {
    let lang_item = match mutability {
        Mutability::Mut => LangItem::DerefMut,
        Mutability::Shared => LangItem::Deref,
    };
    let deref_trait = Trait::lang(db, module.krate(db), lang_item)?;
    let deref_impl = Impl::all_for_type(db, source.clone())
        .into_iter()
        .find(|imp| imp.trait_(db) == Some(deref_trait))?;
    let deref_fn = deref_impl
        .items(db)
        .into_iter()
        .find_map(|item| match item {
            AssocItem::Function(func) => Some(func),
            _ => None,
        })?;
    method_full_names::format_function_full_name(deref_fn, db)
}
