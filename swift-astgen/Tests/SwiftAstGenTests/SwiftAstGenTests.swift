import XCTest

@testable import class SwiftAstGenLib.SwiftAstGenerator

final class SwiftAstGenTests: XCTestCase, TestUtils {

    static var allTests = [
        ("testJsonSourceFileSyntax", testJsonSourceFileSyntax),
        ("testJsonFilePaths", testJsonFilePaths),
        ("testJsonLoc", testJsonLoc),
        ("testIgnoresTestTargetPathsFromPackageSwift", testIgnoresTestTargetPathsFromPackageSwift),
        ("testIgnoresMultipleTestTargetPaths", testIgnoresMultipleTestTargetPaths),
        ("testIgnoresCustomTestTargetPath", testIgnoresCustomTestTargetPath),
    ]

    func testJsonSourceFileSyntax() throws {
        try withCode(
            code: """
                print("Hello World!")
                """
        ) { srcDir, outputDir, jsonFile in

            try SwiftAstGenerator(
                srcDir: srcDir,
                outputDir: outputDir,
                prettyPrint: false
            ).generate()

            XCTAssertTrue(FileManager.default.fileExists(atPath: jsonFile.path))
            if let treeNode = loadJson(file: jsonFile) {
                XCTAssertEqual(treeNode.nodeType, "SourceFileSyntax")
            } else {
                XCTFail("Could not create the JSON containing the Swift AST.")
            }
        }
    }

    func testJsonFilePaths() throws {
        try withCode(
            code: """
                print("Hello World!")
                """
        ) { srcDir, outputDir, jsonFile in

            try SwiftAstGenerator(
                srcDir: srcDir,
                outputDir: outputDir,
                prettyPrint: false
            ).generate()

            XCTAssertTrue(FileManager.default.fileExists(atPath: jsonFile.path))
            if let treeNode = loadJson(file: jsonFile) {
                let projectFullPath = treeNode.projectFullPath!
                let relativeFilePath = treeNode.relativeFilePath!
                let fullFilePath = treeNode.fullFilePath!

                XCTAssertEqual(relativeFilePath, "source.swift")
                XCTAssertEqual(fullFilePath, "\(projectFullPath)/\(relativeFilePath)")
            } else {
                XCTFail("Could not create the JSON containing the Swift AST.")
            }
        }
    }

    func testJsonLoc() throws {
        try withCode(
            code: """
                print("1")
                print("2")
                print("3")
                """
        ) { srcDir, outputDir, jsonFile in

            try SwiftAstGenerator(
                srcDir: srcDir,
                outputDir: outputDir,
                prettyPrint: false
            ).generate()

            XCTAssertTrue(FileManager.default.fileExists(atPath: jsonFile.path))
            if let treeNode = loadJson(file: jsonFile) {
                let loc = treeNode.loc!
                XCTAssertEqual(loc, 3)
            } else {
                XCTFail("Could not create the JSON containing the Swift AST.")
            }
        }
    }

    func testIgnoresTestTargetPathsFromPackageSwift() throws {
        let tempDir = createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }
        
        // Create Package.swift with a test target
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
        createFile(at: tempDir, path: "Package.swift", content: packageContent)
        
        // Create a Swift file in the main source
        let sourceCode = "print(\"Main source\")"
        createFile(at: tempDir, path: "Sources/main.swift", content: sourceCode)
        
        // Create a Swift file in the test target path
        let testCode = "print(\"Test code\")"
        createFile(at: tempDir, path: "Tests/TestProjectTests/TestFile.swift", content: testCode)
        
        let outputDir = tempDir.appendingPathComponent("output")
        
        try SwiftAstGenerator(
            srcDir: tempDir,
            outputDir: outputDir,
            prettyPrint: false
        ).generate()
        
        // Main source file should be processed
        let mainJsonPath = outputDir.appendingPathComponent("Sources/main.swift.json")
        XCTAssertTrue(
            FileManager.default.fileExists(atPath: mainJsonPath.path),
            "Main source file should be processed"
        )
        
        // Test file should be ignored
        let testJsonPath = outputDir.appendingPathComponent("Tests/TestProjectTests/TestFile.swift.json")
        XCTAssertFalse(
            FileManager.default.fileExists(atPath: testJsonPath.path),
            "Test target file should be ignored"
        )
    }
    
    func testIgnoresMultipleTestTargetPaths() throws {
        let tempDir = createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }
        
        // Create Package.swift with multiple test targets
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
        createFile(at: tempDir, path: "Package.swift", content: packageContent)
        
        // Create source file
        createFile(at: tempDir, path: "Sources/main.swift", content: "print(\"main\")")
        
        // Create test files in different test targets
        createFile(at: tempDir, path: "Tests/UnitTests/UnitTest.swift", content: "print(\"unit\")")
        createFile(at: tempDir, path: "Tests/IntegrationTests/IntegrationTest.swift", content: "print(\"integration\")")
        
        let outputDir = tempDir.appendingPathComponent("output")
        
        try SwiftAstGenerator(
            srcDir: tempDir,
            outputDir: outputDir,
            prettyPrint: false
        ).generate()
        
        // Main source should be processed
        XCTAssertTrue(
            FileManager.default.fileExists(atPath: outputDir.appendingPathComponent("Sources/main.swift.json").path)
        )
        
        // Both test files should be ignored
        XCTAssertFalse(
            FileManager.default.fileExists(atPath: outputDir.appendingPathComponent("Tests/UnitTests/UnitTest.swift.json").path),
            "UnitTests should be ignored"
        )
        XCTAssertFalse(
            FileManager.default.fileExists(atPath: outputDir.appendingPathComponent("Tests/IntegrationTests/IntegrationTest.swift.json").path),
            "IntegrationTests should be ignored"
        )
    }
    
    func testIgnoresCustomTestTargetPath() throws {
        let tempDir = createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }
        
        // Create Package.swift with custom test path
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
        createFile(at: tempDir, path: "Package.swift", content: packageContent)
        
        // Create source file
        createFile(at: tempDir, path: "Sources/main.swift", content: "print(\"main\")")
        
        // Create test file in custom path
        createFile(at: tempDir, path: "CustomTestPath/MyTest.swift", content: "print(\"test\")")
        
        let outputDir = tempDir.appendingPathComponent("output")
        
        try SwiftAstGenerator(
            srcDir: tempDir,
            outputDir: outputDir,
            prettyPrint: false
        ).generate()
        
        // Main source should be processed
        XCTAssertTrue(
            FileManager.default.fileExists(atPath: outputDir.appendingPathComponent("Sources/main.swift.json").path)
        )
        
        // Custom test path should be ignored
        XCTAssertFalse(
            FileManager.default.fileExists(atPath: outputDir.appendingPathComponent("CustomTestPath/MyTest.swift.json").path),
            "Custom test path should be ignored"
        )
    }

}
