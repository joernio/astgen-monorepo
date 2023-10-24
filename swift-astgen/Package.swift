// swift-tools-version: 5.9

import PackageDescription

let package = Package(
  name: "SwiftAstGen",
  platforms: [
    .macOS(.v10_15)
  ],
  products: [
    .library(name: "SwiftAstGenLib", targets: ["SwiftAstGenLib"])
  ],
  dependencies: [
    .package(url: "https://github.com/apple/swift-argument-parser", from: "1.2.0"),
    .package(url: "https://github.com/apple/swift-syntax", from: "509.0.0"),
  ],
  targets: [
    .target(
      name: "SwiftAstGenLib",
      dependencies: [
        .product(name: "SwiftSyntax", package: "swift-syntax"),
        .product(name: "SwiftOperators", package: "swift-syntax"),
        .product(name: "SwiftParser", package: "swift-syntax"),
        .product(name: "ArgumentParser", package: "swift-argument-parser"),
        "CodeGeneration",
      ]
    ),
    .target(
      name: "CodeGeneration",
      dependencies: [
        .product(name: "SwiftSyntax", package: "swift-syntax"),
        .product(name: "SwiftSyntaxBuilder", package: "swift-syntax"),
        .product(name: "SwiftOperators", package: "swift-syntax"),
        .product(name: "SwiftParser", package: "swift-syntax"),
      ]
    ),
    .executableTarget(
      name: "SwiftAstGen",
      dependencies: ["SwiftAstGenLib"],
      swiftSettings: [
        .unsafeFlags(["-cross-module-optimization"], .when(configuration: .release))
      ]
    ),
    .testTarget(
      name: "SwiftAstGenTests",
      dependencies: ["SwiftAstGenLib"]
    ),
  ]
)
