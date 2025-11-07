import Foundation

import XCTest

@testable import class SwiftAstGenLib.PackageTestTargetParser

final class PackageTestTargetParserTests: XCTestCase {

    static var allTests = [
        ("testSingleTestTarget", testSingleTestTarget),
        ("testMultipleTestTargets", testMultipleTestTargets),
        ("testTestTargetWithExplicitPath", testTestTargetWithExplicitPath),
        ("testMixedTestTargets", testMixedTestTargets),
        ("testNoTestTargets", testNoTestTargets),
        ("testMissingPackageSwift", testMissingPackageSwift),
    ]

    private func createTemporaryDirectory() -> URL {
        let tempDir = URL(fileURLWithPath: NSTemporaryDirectory())
            .appendingPathComponent("PackageTestTargetParserTests-\(UUID().uuidString)", isDirectory: true)
        try! FileManager.default.createDirectory(atPath: tempDir.path, withIntermediateDirectories: true)
        return tempDir
    }

    private func cleanup(directory: URL) {
        try? FileManager.default.removeItem(at: directory)
    }

    private func createPackageSwift(in directory: URL, content: String) {
        let packageSwiftUrl = directory.appendingPathComponent("Package.swift")
        try! content.write(to: packageSwiftUrl, atomically: true, encoding: .utf8)
    }

    func testSingleTestTarget() throws {
        let tempDir = createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        let packageContent = """
            // swift-tools-version: 5.10
            import PackageDescription

            let package = Package(
            name: "TestPackage",
            targets: [
            	.target(name: "TestPackage"),
            	.testTarget(
            	name: "TestPackageTests",
            	dependencies: ["TestPackage"]
            	),
            ]
            )
            """

        createPackageSwift(in: tempDir, content: packageContent)

        let parser = PackageTestTargetParser(srcDir: tempDir)
        let testTargetPaths = parser.getTestTargetPaths()

        XCTAssertEqual(testTargetPaths.count, 1)
        XCTAssertEqual(testTargetPaths[0], "Tests/TestPackageTests")
    }

    func testMultipleTestTargets() throws {
        let tempDir = createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        let packageContent = """
            // swift-tools-version: 5.10
            import PackageDescription

            let package = Package(
                name: "TestPackage",
                targets: [
                    .target(name: "TestPackage"),
                    .testTarget(
                        name: "TestPackageTests",
                        dependencies: ["TestPackage"]
                    ),
                    .testTarget(
                        name: "IntegrationTests",
                        dependencies: ["TestPackage"]
                    ),
                    .testTarget(
                        name: "PerformanceTests",
                        dependencies: ["TestPackage"]
                    ),
                ]
            )
            """

        createPackageSwift(in: tempDir, content: packageContent)

        let parser = PackageTestTargetParser(srcDir: tempDir)
        let testTargetPaths = parser.getTestTargetPaths()

        XCTAssertEqual(testTargetPaths.count, 3)
        XCTAssertTrue(testTargetPaths.contains("Tests/TestPackageTests"))
        XCTAssertTrue(testTargetPaths.contains("Tests/IntegrationTests"))
        XCTAssertTrue(testTargetPaths.contains("Tests/PerformanceTests"))
    }

    func testTestTargetWithExplicitPath() throws {
        let tempDir = createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        let packageContent = """
            // swift-tools-version: 5.10
            import PackageDescription

            let package = Package(
                name: "TestPackage",
                targets: [
            	    .target(name: "TestPackage"),
            	    .testTarget(
            	        name: "TestPackageTests",
            	        dependencies: ["TestPackage"],
            	        path: "CustomTests/Unit"
            	    ),
                ]
            )
            """

        createPackageSwift(in: tempDir, content: packageContent)

        let parser = PackageTestTargetParser(srcDir: tempDir)
        let testTargetPaths = parser.getTestTargetPaths()

        XCTAssertEqual(testTargetPaths.count, 1)
        XCTAssertEqual(testTargetPaths[0], "CustomTests/Unit")
    }

    func testMixedTestTargets() throws {
        let tempDir = createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        let packageContent = """
            // swift-tools-version: 5.10
            import PackageDescription

            let package = Package(
                name: "TestPackage",
                targets: [
            	    .target(name: "TestPackage"),
            	    .testTarget(
            	        name: "TestPackageTests",
            	        dependencies: ["TestPackage"]
            	    ),
            	    .testTarget(
            	        name: "CustomTests",
            	        dependencies: ["TestPackage"],
            	        path: "MyCustomPath/Tests"
            	    ),
            	    .target(name: "AnotherTarget"),
            	    .testTarget(
            	        name: "AnotherTargetTests",
            	        dependencies: ["AnotherTarget"]
            	    ),
                ]
            )
            """

        createPackageSwift(in: tempDir, content: packageContent)

        let parser = PackageTestTargetParser(srcDir: tempDir)
        let testTargetPaths = parser.getTestTargetPaths()

        XCTAssertEqual(testTargetPaths.count, 3)
        XCTAssertTrue(testTargetPaths.contains("Tests/TestPackageTests"))
        XCTAssertTrue(testTargetPaths.contains("MyCustomPath/Tests"))
        XCTAssertTrue(testTargetPaths.contains("Tests/AnotherTargetTests"))
    }

    func testNoTestTargets() throws {
        let tempDir = createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        let packageContent = """
            // swift-tools-version: 5.10
            import PackageDescription

            let package = Package(
                name: "TestPackage",
                targets: [
                    .target(name: "TestPackage"),
            	    .target(name: "AnotherTarget"),
            	    .executableTarget(
            	        name: "MyExecutable",
            	        dependencies: ["TestPackage"]
            	    ),
                ]
            )
            """

        createPackageSwift(in: tempDir, content: packageContent)

        let parser = PackageTestTargetParser(srcDir: tempDir)
        let testTargetPaths = parser.getTestTargetPaths()

        XCTAssertEqual(testTargetPaths.count, 0)
    }

    func testMissingPackageSwift() throws {
        let tempDir = createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        // Don't create Package.swift

        let parser = PackageTestTargetParser(srcDir: tempDir)
        let testTargetPaths = parser.getTestTargetPaths()

        XCTAssertEqual(testTargetPaths.count, 0)
    }
}
