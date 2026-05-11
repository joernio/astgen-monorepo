/// A serializable representation of a SwiftSyntax node, written out as a JSON document by ``SyntaxParser``.
///
/// Root-only metadata (`projectFullPath`, `relativeFilePath`, `fullFilePath`, `content`, `loc`)
/// is populated by ``SyntaxParser/parse(srcDir:fileUrl:relativeFilePath:prettyPrint:)`` for the
/// top-level node and left `nil` for child nodes. The remaining fields describe the node itself
/// and are present on every node in the tree.
struct TreeNode: Codable {

    /// Absolute path of the project (source) directory. Set on the root node only.
    let projectFullPath: String?

    /// Project-relative path of the source file. Set on the root node only.
    let relativeFilePath: String?

    /// Absolute path of the source file. Set on the root node only.
    let fullFilePath: String?

    /// Full source text of the file. Set on the root node only.
    let content: String?

    /// Line count of the source file. Set on the root node only.
    let loc: Int?

    /// Position of this child within a syntax collection, or `-1` for non-collection children.
    let index: Int

    /// Name of the keyPath under which this node is stored in its parent, or the empty string
    /// for collection children and the root node.
    let name: String

    /// Description of the underlying `TokenKind` if this node is a token, otherwise the empty string.
    let tokenKind: String

    /// Description of the SwiftSyntax node type (for example `SourceFileSyntax`),
    /// or the empty string for token nodes.
    let nodeType: String

    /// Source range covered by this node.
    let range: SourceRange

    /// Children of this node in source order.
    let children: [TreeNode]

    private enum CodingKeys: String, CodingKey {
        case projectFullPath
        case relativeFilePath
        case fullFilePath
        case content
        case loc
        case index
        case name
        case tokenKind
        case nodeType
        case range
        case children
    }

    init(
        index: Int = -1,
        name: String = "",
        tokenKind: String,
        nodeType: String,
        range: SourceRange,
        children: [TreeNode],
        projectFullPath: String? = nil,
        relativeFilePath: String? = nil,
        fullFilePath: String? = nil,
        content: String? = nil,
        loc: Int? = nil
    ) {
        self.index = index
        self.name = name
        self.tokenKind = tokenKind
        self.nodeType = nodeType
        self.range = range
        self.children = children
        self.projectFullPath = projectFullPath
        self.relativeFilePath = relativeFilePath
        self.fullFilePath = fullFilePath
        self.content = content
        self.loc = loc
    }

    /// Returns a copy of this node with the supplied root-only metadata populated.
    func withRootMetadata(
        projectFullPath: String,
        relativeFilePath: String,
        fullFilePath: String,
        content: String,
        loc: Int
    ) -> TreeNode {
        TreeNode(
            index: index,
            name: name,
            tokenKind: tokenKind,
            nodeType: nodeType,
            range: range,
            children: children,
            projectFullPath: projectFullPath,
            relativeFilePath: relativeFilePath,
            fullFilePath: fullFilePath,
            content: content,
            loc: loc
        )
    }
}

/// Inclusive start/exclusive end source range, in offsets and 1-based line/column pairs.
struct SourceRange: Codable {
    let startOffset: Int
    let endOffset: Int
    let startLine: Int
    let startColumn: Int
    let endLine: Int
    let endColumn: Int
}
