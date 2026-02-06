import Foundation

import SwiftOperators

import SwiftParser

@_spi(RawSyntax) import SwiftSyntax

extension SyntaxProtocol {
    internal func toJson(converter: SourceLocationConverter) -> TreeNode {
        var tokenKind = ""
        var nodeType = ""
        if let token = Syntax(self).as(TokenSyntax.self) {
            tokenKind = String(describing: token.tokenKind)
        } else {
            nodeType = String(describing: syntaxNodeType)
        }

        let sourceRange = sourceRange(converter: converter)
        let rangeNode = Range(
            startOffset: sourceRange.start.offset,
            endOffset: sourceRange.end.offset,
            startLine: sourceRange.start.line,
            startColumn: sourceRange.start.column,
            endLine: sourceRange.end.line,
            endColumn: sourceRange.end.column
        )

        let allChildren = children(viewMode: .fixedUp)
        var childrenNodes: [TreeNode] = []

        for (num, child) in allChildren.enumerated() {
            var name = ""
            var index = -1
            if let keyPath = child.keyPathInParent, let cname = childName(keyPath) {
                name = cname
            } else if self.kind.isSyntaxCollection {
                index = num
            }
            let childNode = child.toJson(converter: converter)
            childNode.name = name
            childNode.index = index
            childrenNodes.append(childNode)
        }

        return TreeNode(
            tokenKind: tokenKind,
            nodeType: nodeType,
            range: rangeNode,
            children: childrenNodes
        )
    }
}
struct SyntaxParser {

    /// Counts the number of lines in a given string, handling all common line endings (\n, \r\n, \r) in a platform-independent way.
    /// - Parameter text: The input string to count lines in.
    /// - Returns: The number of lines in the string.
    static func countLines(in text: String) -> Int {
        // Use CharacterSet.newlines which matches \n, \r, \r\n, Unicode line/paragraph separators, etc.
        // Split omitting empty subsequences to correctly handle trailing newlines.
        let lines = text.split(omittingEmptySubsequences: false, whereSeparator: { $0.isNewline })
        return lines.count
    }

    static func parse(
        srcDir: URL,
        fileUrl: URL,
        relativeFilePath: String,
        prettyPrint: Bool
    ) throws -> String {
        let code = try String(contentsOf: fileUrl)
        let loc = countLines(in: code)
        let opPrecedence = OperatorTable.standardOperators
        let ast = Parser.parse(source: code)
        let folded = try opPrecedence.foldAll(ast)

        let locationConverter = SourceLocationConverter(fileName: fileUrl.path, tree: folded)
        let treeNode = folded.toJson(converter: locationConverter)

        treeNode.projectFullPath = srcDir.standardized.resolvingSymlinksInPath().path
        treeNode.fullFilePath = fileUrl.standardized.resolvingSymlinksInPath().path
        treeNode.relativeFilePath = relativeFilePath
        treeNode.content = code
        treeNode.loc = loc

        let encoder = JSONEncoder()
        if prettyPrint { encoder.outputFormatting = .prettyPrinted }
        return String(decoding: try encoder.encode(treeNode), as: UTF8.self)
    }

}
