//! Whether a `CallExpr`'s first argument is a `self` receiver.

use ra_ap_hir::{CallableKind, Semantics};
use ra_ap_ide::RootDatabase;
use ra_ap_syntax::{AstNode, SyntaxNode, ast};

pub(crate) fn has_self_receiver_for_node(
    node: &SyntaxNode,
    semantics: &Semantics<RootDatabase>,
) -> Option<bool> {
    let call_expr = ast::CallExpr::cast(node.clone())?;
    let callee = call_expr.expr()?;
    match semantics.resolve_expr_as_callable(&callee)?.kind() {
        CallableKind::Function(function) if function.has_self_param(semantics.db) => Some(true),
        _ => None,
    }
}
