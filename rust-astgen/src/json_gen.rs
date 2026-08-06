use crate::json_ast::{RustAstGenJsonFile, RustAstGenJsonNode};
use crate::{cargo, config};
use anyhow::Context;
use log::{debug, error};
use ra_ap_hir::{Crate, Semantics, attach_db};
use ra_ap_ide::{Analysis, AnalysisHost, RootDatabase};
use ra_ap_syntax::{AstNode, SyntaxNode};
use ra_ap_vfs::{FileId, VfsPath};
use std::path::Path;

fn write_json_to_file(json_tree: &str, output_file: &Path) -> anyhow::Result<()> {
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

    std::fs::write(output_file, json_tree)
        .with_context(|| format!("failed to write JSON to file: {}", output_file.display()))
}

pub fn run(config: &config::RustAstGenConfig) -> anyhow::Result<()> {
    let (analysis_host, input_rust_files) = load_inputs(config)?;
    process_inputs(&analysis_host, input_rust_files, config);
    Ok(())
}

fn load_inputs(
    config: &config::RustAstGenConfig,
) -> anyhow::Result<(AnalysisHost, Vec<(FileId, VfsPath)>)> {
    let (root_db, vfs) = cargo::load_workspace(config)?;
    let analysis_host = AnalysisHost::with_database(root_db);
    let input_rust_files = cargo::collect_input_files(config, &vfs)?;
    Ok((analysis_host, input_rust_files))
}

fn process_inputs(
    analysis_host: &AnalysisHost,
    input_rust_files: Vec<(FileId, VfsPath)>,
    config: &config::RustAstGenConfig,
) {
    let files_per_worker = input_rust_files
        .len()
        .div_ceil(config.worker_threads)
        .max(1);

    std::thread::scope(|scope| {
        for files in input_rust_files.chunks(files_per_worker) {
            let analysis = analysis_host.analysis();
            let root_db = analysis_host.raw_database().to_owned();

            scope.spawn(move || process_files(files, analysis, root_db, config));
        }
    });
}

fn process_files(
    input_rust_files: &[(FileId, VfsPath)],
    analysis: Analysis,
    root_db: RootDatabase,
    config: &config::RustAstGenConfig,
) {
    let semantics = Semantics::new(&root_db);

    // Process each file
    attach_db(semantics.db, || {
        for (file_id, file_vfs_path) in input_rust_files {
            let input_file_path = file_vfs_path.as_path().map(AsRef::<Path>::as_ref);

            let file_result = if let Some(input_file_path) = input_file_path {
                if let Err(e) =
                    process_file(*file_id, input_file_path, &analysis, &semantics, config)
                {
                    error!("{e}");
                    None
                } else {
                    Some(())
                }
            } else {
                error!("failed to convert VfsPath to Path: {:?}", file_vfs_path);
                None
            };

            // Writing to stdout on purpose so joern's AstGenRunner can detect
            // that some relevant file was skipped.
            if file_result.is_none() {
                println!("Skipped: {}", file_vfs_path);
            }
        }
    });
}

fn process_file(
    file_id: FileId,
    input_file_path: &Path,
    analysis: &Analysis,
    semantics: &Semantics<RootDatabase>,
    config: &config::RustAstGenConfig,
) -> anyhow::Result<()> {
    debug!("parsing: {}", input_file_path.display());
    let source_file = semantics.parse_guess_edition(file_id);
    let syntax_tree = source_file.syntax();

    // If there's no crate, we don't have any type information. Likely, an inactive `#[cfg(..)]`.
    // Thus, skip it.
    let Some(target_crate) = crate_for_file(syntax_tree, semantics) else {
        println!("Skipped: {}", input_file_path.display());
        return Ok(());
    };

    let file_line_index = analysis.file_line_index(file_id)?;

    debug!("building the JSON tree: {}", input_file_path.display());

    let hir_file_id = semantics.hir_file_for(syntax_tree);
    let cfg_options = config.resolve_cfg.then(|| target_crate.cfg(semantics.db));
    let json_root = RustAstGenJsonNode::from_node(
        syntax_tree,
        hir_file_id,
        &file_line_index,
        semantics,
        target_crate,
        cfg_options,
    );
    let contents = syntax_tree.text().to_string();
    let loc = file_line_index
        .line_col(syntax_tree.text_range().end())
        .line;
    let relative_path = config.relativize_input_file(input_file_path)?;

    let crate_name = target_crate
        .display_name(semantics.db)
        .map(|name| name.to_string());
    let module_path = module_path_for_file(syntax_tree, semantics);
    let envelope = RustAstGenJsonFile {
        relative_file_path: relative_path.to_string_lossy().to_string(),
        full_file_path: input_file_path.to_string_lossy().to_string(),
        content: contents,
        crate_name,
        module_path,
        loc,
        children: vec![json_root],
    };

    let output_file = config.make_output_path_for_input_file(input_file_path)?;

    debug!("writing to: {}", output_file.display());

    let json_tree = if config.pretty_print {
        serde_json::to_string_pretty(&envelope)?
    } else {
        serde_json::to_string(&envelope)?
    };
    write_json_to_file(&json_tree, &output_file)?;

    Ok(())
}

fn crate_for_file(syntax_tree: &SyntaxNode, semantics: &Semantics<RootDatabase>) -> Option<Crate> {
    semantics
        .scope(syntax_tree)?
        .module()
        .krate(semantics.db)
        .into()
}

fn module_path_for_file(
    syntax_tree: &SyntaxNode,
    semantics: &Semantics<RootDatabase>,
) -> Option<String> {
    let module = semantics.scope(syntax_tree)?.module();
    let edition = module.krate(semantics.db).edition(semantics.db);
    let segments = module
        .path_to_root(semantics.db)
        .into_iter()
        .rev()
        .filter_map(|m| m.name(semantics.db))
        .map(|name| name.display(semantics.db, edition).to_string())
        .collect::<Vec<String>>();

    if segments.is_empty() {
        None
    } else {
        Some(segments.join("::"))
    }
}
