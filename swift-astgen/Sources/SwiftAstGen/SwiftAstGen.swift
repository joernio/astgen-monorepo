import ArgumentParser
import Foundation
import SwiftAstGenLib

/// Default source directory used when `--src` is omitted.
private let defaultSrcDir = "."

/// Default output directory used when `--output` is omitted in AST mode.
private let defaultOutDir = "./ast_out"

/// Default Scala output file written when `--scala-ast-only` is supplied.
private let defaultScalaOutPath = "./SwiftNodeSyntax.scala"

@main
struct SwiftAstGen: ParsableCommand {

    static let configuration = CommandConfiguration(
        commandName: "SwiftAstGen",
        abstract: "Generates JSON ASTs for Swift source files using SwiftSyntax."
    )

    @Option(
        name: [.customLong("src"), .customShort("i")],
        help: "Source directory (default: `\(defaultSrcDir)`).",
        completion: .file(),
        transform: URL.init(fileURLWithPath:)
    )
    var src: URL = URL(fileURLWithPath: defaultSrcDir)

    @Option(
        name: [.customLong("output"), .customShort("o")],
        help: "Output directory for generated AST json files (default: `\(defaultOutDir)`).",
        completion: .file(),
        transform: URL.init(fileURLWithPath:)
    )
    var output: URL = URL(fileURLWithPath: defaultOutDir)

    @Flag(
        name: [.long, .customLong("prettyPrint"), .customShort("p")],
        help: "Pretty print the generated AST json files."
    )
    var prettyPrint: Bool = false

    @Flag(
        name: [.long, .customLong("scalaAstOnly"), .customShort("s")],
        help: "Only print the generated Scala SwiftSyntax AST nodes (writes `\(defaultScalaOutPath)`)."
    )
    var scalaAstOnly: Bool = false

    func validate() throws {
        guard FileManager.default.fileExists(atPath: src.path) else {
            throw ValidationError("Directory does not exist: `\(src.path)`")
        }
    }

    func run() throws {
        if scalaAstOnly {
            try ScalaAstGenerator(outputUrl: URL(fileURLWithPath: defaultScalaOutPath)).generate()
        } else {
            try SwiftAstGenerator(srcDir: src, outputDir: output, prettyPrint: prettyPrint).generate()
        }
    }
}
