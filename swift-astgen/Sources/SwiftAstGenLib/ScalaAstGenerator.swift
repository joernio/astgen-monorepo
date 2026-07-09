import CodeGeneration
import Foundation

/// Trims `value` and re-indents continuation lines with a single tab.
///
/// Used to embed multi-line documentation strings inside the generated Scala output without
/// breaking the surrounding indentation.
private func indented(_ value: Any) -> String {
    String(describing: value)
        .trimmingCharacters(in: .whitespacesAndNewlines)
        .replacingOccurrences(of: "\n", with: "\n\t")
}

/// Generates a Scala source file (`SwiftNodeSyntax.scala` by default) describing the entire
/// SwiftSyntax node hierarchy.
///
/// The generated file is consumed by downstream Scala code (notably the Joern Swift frontend)
/// to navigate AST JSON produced by ``SwiftAstGenerator``. Output is rendered with tab
/// indentation, which is part of the format Joern expects.
///
/// > Note: The header includes a human-readable timestamp; the rest of the file is fully
/// > deterministic for a given SwiftSyntax version.
public final class ScalaAstGenerator {

    /// Default location of the generated Scala source file (CWD-relative).
    public static let defaultOutputPath = "./SwiftNodeSyntax.scala"

    private let outputUrl: URL

    /// Creates a new generator.
    /// - Parameter outputUrl: Destination of the generated Scala file. Defaults to
    ///   ``defaultOutputPath`` in the current working directory.
    public init(outputUrl: URL = URL(fileURLWithPath: ScalaAstGenerator.defaultOutputPath)) {
        self.outputUrl = outputUrl
    }

    /// Generates `SwiftNodeSyntax.scala` and writes it to ``outputUrl``.
    /// - Throws: Any I/O error encountered while writing the file.
    public func generate() throws {
        let output = renderScalaSource()
        try output.write(to: outputUrl, atomically: true, encoding: .utf8)
        Log.info("Generated Scala Swift AST in file: `\(outputUrl.path)`")
    }

    // MARK: - Top-level rendering

    private func renderScalaSource() -> String {
        let baseNodes = renderBaseNodes()
        let traits = renderTraits()
        let syntaxNodes = NON_BASE_SYNTAX_NODES.map(renderSyntaxNode(_:))
        let tokens = renderTokens()
        let nodeTypeMap = renderNodeTypeMap()
        let tokenKindMap = renderTokenKindMap()

        return """
            \(header())

            object SwiftNodeSyntax {

            \tdef createSwiftNode(json: Value): SwiftNode = {
            \t\tval nodeType = json("nodeType").str
            \t\tval tokenKind = json("tokenKind").str

            \t\tif (nodeType.nonEmpty) {
            \t\t\t_nodeTypeMap.getOrElse(nodeType, throw new UnsupportedOperationException(s"NodeType '$nodeType' is not a known Swift NodeType!"))(json)
            \t\t} else if (tokenKind.nonEmpty) {
            \t\t\tval prefix = { val parenIndex = tokenKind.indexOf('('); if (parenIndex >= 0) tokenKind.substring(0, parenIndex) else tokenKind }
            \t\t\t_tokenKindMap.getOrElse(prefix, throw new UnsupportedOperationException(s"TokenKind '$tokenKind' is not a known Swift TokenKind!"))(json)
            \t\t} else {
            \t\t\tthrow new UnsupportedOperationException("Invalid SwiftSyntax json element. 'nodeType' and 'tokenKind' cannot be empty at the same time!")
            \t\t}
            \t}

            \tprivate val _nodeTypeMap: Map[String, Value => SwiftNode] = Map(
            \t\t\(nodeTypeMap)
            \t)

            \tprivate val _tokenKindMap: Map[String, Value => SwiftNode] = Map(
            \t\t\(tokenKindMap)
            \t)

            \tsealed trait SwiftNode {
            \t\tdef json: Value

            \t\tprotected lazy val _childrenMap: Map[String, Value] = {
            \t\t\tjson.obj.get("children") match {
            \t\t\t\tcase Some(ch) => ch.arr.iterator.flatMap(child => child.obj.get("name").map(_.str -> child)).toMap
            \t\t\t\tcase None => Map.empty
            \t\t\t}
            \t\t}

            \t\tprivate lazy val _rangeObj = json.obj.get("range").map(_.obj)
            \t\tprivate def _rangeField(name: String): Option[Int] = _rangeObj.flatMap(_.get(name)).map(_.num.toInt)

            \t\tdef startOffset: Option[Int] = _rangeField("startOffset")
            \t\tdef endOffset: Option[Int] = _rangeField("endOffset")
            \t\tdef startLine: Option[Int] = _rangeField("startLine")
            \t\tdef startColumn: Option[Int] = _rangeField("startColumn")
            \t\tdef endLine: Option[Int] = _rangeField("endLine")
            \t\tdef endColumn: Option[Int] = _rangeField("endColumn")

            \t\toverride lazy val toString: String = this.getClass.getSimpleName.stripSuffix("$")
            \t}

            \tsealed trait SwiftToken extends SwiftNode

            \t// MARK: tokens:
            \t\(tokens.joined(separator: "\n\t"))

            \t// MARK: base nodes:
            \t\(baseNodes.joined(separator: "\n\t"))

            \t// MARK: marker traits:
            \t\(traits.joined(separator: "\n\t"))

            \t// MARK: syntax nodes:
            \t\(syntaxNodes.joined(separator: "\n\t"))

            \tcase class TokenSyntax(json: Value) extends Syntax

            }
            """
    }

    // MARK: - Header

    private func header() -> String {
        """
        // Automatically generated by 'SwiftAstGen --scala-ast-only'.
        // Do not edit directly!
        // Generated: \(dateString())

        import ujson.Value
        """
    }

    private func dateString() -> String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        // Explicit format avoids locale-inserted narrow no-break spaces around AM/PM on recent OSes.
        formatter.dateFormat = "MMMM d, yyyy 'at' h:mm:ss a zzz"
        return formatter.string(from: Date())
    }

    // MARK: - Pieces

    private func renderBaseNodes() -> [String] {
        let names = Set(SYNTAX_NODES.map(baseTypeName(of:))).sorted()
        return names.map { "sealed trait \($0) extends SwiftNode" }
    }

    private func renderTraits() -> [String] {
        TRAITS.map { trait in
            if trait.documentation.isEmpty {
                return "sealed trait \(trait.traitName)"
            }
            return """
                \n\t\(indented(trait.documentation))
                \tsealed trait \(trait.traitName)
                """
        }
    }

    private func renderTokens() -> [String] {
        Token.allCases.map { "case class \($0)(json: Value) extends SwiftToken" }
    }

    private func renderNodeTypeMap() -> String {
        let entries = NON_BASE_SYNTAX_NODES.map { node -> String in
            let syntaxType = node.kind.syntaxType
            return "\"\(syntaxType)\" -> (json => \(syntaxType)(json))"
        }
        let withTokenSyntax = entries + ["\"TokenSyntax\" -> (json => TokenSyntax(json))"]
        return withTokenSyntax.joined(separator: ",\n\t\t")
    }

    private func renderTokenKindMap() -> String {
        Token.allCases
            .map { "\"\($0)\" -> (json => \($0)(json))" }
            .joined(separator: ",\n\t\t")
    }

    // MARK: - Per-node rendering

    private func renderSyntaxNode(_ node: Node) -> String {
        let syntaxType = node.kind.syntaxType
        let inheritsString = "extends \(inheritsFrom(node).joined(separator: " with "))"
        let childrenString = renderChildren(of: node)
        let docString = renderDocString(for: node)

        return """
            \(docString)
            \tcase class \(syntaxType)(json: Value) \(inheritsString) {
            \t\(childrenString)
            \t}
            """
    }

    private func renderChildren(of node: Node) -> String {
        let layoutChildren = node.layoutNode?.children ?? []
        if !layoutChildren.isEmpty {
            let lines =
                layoutChildren
                .filter { !$0.isUnexpectedNodes }
                .map(renderLayoutChild(_:))
            return lines.joined(separator: "\n\t")
        }

        guard let collection = node.collectionNode else {
            // A node has either layout children or is a collection; if neither, the upstream
            // SwiftSyntax metadata is malformed. Leave a marker rather than crashing.
            return "\t// no children available"
        }
        let elementType = TypeGenerator.collectionElementType(for: collection)
        // `lazy val` (not `def`): each accessor builds a fresh wrapper and re-materializes the whole
        // child Seq, so re-accessing `.children` on the same instance would repeat that work. Caching is
        // safe because a node wrapper (and the JSON tree it navigates) is confined to the single thread
        // that builds the AST for one file.
        return
            "\tlazy val children: Seq[\(elementType)] = "
            + "json(\"children\").arr.iterator.map(child => createSwiftNode(child).asInstanceOf[\(elementType)]).toSeq"
    }

    private func renderLayoutChild(_ child: Child) -> String {
        let varName = lowercaseFirstWord(name: child.name)
        let name = backtickedIfNeeded(name: varName)
        let childType = TypeGenerator.type(for: child)
        // `lazy val` (not `def`): each accessor allocates a fresh wrapper and, on the second field access,
        // rebuilds `_childrenMap`. Caching the wrapper makes re-access free. Safe because a node wrapper is
        // confined to the single thread that builds the AST for one file (see `renderChildren`).
        if child.isOptional {
            return
                "\tlazy val \(name): Option[\(childType)] = "
                + "_childrenMap.get(\"\(varName)\").map(child => createSwiftNode(child).asInstanceOf[\(childType)])"
        } else {
            return
                "\tlazy val \(name): \(childType) = "
                + "createSwiftNode(_childrenMap(\"\(varName)\")).asInstanceOf[\(childType)]"
        }
    }

    private func renderDocString(for node: Node) -> String {
        let indentedDoc = indented(node.documentation)
        let documentation = indentedDoc.isEmpty ? "/// No documentation available." : indentedDoc

        let childrenDoc =
            node.layoutNode?.grammar ?? node.collectionNode?.grammar ?? "/// no children available"
        let childrenDocString = indented(childrenDoc)

        let containedInDocString = indented(node.containedIn)

        return """
            \n\t/**
            \t/// ### Documentation
            \t///
            \t\(documentation)
            \t///
            \t\(childrenDocString)
            \t///
            \t\(containedInDocString.isEmpty ? "/// ### Nowhere contained in" : containedInDocString)
            \t */
            """
            .replacingOccurrences(of: "///", with: " *")
            .replacingOccurrences(of: "```swift", with: "{{{")
            .replacingOccurrences(of: "```", with: "}}}")
    }

    // MARK: - Helpers

    private func baseTypeName(of node: Node) -> String {
        String(describing: node.base.syntaxType)
    }

    private func inheritsFrom(_ node: Node) -> [String] {
        let base = baseTypeName(of: node)
        let traits = node.layoutNode?.traits ?? []
        return [base] + traits
    }

    private func backtickedIfNeeded(name: String) -> String {
        name == "type" ? "`\(name)`" : name
    }
}
