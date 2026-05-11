import Foundation
import SwiftOperators
import SwiftParser

// `childName(_:)` is exposed under the `RawSyntax` SPI; it is needed to retrieve the
// property name a child node is stored under in its parent layout node.
@_spi(RawSyntax) import SwiftSyntax

extension SyntaxProtocol {
    /// Recursively converts this syntax node (and its children) into a ``TreeNode`` suitable
    /// for JSON serialization.
    ///
    /// - Parameters:
    ///   - name: The keyPath name under which this node is stored in its parent. Empty for
    ///     collection children and the root node.
    ///   - index: Position of this child within a syntax collection. `-1` for non-collection
    ///     children and the root node.
    ///   - converter: Used to map syntax positions to file/line/column information.
    fileprivate func toTreeNode(
        name: String = "",
        index: Int = -1,
        converter: SourceLocationConverter
    ) -> TreeNode {
        var tokenKind = ""
        var nodeType = ""
        if let token = Syntax(self).as(TokenSyntax.self) {
            tokenKind = String(describing: token.tokenKind)
        } else {
            nodeType = String(describing: syntaxNodeType)
        }

        let sourceRange = sourceRange(converter: converter)
        let range = SourceRange(
            startOffset: sourceRange.start.offset,
            endOffset: sourceRange.end.offset,
            startLine: sourceRange.start.line,
            startColumn: sourceRange.start.column,
            endLine: sourceRange.end.line,
            endColumn: sourceRange.end.column
        )

        let isCollection = self.kind.isSyntaxCollection
        let childrenNodes: [TreeNode] = children(viewMode: .fixedUp).enumerated().map { (num, child) in
            var resolvedName = ""
            var resolvedIndex = -1
            if let keyPath = child.keyPathInParent, let name = childName(keyPath) {
                resolvedName = name
            } else if isCollection {
                resolvedIndex = num
            }
            return child.toTreeNode(name: resolvedName, index: resolvedIndex, converter: converter)
        }

        return TreeNode(
            index: index,
            name: name,
            tokenKind: tokenKind,
            nodeType: nodeType,
            range: range,
            children: childrenNodes
        )
    }
}

/// Parses a single Swift source file into a ``TreeNode`` and serializes it as JSON.
enum SyntaxParser {

    /// Counts the number of lines in a given string, handling all common line endings
    /// (`\n`, `\r\n`, `\r`) in a platform-independent way.
    /// - Parameter text: The input string to count lines in.
    /// - Returns: The number of lines in the string.
    static func countLines(in text: String) -> Int {
        // `Character.isNewline` matches \n, \r, \r\n, and Unicode line/paragraph separators.
        // Keep empty subsequences so trailing newlines are counted correctly.
        let lines = text.split(omittingEmptySubsequences: false, whereSeparator: { $0.isNewline })
        return lines.count
    }

    /// Reads `fileUrl`, parses it with `SwiftParser` (with operator folding applied),
    /// and returns its JSON-encoded ``TreeNode`` representation.
    ///
    /// - Parameters:
    ///   - srcDir: Project root, embedded in the root node's `projectFullPath`.
    ///   - fileUrl: Absolute path of the source file to parse.
    ///   - relativeFilePath: Path of `fileUrl` relative to `srcDir`, embedded in the root node.
    ///   - prettyPrint: If `true`, the encoded JSON is pretty-printed.
    /// - Returns: UTF-8 JSON data ready to be written to disk.
    /// - Throws: Any error from reading or encoding the file.
    static func parse(
        srcDir: URL,
        fileUrl: URL,
        relativeFilePath: String,
        prettyPrint: Bool
    ) throws -> Data {
        let code = try String(contentsOf: fileUrl)
        let loc = countLines(in: code)
        let opPrecedence = OperatorTable.standardOperators
        let ast = Parser.parse(source: code)
        let folded = opPrecedence.foldAll(ast) { _ in }

        let locationConverter = SourceLocationConverter(fileName: fileUrl.path, tree: folded)
        let rootNode = folded.toTreeNode(converter: locationConverter)
            .withRootMetadata(
                projectFullPath: srcDir.standardized.resolvingSymlinksInPath().path,
                relativeFilePath: relativeFilePath,
                fullFilePath: fileUrl.standardized.resolvingSymlinksInPath().path,
                content: code,
                loc: loc
            )

        let encoder = JSONEncoder()
        if prettyPrint { encoder.outputFormatting = .prettyPrinted }
        // JSONEncoder output is always UTF-8; return Data so callers can write directly without a round-trip through String.
        return try encoder.encode(rootNode)
    }
}
