pub struct ScalaAstGenConfig {
    /// Scala package name, e.g. "io.joern.rust2cpg.parser"
    pub package_name: String,

    /// Wrapper object name, e.g. "RustNodeSyntax"
    pub object_name: String,

    /// Base node trait name, e.g. "RustNode"
    pub base_node_trait: String,

    /// Base token trait name, e.g. "RustToken"
    pub base_token_trait: String,

    /// Grammar node names that should become sealed traits instead of
    /// case classes, e.g. "Expr", "Stmt", etc.
    pub trait_nodes: Vec<String>,

    /// Converts a grammar node name to the JSON representation of the
    /// node, e.g. "BlockExpr" -> "BLOCK_EXPR", etc.
    pub node_name_to_json_kind: fn(&str) -> String,

    /// Converts a grammar node name to a Scala class name.
    pub node_name_to_scala_name: fn(&str) -> String,

    /// Converts a grammar token text to a Scala class name.
    /// E.g. "fn" -> "fnKw", "(" -> "leftParen", etc.
    pub token_name_to_scala_name: fn(&str) -> String,

    /// Converts a grammar token text to the JSON representation of the
    /// token, e.g. "fn" -> "FN_KW", "(" -> "L_PAREN", etc.
    pub token_name_to_json_kind: fn(&str) -> String,

    /// The time this code was generated.
    /// Only included in the header
    pub codegen_date: String,

    /// The version of the code generator.
    /// Only included in the header.
    pub codegen_version: String,
}
