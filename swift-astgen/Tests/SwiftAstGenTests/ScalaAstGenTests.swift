import Foundation
import XCTest

@testable import class SwiftAstGenLib.ScalaAstGenerator

final class ScalaAstGenTests: XCTestCase, TestUtils {

    func testScalaSourceFileOutput() throws {
        let tempDir = try createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        let scalaOutFileUrl = tempDir.appendingPathComponent("SwiftNodeSyntax.scala")
        try ScalaAstGenerator(outputUrl: scalaOutFileUrl).generate()

        XCTAssertTrue(FileManager.default.fileExists(atPath: scalaOutFileUrl.path))
        let content = try String(contentsOf: scalaOutFileUrl, encoding: .utf8)
        XCTAssertTrue(content.contains("object SwiftNodeSyntax {"))
        XCTAssertTrue(content.contains("sealed trait SwiftNode"))
        XCTAssertTrue(content.contains("case class TokenSyntax(json: Value) extends Syntax"))
    }

    func testGeneratedScalaIsDeterministicAcrossRunsExceptHeader() throws {
        let tempDir = try createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        let firstUrl = tempDir.appendingPathComponent("first.scala")
        let secondUrl = tempDir.appendingPathComponent("second.scala")

        try ScalaAstGenerator(outputUrl: firstUrl).generate()
        try ScalaAstGenerator(outputUrl: secondUrl).generate()

        let first = try String(contentsOf: firstUrl, encoding: .utf8)
        let second = try String(contentsOf: secondUrl, encoding: .utf8)

        // Strip the "// Generated: ..." line which intentionally embeds the current time.
        let firstStripped = stripGeneratedLine(first)
        let secondStripped = stripGeneratedLine(second)

        XCTAssertEqual(firstStripped, secondStripped)
    }

    private func stripGeneratedLine(_ source: String) -> String {
        source
            .split(separator: "\n", omittingEmptySubsequences: false)
            .filter { !$0.hasPrefix("// Generated:") }
            .joined(separator: "\n")
    }
}
