import Foundation

import SwiftParser

/// Visitor that extracts test target information from Package.swift
import SwiftSyntax

private class TestTargetVisitor: SyntaxVisitor {
    var testTargetPaths: [String] = []

    override func visit(_ node: FunctionCallExprSyntax) -> SyntaxVisitorContinueKind {
        // Look for .testTarget(...) function calls
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

        // Iterate through the labeled arguments
        for argument in functionCall.arguments {
            guard let label = argument.label?.text else { continue }

            switch label {
            case "name":
                // Extract the string literal value
                if let stringExpr = argument.expression.as(StringLiteralExprSyntax.self),
                    let segment = stringExpr.segments.first?.as(StringSegmentSyntax.self)
                {
                    name = segment.content.text
                }
            case "path":
                // Extract the string literal value for path
                if let stringExpr = argument.expression.as(StringLiteralExprSyntax.self),
                    let segment = stringExpr.segments.first?.as(StringSegmentSyntax.self)
                {
                    path = segment.content.text
                }
            default:
                break
            }
        }

        // If path is explicitly specified, use it; otherwise, use Tests/{name}
        if let path = path {
            testTargetPaths.append(path)
        } else if let name = name {
            testTargetPaths.append("Tests/\(name)")
        }
    }
}
public class PackageTestTargetParser {

    private let srcDir: URL

    public init(srcDir: URL) {
        self.srcDir = srcDir
    }

    /// Returns a list of all testTarget paths found in the Package.swift file at srcDir
    public func getTestTargetPaths() -> [String] {
        let packageSwiftUrl = srcDir.appendingPathComponent("Package.swift")

        guard FileManager.default.fileExists(atPath: packageSwiftUrl.path) else {
            return []
        }

        do {
            let content = try String(contentsOf: packageSwiftUrl, encoding: .utf8)
            return parseTestTargets(from: content)
        } catch {
            return []
        }
    }

    private func parseTestTargets(from content: String) -> [String] {
        // Parse the Swift source code using SwiftParser
        let sourceFile = Parser.parse(source: content)

        // Create a visitor and walk the syntax tree
        let visitor = TestTargetVisitor(viewMode: .sourceAccurate)
        visitor.walk(sourceFile)

        return visitor.testTargetPaths
    }
}
