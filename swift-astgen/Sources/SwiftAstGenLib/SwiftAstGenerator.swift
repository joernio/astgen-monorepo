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

			do {
				try FileManager.default.createDirectory(
					atPath: outfileDirUrl.path,
					withIntermediateDirectories: true,
					attributes: nil
				)
			} catch { /* this is ok; another thread may already created that dir */ }

			FileManager.default.createFile(
				atPath: outFileUrl.path,
				contents: nil,
				attributes: nil
			)
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

	private func iterateFilesInDirectory(_ directoryURL: URL) throws {
		let contents = try FileManager.default.contentsOfDirectory(at: directoryURL, includingPropertiesForKeys: nil, options: .skipsHiddenFiles)
		for item in contents {
		    var isDirectory: ObjCBool = false
		    if FileManager.default.fileExists(atPath: item.path, isDirectory: &isDirectory) {
		        if isDirectory.boolValue {
		            if ignoreDirectory(name: item.lastPathComponent) {
		                continue
		            }
		            try iterateFilesInDirectory(item)
		        } else {
					if item.lastPathComponent.hasSuffix(".swift") {
						parseFile(fileUrl: item)
					}
		        }
		    }
		}
	   
	}

	public func generate() throws {
		try iterateFilesInDirectory(srcDir)
	}

}
