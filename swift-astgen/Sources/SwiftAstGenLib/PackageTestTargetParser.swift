import Foundation
import SwiftParser
import SwiftSyntax

/// Visitor that collects the on-disk paths of every `testTarget(...)` call in a SwiftPM manifest.
///
/// The visitor matches by simple member name (`.testTarget`) and recognizes the conventional
/// SwiftPM defaults: when no `path:` is provided, the path is assumed to be `Tests/<name>`.
/// Calls to `PackageDescription.testTarget(...)` (qualified with the module name) are not matched.
private final class TestTargetVisitor: SyntaxVisitor {
    var testTargetPaths: [String] = []

    override func visit(_ node: FunctionCallExprSyntax) -> SyntaxVisitorContinueKind {
        if let memberAccess = node.calledExpression.as(MemberAccessExprSyntax.self),
            memberAccess.declName.baseName.text == "testTarget"
        {
            extractTestTargetInfo(from: node)
        }
        return .visitChildren
    }

    private func extractTestTargetInfo(from functionCall: FunctionCallExprSyntax) {
        var name: String?
        var path: String?

        for argument in functionCall.arguments {
            guard let label = argument.label?.text else { continue }

            switch label {
            case "name":
                if let stringExpr = argument.expression.as(StringLiteralExprSyntax.self),
                    let segment = stringExpr.segments.first?.as(StringSegmentSyntax.self)
                {
                    name = segment.content.text
                }
            case "path":
                if let stringExpr = argument.expression.as(StringLiteralExprSyntax.self),
                    let segment = stringExpr.segments.first?.as(StringSegmentSyntax.self)
                {
                    path = segment.content.text
                }
            default:
                break
            }
        }

        if let path = path {
            testTargetPaths.append(path)
        } else if let name = name {
            testTargetPaths.append("Tests/\(name)")
        }
    }
}

/// Parses a SwiftPM `Package.swift` to discover the on-disk paths of its test targets.
///
/// Used by ``SwiftAstGenerator`` to skip test target sources during AST generation.
/// Only the manifest at the project root is inspected; nested SwiftPM packages are ignored.
public final class PackageTestTargetParser {

    private let srcDir: URL

    /// Creates a parser rooted at `srcDir`.
    /// - Parameter srcDir: Project root expected to contain a `Package.swift` manifest.
    public init(srcDir: URL) {
        self.srcDir = srcDir
    }

    /// Returns the list of test target paths declared in `srcDir/Package.swift`.
    ///
    /// - Returns: Project-relative paths (for example `Tests/MyPackageTests`). Returns an
    ///   empty array if the manifest is absent or cannot be read; in the latter case a
    ///   warning is logged to stderr.
    public func getTestTargetPaths() -> [String] {
        let packageSwiftUrl = srcDir.appendingPathComponent("Package.swift")

        guard FileManager.default.fileExists(atPath: packageSwiftUrl.path) else {
            return []
        }

        do {
            let content = try String(contentsOf: packageSwiftUrl, encoding: .utf8)
            return parseTestTargets(from: content)
        } catch {
            Log.warn("Could not read `\(packageSwiftUrl.path)` (\(error)); test targets will not be ignored.")
            return []
        }
    }

    private func parseTestTargets(from content: String) -> [String] {
        let sourceFile = Parser.parse(source: content)
        let visitor = TestTargetVisitor(viewMode: .sourceAccurate)
        visitor.walk(sourceFile)
        return visitor.testTargetPaths
    }
}
