use anyhow::{Context, Result};
use clap::Parser;
use heck::ToPascalCase;
use scala_bindings_gen::grammar::model::Model;
use scala_bindings_gen::json_kind::{
    node_name_to_syntax_kind, syntax_kind_to_json_name, token_name_to_syntax_kind,
};
use scala_bindings_gen::scala_gen::config::ScalaAstGenConfig;
use scala_bindings_gen::scala_gen::emitter::generate_scala;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::str::FromStr;
use ungrammar::Grammar;

fn main() -> Result<()> {
    let args = ScalaBindingsGenArgs::parse();
    let scala_output = generate_bindings(args.include_date)?;

    std::fs::write(&args.output_file_path, &scala_output)
        .with_context(|| format!("failed to write to {}", args.output_file_path.display()))?;

    eprintln!(
        "wrote {} bytes to {}",
        scala_output.len(),
        args.output_file_path.display()
    );

    Ok(())
}

fn generate_bindings(include_date: bool) -> Result<String> {
    let grammar_text = include_str!("../rust.ungram");
    let grammar = Grammar::from_str(grammar_text)?;
    let model = Model::from_ungrammar(&grammar)?;

    let codegen_version = env!("CARGO_PKG_VERSION").to_string();
    let codegen_date = include_date.then(|| {
        chrono::Local::now()
            .format("%d %B %Y, %H:%M:%S %Z")
            .to_string()
    });
    let package_name = "io.joern.rust2cpg.parser".to_string();
    let object_name = "RustNodeSyntax".to_string();
    let base_node_trait = "RustNode".to_string();
    let base_token_trait = "RustToken".to_string();
    let trait_nodes = vec![
        "Adt".to_string(),
        "AsmOperand".to_string(),
        "AsmPiece".to_string(),
        "AssocItem".to_string(),
        "CfgPredicate".to_string(),
        "Expr".to_string(),
        "ExternItem".to_string(),
        "FieldList".to_string(),
        "GenericArg".to_string(),
        "GenericParam".to_string(),
        "Item".to_string(),
        "Meta".to_string(),
        "Pat".to_string(),
        "Stmt".to_string(),
        "Type".to_string(),
        "UseBoundGenericArg".to_string(),
        "VariantDef".to_string(),
    ];
    // NB: there's a mismatch between `rust.ungram` and the auto-generated `SyntaxToken` from rust-analyzer.
    // The grammar says that `initializer:Expr` is mandatory, but the generated `LetStmt` sees it as optional.
    // In particular, `let x;` is indeed valid but wouldn't be accepted by the grammar.
    // It turns out mandatory nodes are generated as optional, cf.
    // https://github.com/rust-lang/rust-analyzer/blob/7c3fc8671f83f6e46305358b98354f0611ebb3cd/xtask/src/codegen/grammar.rs#L923
    // and https://github.com/rust-lang/rust-analyzer/blob/7c3fc8671f83f6e46305358b98354f0611ebb3cd/crates/syntax/src/ast/generated/nodes.rs#L923
    // Instead of following the same route (i.e. mandator -> optional), we demote only the cases
    // we care about. Otherwise, rust2cpg would have to always deal with optional nodes.

    let elements_demoted_to_optional =
        HashMap::from([("LetStmt".to_string(), HashSet::from(["Expr".to_string()]))]);
    let accessor_renames = HashMap::from([("type".to_string(), "typ".to_string())]);
    let config = ScalaAstGenConfig {
        package_name,
        object_name,
        base_node_trait,
        base_token_trait,
        trait_nodes,
        elements_demoted_to_optional,
        accessor_renames,
        node_name_to_scala_name,
        node_name_to_json_kind,
        token_name_to_scala_name,
        token_name_to_json_kind,
        codegen_version,
        codegen_date,
    };

    Ok(generate_scala(&model, &config)?)
}

#[derive(Parser)]
struct ScalaBindingsGenArgs {
    #[arg(help = "Output file path for the generated Scala file")]
    #[arg(short = 'o', long = "output")]
    output_file_path: PathBuf,

    #[arg(help = "Include the current date in the generated file header")]
    #[arg(default_value_t = true)]
    #[arg(action = clap::ArgAction::Set)]
    #[arg(long = "include-date")]
    include_date: bool,
}

fn node_name_to_scala_name(node: &str) -> String {
    node.to_string()
}

// We want to crash hard if there's any missing node.
fn node_name_to_json_kind(node: &str) -> String {
    let kind = node_name_to_syntax_kind(node)
        .unwrap_or_else(|| panic!("ungrammar node {node:?} has no SyntaxKind"));
    syntax_kind_to_json_name(kind)
}

// We want to crash hard if there's any missing node.
fn token_name_to_json_kind(token: &str) -> String {
    let kind = token_name_to_syntax_kind(token)
        .unwrap_or_else(|| panic!("ungrammar token {token:?} has no SyntaxKind"));
    syntax_kind_to_json_name(kind)
}

fn token_name_to_scala_name(token: &str) -> String {
    // Suffix token to prevent e.g. `String` from conflicting with Scala's `String` type.
    format!("{}Token", token_name_to_json_kind(token)).to_pascal_case()
}

#[cfg(test)]
mod tests {
    use super::generate_bindings;

    #[test]
    fn every_grammar_name_resolves_to_a_syntax_kind() {
        generate_bindings(false).unwrap();
    }
}
