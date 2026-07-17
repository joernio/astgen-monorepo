use crate::ast::build_ast_node;
use std::cell::RefCell;
use std::fs;
use std::io::BufWriter;
use std::path::Path;

thread_local! {
    static PARSER: RefCell<tree_sitter::Parser> = RefCell::new({
        let mut p = tree_sitter::Parser::new();
        p.set_language(&ts_parser_perl::LANGUAGE.into())
            .expect("load perl language");
        p
    });
}

/// Output directory must already exist; caller is responsible for creating it.
pub fn parse_file(
    source_path: &Path,
    output_path: &Path,
    pretty_print: bool,
) -> Result<(), String> {
    let source =
        fs::read(source_path).map_err(|e| format!("read {}: {}", source_path.display(), e))?;

    let tree = PARSER
        .with(|p| p.borrow_mut().parse(&source, None))
        .ok_or_else(|| format!("parse failed for {}", source_path.display()))?;

    let root = build_ast_node(tree.root_node(), &source);

    let file = fs::File::create(output_path)
        .map_err(|e| format!("create {}: {}", output_path.display(), e))?;
    let writer = BufWriter::new(file);
    if pretty_print {
        serde_json::to_writer_pretty(writer, &root)
    } else {
        serde_json::to_writer(writer, &root)
    }
    .map_err(|e| format!("serialize {}: {}", output_path.display(), e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parse_file_writes_valid_json() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("hello.pl");
        fs::write(&src, b"print \"hello\";").unwrap();
        let out_dir = dir.path().join("out");
        fs::create_dir_all(&out_dir).unwrap();
        let out = out_dir.join("hello.pl.json");
        parse_file(&src, &out, false).expect("parse_file should succeed");
        let json_str = fs::read_to_string(&out).expect("output file should exist");
        let val: serde_json::Value =
            serde_json::from_str(&json_str).expect("output should be valid JSON");
        assert_eq!(val["node_type"], "source_file");
        assert!(val["children"].is_array());
        assert!(val["text"].is_string(), "root node must have text field");
    }

    #[test]
    fn parse_file_returns_error_for_missing_input() {
        let dir = tempfile::tempdir().unwrap();
        let result = parse_file(
            &dir.path().join("nonexistent.pl"),
            &dir.path().join("out.json"),
            false,
        );
        assert!(result.is_err());
    }
}
