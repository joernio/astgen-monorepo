use crate::json::RustAstGenJsonNode;
use anyhow::{Context, Result};
use log::{error, info};
use ra_ap_hir::{Semantics, attach_db};
use ra_ap_ide::{Analysis, AnalysisHost, LineIndex, RootDatabase};
use ra_ap_syntax::SyntaxNode;
use ra_ap_syntax::ast::AstNode;
use ra_ap_vfs::{FileId, VfsPath};
use std::path::Path;

mod cargo;
pub mod config;
mod json;

pub fn run(config: &config::RustAstGenConfig) -> Result<()> {
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
) -> Result<()> {
    let input_file_path = file_vfs_path
        .as_path()
        .map(AsRef::<Path>::as_ref)
        .with_context(|| format!("failed to convert VfsPath to Path: {:?}", file_vfs_path))?;

    info!("parsing: {}", input_file_path.display());

    let source_file = semantics.parse_guess_edition(file_id);
    let syntax_tree = source_file.syntax();
    let file_line_index = analysis.file_line_index(file_id)?;

    info!("building the JSON tree: {}", input_file_path.display());

    let rust_ast_gen_json_node = make_json_node(&syntax_tree, &file_line_index);
    let output_file = config.make_output_path_for_input_file(&input_file_path.to_path_buf())?;

    info!("writing to: {}", output_file.display());

    let json_tree = serde_json::to_string_pretty(&rust_ast_gen_json_node)?;
    write_json_to_file(&json_tree, &output_file)?;

    Ok(())
}

fn make_json_node(node: &SyntaxNode, line_index: &LineIndex) -> RustAstGenJsonNode {
    let node_kind = format!("{:?}", node.kind());

    // Note, LineIndex is 0-based (in both line and column)
    let text_range = node.text_range();
    let start = text_range.start();
    let end = text_range.end();
    let start_line_col = line_index.line_col(start);
    let start_line = start_line_col.line;
    let start_column = start_line_col.col;
    let start_offset = u32::from(start);
    let end_offset = u32::from(end);

    let children = node
        .children()
        .map(|child| make_json_node(&child, &line_index))
        .collect::<Vec<RustAstGenJsonNode>>();

    RustAstGenJsonNode {
        node_kind,
        start_offset,
        end_offset,
        start_line,
        start_column,
        children,
    }
}

fn write_json_to_file(json_tree: &str, output_file: &Path) -> Result<()> {
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
