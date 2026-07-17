use serde::Serialize;

#[derive(Serialize)]
pub struct Point {
    pub row: usize,
    pub column: usize,
}

#[derive(Serialize)]
pub struct AstNode {
    pub node_type: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_position: Point,
    pub end_position: Point,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_name: Option<String>,
    pub children: Vec<AstNode>,
}

pub fn build_ast_node(node: tree_sitter::Node, source_bytes: &[u8]) -> AstNode {
    build_ast_node_with_field(node, source_bytes, None)
}

fn build_ast_node_with_field(
    node: tree_sitter::Node,
    source_bytes: &[u8],
    field_name: Option<String>,
) -> AstNode {
    let children: Vec<AstNode> = {
        let mut cursor = node.walk();
        node.children(&mut cursor)
            .enumerate()
            .filter_map(|(i, child)| {
                let fname = node.field_name_for_child(i as u32).map(str::to_owned);
                if child.is_named() || fname.is_some() {
                    Some(build_ast_node_with_field(child, source_bytes, fname))
                } else {
                    None
                }
            })
            .collect()
    };

    let text = node.utf8_text(source_bytes).ok().map(|s| s.to_owned());
    let sp = node.start_position();
    let ep = node.end_position();

    AstNode {
        node_type: node.kind().to_owned(),
        start_byte: node.start_byte(),
        end_byte: node.end_byte(),
        start_position: Point {
            row: sp.row,
            column: sp.column,
        },
        end_position: Point {
            row: ep.row,
            column: ep.column,
        },
        text,
        field_name,
        children,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_nodes_have_text() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&ts_parser_perl::LANGUAGE.into())
            .unwrap();
        let source = b"print(\"Hello World\\n\");";
        let tree = parser.parse(source, None).unwrap();
        let root = build_ast_node(tree.root_node(), source);
        assert_eq!(root.node_type, "source_file");
        assert!(root.text.is_some());
        fn check(node: &AstNode) {
            assert!(node.text.is_some(), "node {} missing text", node.node_type);
            for child in &node.children {
                check(child);
            }
        }
        check(&root);
    }

    #[test]
    fn binary_expression_children_have_field_names() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&ts_parser_perl::LANGUAGE.into())
            .unwrap();
        let source = b"my $x = 1 + 2;";
        let tree = parser.parse(source, None).unwrap();
        let root = build_ast_node(tree.root_node(), source);
        fn find_binary(node: &AstNode) -> Option<&AstNode> {
            if node.node_type == "binary_expression" {
                return Some(node);
            }
            node.children.iter().find_map(find_binary)
        }
        let bin = find_binary(&root).expect("binary_expression not found");
        let field_names: Vec<Option<&str>> = bin
            .children
            .iter()
            .map(|c| c.field_name.as_deref())
            .collect();
        assert!(
            field_names.contains(&Some("left")),
            "expected 'left' field, got {:?}",
            field_names
        );
        assert!(
            field_names.contains(&Some("right")),
            "expected 'right' field, got {:?}",
            field_names
        );
        assert!(
            field_names.contains(&Some("operator")),
            "expected 'operator' token field, got {:?}",
            field_names
        );
    }

    #[test]
    fn token_field_child_has_text() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&ts_parser_perl::LANGUAGE.into())
            .unwrap();
        let source = b"my $x = 1 + 2;";
        let tree = parser.parse(source, None).unwrap();
        let root = build_ast_node(tree.root_node(), source);
        fn find_binary(node: &AstNode) -> Option<&AstNode> {
            if node.node_type == "binary_expression" {
                return Some(node);
            }
            node.children.iter().find_map(find_binary)
        }
        let bin = find_binary(&root).expect("binary_expression not found");
        let op = bin
            .children
            .iter()
            .find(|c| c.field_name.as_deref() == Some("operator"))
            .expect("operator child not found");
        assert!(op.text.is_some(), "operator token child must have text");
        assert_eq!(op.text.as_deref(), Some("+"));
    }
}
