import Foundation
import XCTest

@testable import class SwiftAstGenLib.PackageTestTargetParser

final class PackageTestTargetParserTests: XCTestCase, TestUtils {

    private func writePackageSwift(in directory: URL, content: String) throws {
        try createFile(at: directory, path: "Package.swift", content: content)
    }

    func testSingleTestTarget() throws {
        let tempDir = try createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        try writePackageSwift(
            in: tempDir,
            content: """
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
        )

        let parser = PackageTestTargetParser(srcDir: tempDir)
        let testTargetPaths = parser.getTestTargetPaths()

        XCTAssertEqual(testTargetPaths, ["Tests/TestPackageTests"])
    }

    func testMultipleTestTargets() throws {
        let tempDir = try createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        try writePackageSwift(
            in: tempDir,
            content: """
                // swift-tools-version: 5.10
                import PackageDescription

                let package = Package(
                    name: "TestPackage",
                    targets: [
                        .target(name: "TestPackage"),
                        .testTarget(name: "TestPackageTests", dependencies: ["TestPackage"]),
                        .testTarget(name: "IntegrationTests", dependencies: ["TestPackage"]),
                        .testTarget(name: "PerformanceTests", dependencies: ["TestPackage"]),
                    ]
                )
                """
        )

        let parser = PackageTestTargetParser(srcDir: tempDir)
        let testTargetPaths = parser.getTestTargetPaths()

        XCTAssertEqual(testTargetPaths.count, 3)
        XCTAssertTrue(testTargetPaths.contains("Tests/TestPackageTests"))
        XCTAssertTrue(testTargetPaths.contains("Tests/IntegrationTests"))
        XCTAssertTrue(testTargetPaths.contains("Tests/PerformanceTests"))
    }

    func testTestTargetWithExplicitPath() throws {
        let tempDir = try createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        try writePackageSwift(
            in: tempDir,
            content: """
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
        )

        let parser = PackageTestTargetParser(srcDir: tempDir)
        let testTargetPaths = parser.getTestTargetPaths()

        XCTAssertEqual(testTargetPaths, ["CustomTests/Unit"])
    }

    func testMixedTestTargets() throws {
        let tempDir = try createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        try writePackageSwift(
            in: tempDir,
            content: """
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
        )

        let parser = PackageTestTargetParser(srcDir: tempDir)
        let testTargetPaths = parser.getTestTargetPaths()

        XCTAssertEqual(testTargetPaths.count, 3)
        XCTAssertTrue(testTargetPaths.contains("Tests/TestPackageTests"))
        XCTAssertTrue(testTargetPaths.contains("MyCustomPath/Tests"))
        XCTAssertTrue(testTargetPaths.contains("Tests/AnotherTargetTests"))
    }

    func testNoTestTargets() throws {
        let tempDir = try createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        try writePackageSwift(
            in: tempDir,
            content: """
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
        )

        let parser = PackageTestTargetParser(srcDir: tempDir)
        XCTAssertEqual(parser.getTestTargetPaths(), [])
    }

    func testMissingPackageSwift() throws {
        let tempDir = try createTemporaryDirectory()
        defer { cleanup(directory: tempDir) }

        let parser = PackageTestTargetParser(srcDir: tempDir)
        XCTAssertEqual(parser.getTestTargetPaths(), [])
    }
}
