import Foundation

public class SwiftAstGenerator {

	private var srcDir: URL
	private var outputDir: URL
	private var prettyPrint: Bool

	public init(srcDir: URL, outputDir: URL, prettyPrint: Bool) throws {
		self.srcDir = srcDir
		self.outputDir = outputDir
		self.prettyPrint = prettyPrint
		if !FileManager.default.fileExists(atPath: outputDir.path) {
			try FileManager.default.createDirectory(
				atPath: outputDir.path,
				withIntermediateDirectories: true,
				attributes: nil
			)
		}
	}

	private func ignoreDirectory(name: String) -> Bool {
		return name.starts(with: ".")
			|| name.starts(with: "__")
			|| name.starts(with: "Tests/")
			|| name.starts(with: "Specs/")
			|| name.starts(with: "Test/")
			|| name.starts(with: "Spec/")
	}

	private func parseFile(fileUrl: URL) {
		do {
			let relativeFilePath = fileUrl.relativePath(from: srcDir)!
			let astJsonString = try SyntaxParser.parse(
				srcDir: srcDir,
				fileUrl: fileUrl,
				relativeFilePath: relativeFilePath,
				prettyPrint: prettyPrint)
			let outFileUrl =
				outputDir
				.appendingPathComponent(relativeFilePath)
				.appendingPathExtension("json")
			let outfileDirUrl = outFileUrl.deletingLastPathComponent()
			if !FileManager.default.fileExists(atPath: outfileDirUrl.path) {
				try FileManager.default.createDirectory(
					atPath: outfileDirUrl.path,
					withIntermediateDirectories: true,
					attributes: nil
				)
			}
			if !FileManager.default.fileExists(atPath: outFileUrl.path) {
				FileManager.default.createFile(
					atPath: outFileUrl.path,
					contents: nil,
					attributes: nil
				)
			}
			try astJsonString.write(
				to: outFileUrl,
				atomically: true,
				encoding: String.Encoding.utf8
			)

			print("Generated AST for file: `\(fileUrl.path)`")
		} catch {
			print("Parsing failed for file: `\(fileUrl.path)` (\(error))")
		}
	}

	public func generate() throws {
		let resourceKeys = Set<URLResourceKey>([.nameKey, .isDirectoryKey, .isRegularFileKey])
		let directoryEnumerator = FileManager.default.enumerator(
			at: srcDir,
			includingPropertiesForKeys: Array(resourceKeys),
			options: [.skipsPackageDescendants])!

		for case let fileUrl as URL in directoryEnumerator {
			guard let resourceValues = try? fileUrl.resourceValues(forKeys: resourceKeys),
				let isDirectory = resourceValues.isDirectory,
				let isRegularFile = resourceValues.isRegularFile,
				let name = resourceValues.name
			else {
				continue
			}

			if isDirectory {
				if ignoreDirectory(name: name) {
					directoryEnumerator.skipDescendants()
				}
			} else if isRegularFile && name.hasSuffix(".swift") {
				parseFile(fileUrl: fileUrl)
			}
		}
	}

}
