use crate::{cargo, config};
use anyhow::Context;
use log::{error, info};
use ra_ap_hir::{Semantics, attach_db};
use ra_ap_ide::{Analysis, AnalysisHost, LineIndex, RootDatabase};
use ra_ap_syntax::{AstNode, NodeOrToken, SyntaxNode, SyntaxToken};
use ra_ap_vfs::{FileId, VfsPath};
use serde::Serialize;
use std::path::Path;

/// Per-file envelope wrapping the AST.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RustAstGenJsonFile {
    pub(crate) relative_file_path: String,
    pub(crate) full_file_path: String,
    pub(crate) content: String,
    pub(crate) loc: u32,
    pub(crate) children: Vec<RustAstGenJsonNode>,
}

/// A single node or token in the AST.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RustAstGenJsonNode {
    pub(crate) node_kind: String,
    pub(crate) range: RustAstGenJsonNodeRange,
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
    pub(crate) fn from_node(node: &SyntaxNode, line_index: &LineIndex) -> Self {
        let node_kind = format!("{:?}", node.kind());
        let range = RustAstGenJsonNodeRange::from_node(node, line_index);
        let children = node
            .children_with_tokens()
            .filter(|child| !child.kind().is_trivia())
            .map(|node_or_token| match node_or_token {
                NodeOrToken::Node(child_node) => Self::from_node(&child_node, line_index),
                NodeOrToken::Token(child_token) => {
                    RustAstGenJsonNode::from_token(&child_token, line_index)
                }
            })
            .collect();

        Self {
            node_kind,
            range,
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
        }
    }
}

pub(crate) fn write_json_to_file(json_tree: &str, output_file: &Path) -> anyhow::Result<()> {
    let output_parent = output_file.parent().with_context(|| {
        format!(
            "failed to get parent directory of output file: {}",
            output_file.display()
        )
    })?;

    std::fs::create_dir_all(output_parent).with_context(|| {
        format!(
            "failed to create output directory for: {}",
            output_file.display()
        )
    })?;

    std::fs::write(&output_file, json_tree)
        .with_context(|| format!("failed to write JSON to file: {}", output_file.display()))
}

pub fn run(config: &config::RustAstGenConfig) -> anyhow::Result<()> {
    // Load the workspace
    let (root_db, vfs) = cargo::load_workspace(config)?;

    // Load the project model
    let analysis_host = AnalysisHost::with_database(root_db);
    let analysis = analysis_host.analysis();
    let root_db = analysis_host.raw_database();
    let semantics = Semantics::new(root_db);

    // Pick only the relevant files: those inside the input directory
    let input_rust_files = cargo::collect_input_files(config, &vfs)?;

    // Process each file
    attach_db(semantics.db, || {
        for (file_id, file_vfs_path) in input_rust_files {
            if let Err(e) = process_file(file_id, file_vfs_path, &analysis, &semantics, config) {
                error!("{e}")
            }
        }
    });

    Ok(())
}

fn process_file(
    file_id: FileId,
    file_vfs_path: VfsPath,
    analysis: &Analysis,
    semantics: &Semantics<RootDatabase>,
    config: &config::RustAstGenConfig,
) -> anyhow::Result<()> {
    let input_file_path = file_vfs_path
        .as_path()
        .map(AsRef::<Path>::as_ref)
        .with_context(|| format!("failed to convert VfsPath to Path: {:?}", file_vfs_path))?;

    info!("parsing: {}", input_file_path.display());

    let source_file = semantics.parse_guess_edition(file_id);
    let syntax_tree = source_file.syntax();
    let file_line_index = analysis.file_line_index(file_id)?;

    info!("building the JSON tree: {}", input_file_path.display());

    let json_root = RustAstGenJsonNode::from_node(&syntax_tree, &file_line_index);
    let contents = syntax_tree.text().to_string();
    let loc = file_line_index
        .line_col(syntax_tree.text_range().end())
        .line;
    // TODO: we already have similar in config. Refactor
    let relative_path = input_file_path
        .strip_prefix(&config.input_dir_full_path)
        .with_context(|| format!("failed to strip prefix: {:?}", input_file_path))?;

    let envelope = RustAstGenJsonFile {
        relative_file_path: relative_path.to_string_lossy().to_string(),
        full_file_path: input_file_path.to_string_lossy().to_string(),
        content: contents,
        loc,
        children: vec![json_root],
    };

    let output_file = config.make_output_path_for_input_file(&input_file_path.to_path_buf())?;

    info!("writing to: {}", output_file.display());

    let json_tree = serde_json::to_string_pretty(&envelope)?;
    write_json_to_file(&json_tree, &output_file)?;

    Ok(())
}
