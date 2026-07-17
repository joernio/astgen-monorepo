mod ast;
mod collect;
mod parse;
mod scala_gen;

use clap::Parser;
use collect::collect_perl_files;
use parse::parse_file;
use rayon::prelude::*;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Parser)]
#[command(
    name = "perl-astgen",
    about = "Parse Perl files and emit AST JSON",
    version
)]
struct Cli {
    /// Source directory or file (default: `.`)
    #[arg(short = 'i', long = "src", value_name = "src", default_value = ".")]
    input: PathBuf,

    /// Output directory for generated AST json files (default: `./ast_out`)
    #[arg(
        short = 'o',
        long = "output",
        value_name = "output",
        default_value = "./ast_out"
    )]
    output_dir: PathBuf,

    /// Exclude a specific file (by absolute path). Can be specified multiple times.
    #[arg(long = "exclude-file", value_name = "PATH")]
    exclude_files: Vec<PathBuf>,

    /// Exclude files whose absolute path matches this regex
    #[arg(long = "exclude-regex", value_name = "PATTERN")]
    exclude_regex: Option<String>,

    /// Only generate the Scala PerlNodeSyntax AST types file (writes `./PerlNodeSyntax.scala`)
    #[arg(short = 's', long = "scala-ast-only")]
    scala_ast_only: bool,

    /// Pretty-print the generated AST JSON files
    #[arg(short = 'p', long = "pretty-print")]
    pretty_print: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.scala_ast_only {
        let output = std::path::Path::new(scala_gen::DEFAULT_OUTPUT_PATH);
        if let Err(e) = scala_gen::generate(output) {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    let exclude_regex: Option<Regex> = match cli.exclude_regex {
        Some(ref pattern) => match Regex::new(pattern) {
            Ok(re) => Some(re),
            Err(e) => {
                eprintln!("warning: invalid --exclude-regex '{}': {}", pattern, e);
                None
            }
        },
        None => None,
    };

    let mut exclude_files: Vec<PathBuf> = Vec::new();
    for p in &cli.exclude_files {
        match p.canonicalize() {
            Ok(canonical) => exclude_files.push(canonical),
            Err(e) => eprintln!(
                "warning: cannot resolve --exclude-file '{}': {}",
                p.display(),
                e
            ),
        }
    }

    let files = collect_perl_files(&cli.input, &exclude_files, exclude_regex.as_ref());

    if files.is_empty() {
        eprintln!("warning: no Perl files found in '{}'", cli.input.display());
        std::process::exit(0);
    }

    if let Err(e) = std::fs::create_dir_all(&cli.output_dir) {
        eprintln!(
            "error: cannot create output directory '{}': {}",
            cli.output_dir.display(),
            e
        );
        std::process::exit(1);
    }

    let input_base: PathBuf = if cli.input.is_dir() {
        cli.input.canonicalize().unwrap_or(cli.input.clone())
    } else {
        // For single-file input, derive base from the canonicalized file's parent.
        // cli.input may be relative/bare, so use the already-canonicalized path from `files`.
        files
            .first()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_default()
    };

    // Pre-compute all (source, output) pairs and collect unique output directories.
    let pairs: Vec<(PathBuf, PathBuf)> = files
        .iter()
        .map(|source_path| {
            let rel = source_path.strip_prefix(&input_base).unwrap_or(
                source_path
                    .file_name()
                    .map(std::path::Path::new)
                    .unwrap_or(source_path),
            );
            let mut output_path = cli.output_dir.join(rel);
            let mut name = output_path.file_name().unwrap_or_default().to_os_string();
            name.push(".json");
            output_path.set_file_name(name);
            (source_path.clone(), output_path)
        })
        .collect();

    // Create all required output directories up front (deduped).
    let mut dirs: Vec<&Path> = pairs.iter().filter_map(|(_, out)| out.parent()).collect();
    dirs.sort_unstable();
    dirs.dedup();
    for dir in dirs {
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!(
                "error: cannot create output directory '{}': {}",
                dir.display(),
                e
            );
            std::process::exit(1);
        }
    }

    let had_error = Arc::new(AtomicBool::new(false));

    pairs.par_iter().for_each(|(source_path, output_path)| {
        if let Err(e) = parse_file(source_path, output_path, cli.pretty_print) {
            eprintln!("warning: {}", e);
            had_error.store(true, Ordering::Relaxed);
        }
    });

    if had_error.load(Ordering::Relaxed) {
        std::process::exit(1);
    }
}
