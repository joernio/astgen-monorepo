final class TreeNode: Codable {

    var projectFullPath: String?
    var relativeFilePath: String?
    var fullFilePath: String?
    var content: String?
    var loc: Int?

    var index: Int
    var name: String
    var tokenKind: String
    var nodeType: String
    var range: Range
    var children: [TreeNode]

    init(
        tokenKind: String,
        nodeType: String,
        range: Range,
        children: [TreeNode]
    ) {
        self.index = -1
        self.name = ""
        self.tokenKind = tokenKind
        self.nodeType = nodeType
        self.range = range
        self.children = children
    }
}
struct Range: Codable {
    let startOffset: Int
    let endOffset: Int
    let startLine: Int
    let startColumn: Int
    let endLine: Int
    let endColumn: Int
}
