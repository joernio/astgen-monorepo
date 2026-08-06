//! The actual JSON shape we emit per Rust source file.

use crate::adjustments::{Adjustment, adjustments_for_node};
use crate::json_kind::syntax_kind_to_json_name;
use crate::names::{
    implemented_traits_for_node, method_full_name_for_node, supertraits_for_node,
    type_full_name_for_node,
};
use crate::receivers::has_self_receiver_for_node;
use ra_ap_hir::{
    CfgExpr, CfgOptions, Crate, HirFileId, Semantics, db::ExpandDatabase, prettify_macro_expansion,
};
use ra_ap_ide::{LineIndex, RootDatabase, TextRange};
use ra_ap_syntax::{AstNode, NodeOrToken, SyntaxElement, SyntaxNode, SyntaxToken, ast};
use serde::Serialize;

/// Per-file envelope wrapping the AST.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RustAstGenJsonFile {
    pub(crate) relative_file_path: String,
    pub(crate) full_file_path: String,
    pub(crate) content: String,
    // NB: we may scan a project with multiple crates, so we attach it to the file.
    // In joern this, together with `module_path`, shall give us the namespace_block
    // for this file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) crate_name: Option<String>,
    // The canonical module path for this file, excluding the crate name.
    // So, it's `None` when the file is the crate root module, or if it's
    // unresolvable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) module_path: Option<String>,
    pub(crate) loc: u32,
    pub(crate) children: Vec<RustAstGenJsonNode>,
}

/// A single node or token in the AST.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RustAstGenJsonNode {
    pub(crate) node_kind: String,
    pub(crate) range: RustAstGenJsonNodeRange,
    // When extracting text, prefer this when defined, otherwise substring with range.
    // Currently this is only used for macro-expanded nodes/tokens, which have no suitable range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) method_full_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) type_full_name: Option<String>,
    // Only applicable when node_kind is Struct.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) implemented_traits: Option<Vec<String>>,
    // Only applicable when node_kind is Trait.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) supertraits: Option<Vec<String>>,
    // Only applicable when node_kind is MacroCall.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) macro_expansion: Option<Box<RustAstGenJsonNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) adjustments: Option<Vec<Adjustment>>,
    // Only applicable when node_kind is CallExpr.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) has_self_receiver: Option<bool>,
    pub(crate) children: Vec<RustAstGenJsonNode>,
}

/// Source location range for a node/token.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RustAstGenJsonNodeRange {
    pub(crate) start_offset: u32,
    pub(crate) end_offset: u32,
    pub(crate) start_line: u32,
    pub(crate) start_column: u32,
}

impl RustAstGenJsonNodeRange {
    // NB: remember that macro expansion ranges are not meaningful in the JSON AST,
    // since they are expanded in a separate virtual file (which is not emitted.)
    // This is just a helper to build the JsonNodeRange.
    fn from_text_range(text_range: TextRange, line_index: &LineIndex) -> Self {
        let start = text_range.start();
        let end = text_range.end();
        let start_line_col = line_index.line_col(start);

        Self {
            start_offset: u32::from(start),
            end_offset: u32::from(end),
            start_line: start_line_col.line,
            start_column: start_line_col.col,
        }
    }

    fn empty() -> Self {
        Self {
            start_offset: 0,
            end_offset: 0,
            start_line: 0,
            start_column: 0,
        }
    }
}

impl RustAstGenJsonNode {
    // NB: we need the hir_file_id to know if the current node is coming from
    // a macro expansion. (Macros are expanded in a separate virtual file.)
    pub(crate) fn from_node(
        node: &SyntaxNode,
        hir_file_id: HirFileId,
        line_index: &LineIndex,
        semantics: &Semantics<RootDatabase>,
        target_crate: Crate,
        cfg_options: Option<&CfgOptions>,
    ) -> Self {
        let node_kind = syntax_kind_to_json_name(node.kind());
        let range = Self::make_range(node.text_range(), hir_file_id, line_index);
        let text = macro_text(node, hir_file_id, semantics, target_crate);
        let method_full_name = method_full_name_for_node(node, semantics);
        let type_full_name = type_full_name_for_node(node, semantics);
        let implemented_traits = implemented_traits_for_node(node, semantics);
        let supertraits = supertraits_for_node(node, semantics);
        let adjustments = adjustments_for_node(node, semantics);
        let has_self_receiver = has_self_receiver_for_node(node, semantics);

        let macro_expansion = ast::MacroCall::cast(node.clone())
            .and_then(|macro_call| semantics.expand_macro_call(&macro_call))
            .filter(|expanded| expansion_has_no_errors(expanded.file_id, semantics))
            .map(|expanded| {
                Self::from_node(
                    &expanded.value,
                    expanded.file_id,
                    line_index,
                    semantics,
                    target_crate,
                    cfg_options,
                )
                .into()
            });

        let children = node
            .children_with_tokens()
            .filter(|child| !child.kind().is_trivia())
            .filter(|child| !is_cfg_inactive(child, cfg_options))
            .map(|node_or_token| match node_or_token {
                NodeOrToken::Node(child_node) => Self::from_node(
                    &child_node,
                    hir_file_id,
                    line_index,
                    semantics,
                    target_crate,
                    cfg_options,
                ),
                NodeOrToken::Token(child_token) => {
                    Self::from_token(&child_token, hir_file_id, line_index)
                }
            })
            .collect();

        Self {
            node_kind,
            range,
            text,
            macro_expansion,
            method_full_name,
            type_full_name,
            implemented_traits,
            supertraits,
            adjustments,
            has_self_receiver,
            children,
        }
    }

    pub(crate) fn from_token(
        token: &SyntaxToken,
        hir_file_id: HirFileId,
        line_index: &LineIndex,
    ) -> Self {
        let node_kind = syntax_kind_to_json_name(token.kind());
        let range = Self::make_range(token.text_range(), hir_file_id, line_index);
        let text = hir_file_id.is_macro().then(|| token.text().to_string());
        let children = vec![];

        Self {
            node_kind,
            range,
            text,
            children,
            macro_expansion: None,
            method_full_name: None,
            type_full_name: None,
            implemented_traits: None,
            supertraits: None,
            adjustments: None,
            has_self_receiver: None,
        }
    }

    // Inside a macro expansion, ranges are relative to the macro expansion file.
    // This gets worse as macros can expand into other macros.
    // So, macro-expanded nodes/tokens have empty ranges.
    fn make_range(
        text_range: TextRange,
        hir_file_id: HirFileId,
        line_index: &LineIndex,
    ) -> RustAstGenJsonNodeRange {
        if hir_file_id.is_macro() {
            RustAstGenJsonNodeRange::empty()
        } else {
            RustAstGenJsonNodeRange::from_text_range(text_range, line_index)
        }
    }
}

fn is_cfg_inactive(child: &SyntaxElement, cfg_options: Option<&CfgOptions>) -> bool {
    let (Some(cfg_options), NodeOrToken::Node(node)) = (cfg_options, child) else {
        return false;
    };

    // EXPR_STMT nodes don't have attrs, but their children might.
    // If an EXPR_STMT child is cfg-inactive, we must drop the entire EXPR_STMT, otherwise
    // we'd build an invalid EXPR_STMT AST.

    has_inactive_cfg_attr(node, cfg_options)
        || (ast::ExprStmt::can_cast(node.kind())
            && node
                .children()
                .any(|child| has_inactive_cfg_attr(&child, cfg_options)))
}

fn has_inactive_cfg_attr(node: &SyntaxNode, cfg_options: &CfgOptions) -> bool {
    node.children()
        .filter_map(ast::Attr::cast)
        .filter_map(|attr| match attr.meta()? {
            ast::Meta::CfgMeta(cfg_meta) => cfg_meta.cfg_predicate(),
            _ => None,
        })
        .any(|predicate| cfg_options.check(&CfgExpr::parse_from_ast(predicate)) == Some(false))
}

fn expansion_has_no_errors(hir_file_id: HirFileId, semantics: &Semantics<RootDatabase>) -> bool {
    let Some(macro_file) = hir_file_id.macro_file() else {
        return true;
    };

    let (parse, _) = &semantics.db.parse_macro_expansion(macro_file).value;
    parse.errors().is_empty()
}

// Macro expansions are whitespace-stripped (via `node.text()`), but rust-analyzer
// provides `prettify_macro_expansion` for this purpose.
fn macro_text(
    node: &SyntaxNode,
    hir_file_id: HirFileId,
    semantics: &Semantics<RootDatabase>,
    target_crate: Crate,
) -> Option<String> {
    let macro_file = hir_file_id.macro_file()?;
    let span_map = semantics.db.expansion_span_map(macro_file);
    Some(
        prettify_macro_expansion(semantics.db, node.clone(), span_map, target_crate.into())
            .to_string(),
    )
}
