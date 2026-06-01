//! The actual JSON shape we emit per Rust source file.

use crate::names::{method_full_name_for_node, type_full_name_for_node};
use ra_ap_hir::{HirFileId, Semantics};
use ra_ap_ide::{LineIndex, RootDatabase, TextRange};
use ra_ap_syntax::{AstNode, NodeOrToken, SyntaxNode, SyntaxToken, ast};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) method_full_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) type_full_name: Option<String>,
    // Only applicable when node_kind is MacroCall.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) macro_expansion: Option<Box<RustAstGenJsonNode>>,
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
    ) -> Self {
        let node_kind = format!("{:?}", node.kind());
        let range = Self::make_range(node.text_range(), hir_file_id, line_index);
        let method_full_name = method_full_name_for_node(node, semantics);
        let type_full_name = type_full_name_for_node(node, semantics);

        let macro_expansion = ast::MacroCall::cast(node.clone())
            .and_then(|macro_call| semantics.expand_macro_call(&macro_call))
            .map(|expanded| {
                Self::from_node(&expanded.value, expanded.file_id, line_index, semantics).into()
            });

        let children = node
            .children_with_tokens()
            .filter(|child| !child.kind().is_trivia())
            .map(|node_or_token| match node_or_token {
                NodeOrToken::Node(child_node) => {
                    Self::from_node(&child_node, hir_file_id, line_index, semantics)
                }
                NodeOrToken::Token(child_token) => {
                    Self::from_token(&child_token, hir_file_id, line_index)
                }
            })
            .collect();

        Self {
            node_kind,
            range,
            macro_expansion,
            method_full_name,
            type_full_name,
            children,
        }
    }

    pub(crate) fn from_token(
        token: &SyntaxToken,
        hir_file_id: HirFileId,
        line_index: &LineIndex,
    ) -> Self {
        let node_kind = format!("{:?}", token.kind());
        let range = Self::make_range(token.text_range(), hir_file_id, line_index);
        let children = vec![];

        Self {
            node_kind,
            range,
            children,
            macro_expansion: None,
            method_full_name: None,
            type_full_name: None,
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
