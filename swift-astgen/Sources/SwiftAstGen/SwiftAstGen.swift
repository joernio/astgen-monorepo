import ArgumentParser
import Foundation
import SwiftAstGenLib

@main
struct SwiftAstGen: ParsableCommand {
  @Option(
    name: [.customLong("src"), .customShort("i")],
    help: "Source directory (default: `\(Defaults.defaultSrcDir)`).",
    completion: .file(),
    transform: URL.init(fileURLWithPath:))
  var src: URL = URL(fileURLWithPath: Defaults.defaultSrcDir)

  @Option(
    name: [.customLong("output"), .customShort("o")],
    help: "Output directory for generated AST json files (default: `\(Defaults.defaultOutDir)`).",
    completion: .file(),
    transform: URL.init(fileURLWithPath:))
  var output: URL = URL(fileURLWithPath: Defaults.defaultOutDir)

  @Flag(
    name: [.customLong("prettyPrint"), .customShort("p")],
    help: "Pretty print the generated AST json files (default: `false`).")
  var prettyPrint: Bool = false

  @Flag(
    name: [.customLong("scalaAstOnly"), .customShort("s")],
    help: "Only print the generated Scala SwiftSyntax AST nodes (default: `false`).")
  var scalaAstOnly: Bool = false

  func validate() throws {
    guard FileManager.default.fileExists(atPath: src.path) else {
      throw ValidationError("Directory does not exist: `\(src.path)`")
    }
  }
}

extension SwiftAstGen {
  func run() throws {
    if scalaAstOnly {
      try ScalaAstGenerator().generate()
    } else {
      try SwiftAstGenerator(srcDir: src, outputDir: output, prettyPrint: prettyPrint).generate()
    }
  }

}
