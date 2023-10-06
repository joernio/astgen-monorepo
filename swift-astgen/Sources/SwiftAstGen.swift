import ArgumentParser
import Foundation
import Logging

struct Defaults {
	static let defaultSrcDir = "."
	static let defaultOutDir = "./ast_out"
}

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

  	mutating func validate() throws {
    	guard FileManager.default.fileExists(atPath: src.path) else {
      		throw ValidationError("Directory does not exist: `\(src.path)`")
    	}
  	}
}

extension SwiftAstGen {
	var logger: Logger {
		get { Logger(label: "io.joern.SwiftAstGen") }
	}

    mutating func run() throws {
      logger.info("Src directory is: `\(src.path)`")
      logger.info("Out directory is: `\(output.path)`")

      try SwiftAstGenerator(srcDir: src, outputDir: output).generate()
    }

}