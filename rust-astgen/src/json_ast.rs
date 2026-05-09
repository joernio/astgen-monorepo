//! The actual JSON shape we emit per Rust source file.

use crate::names::{method_full_name_for_node, type_full_name_for_node};
use ra_ap_hir::Semantics;
use ra_ap_ide::{LineIndex, RootDatabase};
use ra_ap_syntax::{NodeOrToken, SyntaxNode, SyntaxToken};
use serde::Serialize;

/// Per-file envelope wrapping the AST.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RustAstGenJsonFile {
    pub(crate) relative_file_path: String,
    pub(crate) full_file_path: String,
    pub(crate) content: String,
    // NB: we may scan a project with multiple crates, so we attach it to the file.
    // In joern this shall give us the namespace_block for this file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) crate_name: Option<String>,
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
    pub(crate) fn from_node(node: &SyntaxNode, line_index: &LineIndex) -> Self {
        let text_range = node.text_range();
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

    pub(crate) fn from_token(token: &SyntaxToken, line_index: &LineIndex) -> Self {
        let text_range = token.text_range();
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
}

impl RustAstGenJsonNode {
    pub(crate) fn from_node(
        node: &SyntaxNode,
        line_index: &LineIndex,
        semantics: &Semantics<RootDatabase>,
    ) -> Self {
        let node_kind = format!("{:?}", node.kind());
        let range = RustAstGenJsonNodeRange::from_node(node, line_index);
        let method_full_name = method_full_name_for_node(node, semantics);
        let type_full_name = type_full_name_for_node(node, semantics);
        let children = node
            .children_with_tokens()
            .filter(|child| !child.kind().is_trivia())
            .map(|node_or_token| match node_or_token {
                NodeOrToken::Node(child_node) => {
                    Self::from_node(&child_node, line_index, semantics)
                }
                NodeOrToken::Token(child_token) => Self::from_token(&child_token, line_index),
            })
            .collect();

        Self {
            node_kind,
            range,
            method_full_name,
            type_full_name,
            children,
        }
    }

    pub(crate) fn from_token(token: &SyntaxToken, line_index: &LineIndex) -> Self {
        let node_kind = format!("{:?}", token.kind());
        let range = RustAstGenJsonNodeRange::from_token(token, line_index);
        let children = vec![];

        Self {
            node_kind,
            range,
            children,
            method_full_name: None,
            type_full_name: None,
        }
    }
}
