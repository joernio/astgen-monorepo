use anyhow::{Context, Result};
use clap::Parser;
use heck::{ToPascalCase, ToShoutySnakeCase};
use rust_ast_gen::grammar::model::Model;
use rust_ast_gen::scala_gen::config::ScalaAstGenConfig;
use rust_ast_gen::scala_gen::emitter::generate_scala;
use std::path::PathBuf;
use std::str::FromStr;
use ungrammar::Grammar;

fn main() -> Result<()> {
    let args = ScalaBindingsGenArgs::parse();
    let grammar_text = include_str!("../../rust.ungram");
    let grammar = Grammar::from_str(grammar_text)?;
    let model = Model::from_ungrammar(&grammar)?;

    let codegen_version = env!("CARGO_PKG_VERSION").to_string();
    let codegen_date = args.include_date.then(|| {
        chrono::Local::now()
            .format("%d %B %Y, %H:%M:%S %Z")
            .to_string()
    });
    let package_name = "io.joern.rust2cpg.parser".to_string();
    let object_name = "RustNodeSyntax".to_string();
    let base_node_trait = "RustNode".to_string();
    let base_token_trait = "RustToken".to_string();
    let trait_nodes = vec![
        // TODO: Add more as needed
        "Expr".to_string(),
        "Type".to_string(),
        "Stmt".to_string(),
        "Item".to_string(),
        "Meta".to_string(),
        "Pat".to_string(),
    ];
    let config = ScalaAstGenConfig {
        package_name,
        object_name,
        base_node_trait,
        base_token_trait,
        trait_nodes,
        node_name_to_scala_name,
        node_name_to_json_kind,
        token_name_to_scala_name,
        token_name_to_json_kind,
        codegen_version,
        codegen_date,
    };

    let scala_output = generate_scala(&model, &config)?;

    std::fs::write(&args.output_file_path, &scala_output)
        .with_context(|| format!("failed to write to {}", args.output_file_path.display()))?;

    eprintln!(
        "wrote {} bytes to {}",
        scala_output.len(),
        args.output_file_path.display()
    );

    Ok(())
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
    format!("{}", node)
}

/// This one is important, as it MUST match the nodeKind in the JSON representation.
fn node_name_to_json_kind(node: &str) -> String {
    node.to_shouty_snake_case()
}

fn token_operator_or_punct_to_json_kind(token: &str) -> Option<&'static str> {
    match token {
        ";" => Some("SEMICOLON"),
        "," => Some("COMMA"),
        "(" => Some("L_PAREN"),
        ")" => Some("R_PAREN"),
        "{" => Some("L_CURLY"),
        "}" => Some("R_CURLY"),
        "[" => Some("L_BRACK"),
        "]" => Some("R_BRACK"),
        "<" => Some("L_ANGLE"),
        ">" => Some("R_ANGLE"),
        "@" => Some("AT"),
        "#" => Some("POUND"),
        "~" => Some("TILDE"),
        "?" => Some("QUESTION"),
        "&" => Some("AMP"),
        "|" => Some("PIPE"),
        "+" => Some("PLUS"),
        "*" => Some("STAR"),
        "/" => Some("SLASH"),
        "^" => Some("CARET"),
        "%" => Some("PERCENT"),
        "_" => Some("UNDERSCORE"),
        "." => Some("DOT"),
        ".." => Some("DOT2"),
        "..." => Some("DOT3"),
        "..=" => Some("DOT2EQ"),
        ":" => Some("COLON"),
        "::" => Some("COLON2"),
        "=" => Some("EQ"),
        "==" => Some("EQ2"),
        "=>" => Some("FAT_ARROW"),
        "!" => Some("BANG"),
        "!=" => Some("NEQ"),
        "-" => Some("MINUS"),
        "->" => Some("THIN_ARROW"),
        "<=" => Some("LTEQ"),
        ">=" => Some("GTEQ"),
        "+=" => Some("PLUSEQ"),
        "-=" => Some("MINUSEQ"),
        "|=" => Some("PIPEEQ"),
        "&=" => Some("AMPEQ"),
        "^=" => Some("CARETEQ"),
        "/=" => Some("SLASHEQ"),
        "*=" => Some("STAREQ"),
        "%=" => Some("PERCENTEQ"),
        "&&" => Some("AMP2"),
        "||" => Some("PIPE2"),
        "<<" => Some("SHL"),
        ">>" => Some("SHR"),
        "<<=" => Some("SHLEQ"),
        ">>=" => Some("SHREQ"),
        _ => None,
    }
}

/// This one is important, as it MUST match the nodeKind in the JSON representation.
/// TODO: Currently manual. See how it could be automated.
fn token_name_to_json_kind(token: &str) -> String {
    if let Some(json_kind) = token_operator_or_punct_to_json_kind(token) {
        return json_kind.to_string();
    }

    // #ident  -> IDENT
    if let Some(inner) = token.strip_prefix('#') {
        return inner.to_shouty_snake_case();
    }

    // @int_number -> INT_NUMBER
    if let Some(inner) = token.strip_prefix('@') {
        return inner.to_shouty_snake_case();
    }

    // Self is a special case keyword
    if token == "Self" {
        return "SELF_TYPE_KW".to_string();
    }

    // Any other keyword, fn -> FN_KW
    format!("{}_KW", token.to_shouty_snake_case())
}

fn token_name_to_scala_name(token: &str) -> String {
    // Suffix token to prevent e.g. `String` from conflicting with Scala's `String` type.
    format!("{}Token", token_name_to_json_kind(token)).to_pascal_case()
}
