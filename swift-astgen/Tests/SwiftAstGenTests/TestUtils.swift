import Foundation
import XCTest

@testable import struct SwiftAstGenLib.TreeNode

/// Shared helpers for `SwiftAstGenTests`.
///
/// Provides scratch-directory creation, JSON loading, and a `withCode { ... }` fixture that
/// owns the lifetime of a temporary source/output tree so individual tests stay focused on
/// behaviour.
protocol TestUtils {}

extension TestUtils where Self: XCTestCase {

    /// Creates a unique temporary directory and returns its URL.
    /// The directory is owned by the caller; pair with ``cleanup(directory:)`` (or `defer`).
    func createTemporaryDirectory(file: StaticString = #file, line: UInt = #line) throws -> URL {
        let tempDir = URL(fileURLWithPath: NSTemporaryDirectory())
            .appendingPathComponent(uniqueName(), isDirectory: true)
        try FileManager.default.createDirectory(
            atPath: tempDir.path,
            withIntermediateDirectories: true
        )
        return tempDir
    }

    /// Best-effort recursive removal; failures are deliberately ignored so cleanup never
    /// masks the original test failure.
    func cleanup(directory: URL) {
        try? FileManager.default.removeItem(at: directory)
    }

    /// Writes `content` to `<baseDir>/<path>`, creating intermediate directories as needed.
    func createFile(
        at baseDir: URL,
        path: String,
        content: String,
        file: StaticString = #file,
        line: UInt = #line
    ) throws {
        let fileUrl = baseDir.appendingPathComponent(path)
        let dirUrl = fileUrl.deletingLastPathComponent()
        try FileManager.default.createDirectory(
            atPath: dirUrl.path,
            withIntermediateDirectories: true
        )
        try content.write(to: fileUrl, atomically: true, encoding: .utf8)
    }

    /// Decodes a JSON file produced by `SwiftAstGenerator` as a ``TreeNode``.
    /// Throws (rather than returning `nil`) so the underlying decoding error surfaces in
    /// the test failure message.
    func loadJson(file url: URL) throws -> TreeNode {
        let data = try Data(contentsOf: url)
        return try JSONDecoder().decode(TreeNode.self, from: data)
    }

    /// Creates a temporary `srcDir`/`outputDir` pair seeded with a single `source.swift`,
    /// runs `body`, and cleans up the directory afterwards even if the test throws.
    ///
    /// - Parameters:
    ///   - code: Swift source written to `<srcDir>/source.swift`.
    ///   - body: Receives `(srcDir, outputDir, expected json file URL)`.
    func withCode(
        code: String,
        body: (_ srcDir: URL, _ outputDir: URL, _ jsonFile: URL) throws -> Void
    ) throws {
        let srcDir = URL(fileURLWithPath: NSTemporaryDirectory(), isDirectory: true)
            .appendingPathComponent(uniqueName(), isDirectory: true)
        defer { cleanup(directory: srcDir) }

        let outputDir = srcDir.appendingPathComponent("out", isDirectory: true)
        let srcFile = srcDir.appendingPathComponent("source.swift")
        let jsonFile = outputDir.appendingPathComponent("source.swift.json")

        try FileManager.default.createDirectory(
            atPath: srcDir.path,
            withIntermediateDirectories: true,
            attributes: nil
        )
        try code.write(to: srcFile, atomically: true, encoding: .utf8)

        try body(srcDir, outputDir, jsonFile)
    }

    private func uniqueName() -> String {
        "SwiftAstGenTests\(UUID().uuidString)"
    }
}
