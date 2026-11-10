use ra_ap_hir::{CallableKind, FnTrait, Semantics};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::ast;

pub fn for_call_expr(
    call_expr: &ast::CallExpr,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let callee = call_expr.expr()?;
    let fn_trait = match semantics.resolve_expr_as_callable(&callee)?.kind() {
        CallableKind::Function(_)
        | CallableKind::TupleStruct(_)
        | CallableKind::TupleEnumVariant(_) => return None,
        CallableKind::Closure(closure) => closure.fn_trait(semantics.db),
        CallableKind::FnImpl(fn_trait) => fn_trait,
        CallableKind::FnPtr => FnTrait::Fn,
    };
    Some(fn_trait_full_name(fn_trait).to_owned())
}

fn fn_trait_full_name(fn_trait: FnTrait) -> &'static str {
    match fn_trait {
        FnTrait::Fn => "core::ops::function::Fn",
        FnTrait::FnMut => "core::ops::function::FnMut",
        FnTrait::FnOnce => "core::ops::function::FnOnce",
        FnTrait::AsyncFn => "core::ops::async_function::AsyncFn",
        FnTrait::AsyncFnMut => "core::ops::async_function::AsyncFnMut",
        FnTrait::AsyncFnOnce => "core::ops::async_function::AsyncFnOnce",
    }
}
