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
      children: childrenNodes)
  }
}

struct SyntaxParser {

  static func parse(
    srcDir: URL,
    fileUrl: URL,
    relativeFilePath: String,
    prettyPrint: Bool
  ) throws -> String {
    let code = try String(contentsOf: fileUrl)
    let ast = Parser.parse(source: code)

    let locationConverter = SourceLocationConverter(fileName: fileUrl.path, tree: ast)
    let treeNode = ast.toJson(converter: locationConverter)

    treeNode.projectFullPath = srcDir.path
    treeNode.relativeFilePath = relativeFilePath
    treeNode.fullFilePath = fileUrl.path

    let encoder = JSONEncoder()
    if prettyPrint { encoder.outputFormatting = .prettyPrinted }
    return String(decoding: try encoder.encode(treeNode), as: UTF8.self)
  }

}
