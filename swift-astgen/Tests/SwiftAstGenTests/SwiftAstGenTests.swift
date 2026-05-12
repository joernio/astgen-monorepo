import Foundation
import XCTest

@testable import class SwiftAstGenLib.SwiftAstGenerator

final class SwiftAstGenTests: XCTestCase, TestUtils {

    func testJsonSourceFileSyntax() throws {
        try withCode(code: #"print("Hello World!")"#) { srcDir, outputDir, jsonFile in
            try SwiftAstGenerator(srcDir: srcDir, outputDir: outputDir, prettyPrint: false).generate()

            XCTAssertTrue(FileManager.default.fileExists(atPath: jsonFile.path))
            let treeNode = try loadJson(file: jsonFile)
            XCTAssertEqual(treeNode.nodeType, "SourceFileSyntax")
        }
    }

    func testJsonFilePaths() throws {
        try withCode(code: #"print("Hello World!")"#) { srcDir, outputDir, jsonFile in
            try SwiftAstGenerator(srcDir: srcDir, outputDir: outputDir, prettyPrint: false).generate()

            let treeNode = try loadJson(file: jsonFile)
            let projectFullPath = try XCTUnwrap(treeNode.projectFullPath)
            let relativeFilePath = try XCTUnwrap(treeNode.relativeFilePath)
            let fullFilePath = try XCTUnwrap(treeNode.fullFilePath)

            XCTAssertEqual(relativeFilePath, "source.swift")
            XCTAssertEqual(fullFilePath, "\(projectFullPath)/\(relativeFilePath)")
        }
    }

    func testJsonLoc() throws {
        let code = """
            print("1")
            print("2")
            print("3")
            """
        try withCode(code: code) { srcDir, outputDir, jsonFile in
            try SwiftAstGenerator(srcDir: srcDir, outputDir: outputDir, prettyPrint: false).generate()

            let treeNode = try loadJson(file: jsonFile)
            XCTAssertEqual(treeNode.loc, 3)
        }
    }

    func testPrettyPrintProducesIndentedJson() throws {
        try withCode(code: #"let x = 1"#) { srcDir, outputDir, jsonFile in
            try SwiftAstGenerator(srcDir: srcDir, outputDir: outputDir, prettyPrint: true).generate()

            let raw = try String(contentsOf: jsonFile, encoding: .utf8)
            XCTAssertTrue(raw.contains("\n  "), "Pretty-printed JSON should contain indentation")
        }
    }

    func testCustomOperatorDoesNotFailParsing() throws {
        let code = """
            infix operator <<<: AdditionPrecedence
            func <<< (lhs: Int, rhs: Int) -> Int { lhs + rhs }
            let result = 1 <<< 2
            """
        try withCode(code: code) { srcDir, outputDir, jsonFile in
            try SwiftAstGenerator(srcDir: srcDir, outputDir: outputDir, prettyPrint: false).generate()

            let treeNode = try loadJson(file: jsonFile)
            XCTAssertEqual(treeNode.nodeType, "SourceFileSyntax")
        }
    }

    func testMalformedSourceIsLoggedNotFatal() throws {
        try withCode(code: "let x = ") { srcDir, outputDir, _ in
            // Generation must not throw even if the file is unparseable; the error is logged
            // to stderr and the run continues.
            XCTAssertNoThrow(
                try SwiftAstGenerator(srcDir: srcDir, outputDir: outputDir, prettyPrint: false).generate()
            )
        }
    }

    func testIgnoresTestTargetPathsFromPackageSwift() throws {
        let tempDir = try createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        let packageContent = """
            // swift-tools-version: 5.10
            import PackageDescription

            let package = Package(
                name: "TestProject",
                targets: [
                    .target(name: "TestProject"),
                    .testTarget(
                        name: "TestProjectTests",
                        dependencies: ["TestProject"]
                    ),
                ]
            )
            """
        try createFile(at: tempDir, path: "Package.swift", content: packageContent)
        try createFile(at: tempDir, path: "Sources/main.swift", content: #"print("Main source")"#)
        try createFile(at: tempDir, path: "Tests/TestProjectTests/TestFile.swift", content: #"print("Test code")"#)

        let outputDir = tempDir.appendingPathComponent("output")
        try SwiftAstGenerator(srcDir: tempDir, outputDir: outputDir, prettyPrint: false).generate()

        XCTAssertTrue(
            FileManager.default.fileExists(atPath: outputDir.appendingPathComponent("Sources/main.swift.json").path),
            "Main source file should be processed"
        )
        XCTAssertFalse(
            FileManager.default.fileExists(
                atPath: outputDir.appendingPathComponent("Tests/TestProjectTests/TestFile.swift.json").path
            ),
            "Test target file should be ignored"
        )
    }

    func testIgnoresMultipleTestTargetPaths() throws {
        let tempDir = try createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        let packageContent = """
            // swift-tools-version: 5.10
            import PackageDescription

            let package = Package(
                name: "TestProject",
                targets: [
                    .target(name: "TestProject"),
                    .testTarget(name: "UnitTests", dependencies: ["TestProject"]),
                    .testTarget(name: "IntegrationTests", dependencies: ["TestProject"]),
                ]
            )
            """
        try createFile(at: tempDir, path: "Package.swift", content: packageContent)
        try createFile(at: tempDir, path: "Sources/main.swift", content: #"print("main")"#)
        try createFile(at: tempDir, path: "Tests/UnitTests/UnitTest.swift", content: #"print("unit")"#)
        try createFile(
            at: tempDir,
            path: "Tests/IntegrationTests/IntegrationTest.swift",
            content: #"print("integration")"#
        )

        let outputDir = tempDir.appendingPathComponent("output")
        try SwiftAstGenerator(srcDir: tempDir, outputDir: outputDir, prettyPrint: false).generate()

        XCTAssertTrue(
            FileManager.default.fileExists(atPath: outputDir.appendingPathComponent("Sources/main.swift.json").path)
        )
        XCTAssertFalse(
            FileManager.default.fileExists(
                atPath: outputDir.appendingPathComponent("Tests/UnitTests/UnitTest.swift.json").path
            ),
            "UnitTests should be ignored"
        )
        XCTAssertFalse(
            FileManager.default.fileExists(
                atPath: outputDir.appendingPathComponent("Tests/IntegrationTests/IntegrationTest.swift.json").path
            ),
            "IntegrationTests should be ignored"
        )
    }

    func testExcludeRegexSkipsMatchingFiles() throws {
        let tempDir = try createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        try createFile(at: tempDir, path: "Sources/keep.swift", content: #"print("keep")"#)
        try createFile(at: tempDir, path: "Sources/skip.swift", content: #"print("skip")"#)

        let outputDir = tempDir.appendingPathComponent("output")
        let regex = try NSRegularExpression(pattern: "skip\\.swift$", options: .caseInsensitive)
        try SwiftAstGenerator(srcDir: tempDir, outputDir: outputDir, prettyPrint: false, excludeRegex: regex)
            .generate()

        XCTAssertTrue(
            FileManager.default.fileExists(atPath: outputDir.appendingPathComponent("Sources/keep.swift.json").path)
        )
        XCTAssertFalse(
            FileManager.default.fileExists(atPath: outputDir.appendingPathComponent("Sources/skip.swift.json").path),
            "File matching --exclude-regex should be skipped"
        )
    }

    func testExcludeRegexSkipsMatchingDirectories() throws {
        let tempDir = try createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        try createFile(at: tempDir, path: "Sources/main.swift", content: #"print("main")"#)
        try createFile(at: tempDir, path: "Generated/api.swift", content: #"print("generated")"#)

        let outputDir = tempDir.appendingPathComponent("output")
        let regex = try NSRegularExpression(pattern: "/Generated/", options: .caseInsensitive)
        try SwiftAstGenerator(srcDir: tempDir, outputDir: outputDir, prettyPrint: false, excludeRegex: regex)
            .generate()

        XCTAssertTrue(
            FileManager.default.fileExists(atPath: outputDir.appendingPathComponent("Sources/main.swift.json").path)
        )
        XCTAssertFalse(
            FileManager.default.fileExists(atPath: outputDir.appendingPathComponent("Generated/api.swift.json").path),
            "Files inside a directory matching --exclude-regex should be skipped"
        )
    }

    func testExcludeRegexIsCaseInsensitive() throws {
        let tempDir = try createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        try createFile(at: tempDir, path: "FOO/bar.swift", content: #"print("bar")"#)

        let outputDir = tempDir.appendingPathComponent("output")
        let regex = try NSRegularExpression(pattern: "foo", options: .caseInsensitive)
        try SwiftAstGenerator(srcDir: tempDir, outputDir: outputDir, prettyPrint: false, excludeRegex: regex)
            .generate()

        XCTAssertFalse(
            FileManager.default.fileExists(atPath: outputDir.appendingPathComponent("FOO/bar.swift.json").path),
            "Case-insensitive --exclude-regex should match regardless of letter case"
        )
    }

    func testIgnoresCustomTestTargetPath() throws {
        let tempDir = try createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        let packageContent = """
            // swift-tools-version: 5.10
            import PackageDescription

            let package = Package(
                name: "TestProject",
                targets: [
                    .target(name: "TestProject"),
                    .testTarget(
                        name: "MyTests",
                        dependencies: ["TestProject"],
                        path: "CustomTestPath"
                    ),
                ]
            )
            """
        try createFile(at: tempDir, path: "Package.swift", content: packageContent)
        try createFile(at: tempDir, path: "Sources/main.swift", content: #"print("main")"#)
        try createFile(at: tempDir, path: "CustomTestPath/MyTest.swift", content: #"print("test")"#)

        let outputDir = tempDir.appendingPathComponent("output")
        try SwiftAstGenerator(srcDir: tempDir, outputDir: outputDir, prettyPrint: false).generate()

        XCTAssertTrue(
            FileManager.default.fileExists(atPath: outputDir.appendingPathComponent("Sources/main.swift.json").path)
        )
        XCTAssertFalse(
            FileManager.default.fileExists(
                atPath: outputDir.appendingPathComponent("CustomTestPath/MyTest.swift.json").path
            ),
            "Custom test path should be ignored"
        )
    }
}
