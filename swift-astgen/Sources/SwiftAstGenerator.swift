import Foundation
import Logging

class SwiftAstGenerator {

	private var srcDir: URL
	private var outputDir: URL

	private var logger: Logger = Logger(label: "io.joern.SwiftAstGenerator")

	init(srcDir: URL, outputDir: URL) throws {
		self.srcDir = srcDir
		self.outputDir = outputDir
		if (FileManager.default.fileExists(atPath: outputDir.path)) {
			try FileManager.default.removeItem(at: outputDir)
		}
		try FileManager.default.createDirectory(
			atPath: outputDir.path,
			withIntermediateDirectories: true,
			attributes: nil
		)
	}

	private func filter(fileURL: URL) throws -> Bool {
		let fileAttributes = try fileURL.resourceValues(forKeys: [.isRegularFileKey])
		return fileAttributes.isRegularFile! 
			&& !fileURL.path.contains("/.")
			&& !fileURL.path.contains("/__")
			&& !fileURL.path.contains("/Tests/")
			&& !fileURL.path.contains("/Specs/")
			&& !fileURL.path.contains("/Test/")
			&& !fileURL.path.contains("/Spec/")
			&& fileURL.path.hasSuffix(".swift")
	}

	func generate() throws {
		if let enumerator = FileManager.default.enumerator(at: srcDir, includingPropertiesForKeys: [.isRegularFileKey], options: [.skipsHiddenFiles, .skipsPackageDescendants]) {
    		for case let fileURL as URL in enumerator {
        		if try filter(fileURL: fileURL) {
            		do {
            			let astJson = try SyntaxParser.parse(fileURL: fileURL)

						let str = fileURL.absoluteString
						let replaced = str.replacingOccurrences(of: srcDir.absoluteString, with: "")
            			let outFile = outputDir.appendingPathComponent(replaced).deletingPathExtension().appendingPathExtension("json")
						let dirUrl = outFile.deletingLastPathComponent()

						try FileManager.default.createDirectory(
							atPath: dirUrl.path,
							withIntermediateDirectories: true,
							attributes: nil
						)

            			FileManager.default.createFile(
							atPath: outFile.path,
							contents: nil,
							attributes: nil
						)
            			try astJson.write(
							to: outFile,
							atomically: true,
							encoding: String.Encoding.utf8
						)
						logger.warning("Generated AST for file: `\(fileURL.path)`")
        			} catch {
        				logger.warning("Parsing failed for file: \(fileURL.path) (\(error))")
        			}
        		}
    		}
		}
	}

}