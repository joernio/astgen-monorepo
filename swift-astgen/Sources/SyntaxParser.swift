import Foundation
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

    let allChildren = children(viewMode: .all)
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
      tokenKind: tokenKind, nodeType: nodeType, range: rangeNode, children: childrenNodes)
  }
}

struct SyntaxParser {

  static func parse(fileURL: URL, prettyPrint: Bool) throws -> String {
    let code = try String(contentsOf: fileURL)
    let sourceFile = Parser.parse(source: code)
    let syntax = Syntax(sourceFile)

    let locationConverter = SourceLocationConverter(fileName: fileURL.path, tree: sourceFile)
    let jsonNode = syntax.toJson(converter: locationConverter)

    let encoder = JSONEncoder()
    if prettyPrint { encoder.outputFormatting = .prettyPrinted }
    return String(decoding: try encoder.encode(jsonNode), as: UTF8.self)
  }

}
