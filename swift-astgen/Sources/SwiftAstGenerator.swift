import Foundation
import Logging

class SwiftAstGenerator {

  private var srcDir: URL
  private var outputDir: URL
  private let fileManager = FileManager.default
  private let availableProcessors = ProcessInfo.processInfo.activeProcessorCount

  private var logger: Logger = Logger(label: "io.joern.SwiftAstGenerator")

  init(srcDir: URL, outputDir: URL) throws {
    self.srcDir = srcDir
    self.outputDir = outputDir
    if FileManager.default.fileExists(atPath: outputDir.path) {
      try FileManager.default.removeItem(at: outputDir)
    }
    try FileManager.default.createDirectory(
      atPath: outputDir.path,
      withIntermediateDirectories: true,
      attributes: nil
    )
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
      let astJsonString = try SyntaxParser.parse(fileURL: fileUrl)
      let absoluteFilePath = fileUrl.absoluteString
      let relativeFilePath = absoluteFilePath.replacingOccurrences(
        of: srcDir.absoluteString, with: "")
      let outFileUrl = outputDir.appendingPathComponent(relativeFilePath).deletingPathExtension()
        .appendingPathExtension("json")
      let outfileDirUrl = outFileUrl.deletingLastPathComponent()

      try fileManager.createDirectory(
        atPath: outfileDirUrl.path,
        withIntermediateDirectories: true,
        attributes: nil
      )

      fileManager.createFile(
        atPath: outFileUrl.path,
        contents: nil,
        attributes: nil
      )
      try astJsonString.write(
        to: outFileUrl,
        atomically: true,
        encoding: String.Encoding.utf8
      )
      logger.info("Generated AST for file: `\(fileUrl.path)`")
    } catch {
      logger.warning("Parsing failed for file: `\(fileUrl.path)` (\(error))")
    }
  }

  func generate() throws {
    let resourceKeys = Set<URLResourceKey>([.nameKey, .isDirectoryKey, .isRegularFileKey])
    let directoryEnumerator = fileManager.enumerator(
      at: srcDir,
      includingPropertiesForKeys: Array(resourceKeys),
      options: [.skipsHiddenFiles, .skipsPackageDescendants])!

    let queue = OperationQueue()
    queue.name = "SwiftAstGen"
    queue.qualityOfService = .userInitiated
    queue.maxConcurrentOperationCount = availableProcessors

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
        queue.addOperation {
          self.parseFile(fileUrl: fileUrl)
        }
      }
    }

    queue.waitUntilAllOperationsAreFinished()
  }

}
