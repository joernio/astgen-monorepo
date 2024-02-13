import Foundation

public class SwiftAstGenerator {

	private var srcDir: URL
	private var outputDir: URL
	private var prettyPrint: Bool
	private let availableProcessors = ProcessInfo.processInfo.activeProcessorCount

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
		let nameLowercased = name.lowercased()
		return nameLowercased.contains("/.")
			|| nameLowercased.contains("/__")
			|| nameLowercased.contains("/tests/")
			|| nameLowercased.contains("/specs/")
			|| nameLowercased.contains("/test/")
			|| nameLowercased.contains("/spec/")
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
			} catch {}

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

	private func iterateSwiftFiles(at url: URL) {
		let queue = OperationQueue()
		queue.name = "SwiftAstGen"
		queue.qualityOfService = .userInitiated
		queue.maxConcurrentOperationCount = availableProcessors

		if let enumerator = FileManager.default.enumerator(
			at: url,
			includingPropertiesForKeys: [.isRegularFileKey],
			options: [.skipsHiddenFiles, .skipsPackageDescendants]) {
		    for case let fileURL as URL in enumerator {
	            let fileAttributes = try! fileURL.resourceValues(forKeys:[.isRegularFileKey])
	            if fileAttributes.isRegularFile! && fileURL.pathExtension == "swift" {
	            	let relativeFilePath = fileURL.relativePath(from: srcDir)!
	            	if !ignoreDirectory(name: "/\(relativeFilePath)") {
	            		queue.addOperation {
	            			self.parseFile(fileUrl: fileURL)
	            		}
	            	}
	            }
		    }
		}

		queue.waitUntilAllOperationsAreFinished()
	} 

	public func generate() throws {
	    iterateSwiftFiles(at: srcDir)
	}

}
