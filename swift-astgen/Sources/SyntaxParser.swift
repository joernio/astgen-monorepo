import Foundation
import SwiftSyntax
import SwiftOperators
import SwiftParser

struct SyntaxParser {
  static func parse(fileURL: URL) throws -> String {
    let code = try String(contentsOf: fileURL)
    let sourceFile = Parser.parse(source: code)
    let syntax = Syntax(sourceFile)
    let visitor = TokenVisitor(
      locationConverter: SourceLocationConverter.init(fileName: fileURL.path, tree: sourceFile),
      showMissingTokens: false
    )

    _ = visitor.rewrite(syntax)

    let tree = visitor.tree
    let encoder = JSONEncoder()
    let json = String(decoding: try encoder.encode(tree), as: UTF8.self)
    return json
  }
}