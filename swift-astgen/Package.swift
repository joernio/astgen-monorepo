// swift-tools-version: 5.10

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
    .package(url: "https://github.com/apple/swift-syntax", from: "510.0.1"),
  ],
  targets: [
    .target(
      name: "SwiftAstGenLib",
      dependencies: [
        .product(name: "SwiftSyntax", package: "swift-syntax"),
        .product(name: "SwiftParser", package: "swift-syntax"),
        .product(name: "SwiftOperators", package: "swift-syntax"),
        "CodeGeneration",
      ],
      swiftSettings: [
        .unsafeFlags(["-warnings-as-errors"])
      ]
    ),
    .target(
      name: "CodeGeneration",
      dependencies: [
        .product(name: "SwiftSyntax", package: "swift-syntax"),
        .product(name: "SwiftSyntaxBuilder", package: "swift-syntax"),
      ]
    ),
    .executableTarget(
      name: "SwiftAstGen",
      dependencies: [
        .product(name: "ArgumentParser", package: "swift-argument-parser"),
        "SwiftAstGenLib",
      ],
      swiftSettings: [
        .unsafeFlags(["-cross-module-optimization"], .when(configuration: .release))
      ],
      linkerSettings: [
        // 2M stack size, as swift-linux-musl provides a default thread stack size of only 128k.
        // This prevents segmentation faults for larger Swift projects being scanned.
        .unsafeFlags(["-Xlinker", "-z", "-Xlinker", "stack-size=2097152"], .when(platforms: [.linux], configuration: .release))
      ],
    ),
    .testTarget(
      name: "SwiftAstGenTests",
      dependencies: ["SwiftAstGenLib"]
    ),
  ]
)
