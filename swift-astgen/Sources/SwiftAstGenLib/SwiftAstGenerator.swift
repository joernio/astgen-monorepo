import Foundation

public class SwiftAstGenerator {

    private static let ignoredPathSubstrings: [String] = [
        "/.", "/__", "/tests/", "/specs/", "/test/", "/spec/",
    ]

    private let srcDir: URL
    private let outputDir: URL
    private let prettyPrint: Bool
    private let ignorePathsFromPackageSwift: [String]
    private let availableProcessors: Int = ProcessInfo.processInfo.activeProcessorCount

    public init(srcDir: URL, outputDir: URL, prettyPrint: Bool) throws {
        self.srcDir = srcDir
        self.outputDir = outputDir
        self.prettyPrint = prettyPrint
        self.ignorePathsFromPackageSwift = PackageTestTargetParser(srcDir: srcDir)
            .getTestTargetPaths()
            .map { $0.lowercased() }

        try FileManager.default.createDirectory(
            atPath: outputDir.path,
            withIntermediateDirectories: true,
            attributes: nil
        )
    }

    private func shouldIgnore(path: String) -> Bool {
        let pathLowercased = path.lowercased()
        for substring in Self.ignoredPathSubstrings where pathLowercased.contains(substring) {
            return true
        }
        return ignorePathsFromPackageSwift.contains { pathLowercased.contains($0) }
    }

    private func parseFile(fileUrl: URL, relativeFilePath: String) {
        do {
            let astJsonData = try SyntaxParser.parse(
                srcDir: srcDir,
                fileUrl: fileUrl,
                relativeFilePath: relativeFilePath,
                prettyPrint: prettyPrint
            )
            let outFileUrl =
                outputDir
                .appendingPathComponent(relativeFilePath)
                .appendingPathExtension("json")
            let outfileDirUrl = outFileUrl.deletingLastPathComponent()

            try FileManager.default.createDirectory(
                atPath: outfileDirUrl.path,
                withIntermediateDirectories: true,
                attributes: nil
            )

            try astJsonData.write(to: outFileUrl, options: .atomic)
            print("Generated AST for file: `\(fileUrl.path)`")
        } catch {
            print("Parsing failed for file: `\(fileUrl.path)` (\(error))")
        }
    }

    private func iterateSwiftFiles(at url: URL) {
        let queue = OperationQueue()
        queue.name = "io.joern.swiftastgen.iteratequeue"
        queue.qualityOfService = .userInitiated
        queue.maxConcurrentOperationCount = availableProcessors

        if let enumerator = FileManager.default.enumerator(
            at: url,
            includingPropertiesForKeys: [.isRegularFileKey],
            options: [.skipsHiddenFiles, .skipsPackageDescendants]
        ) {
            for case let fileURL as URL in enumerator {
                let fileAttributes = try! fileURL.resourceValues(forKeys: [.isRegularFileKey])
                if fileAttributes.isRegularFile! && fileURL.pathExtension == "swift" {
                    let relativeFilePath = fileURL.relativePath(from: srcDir)!
                    if !shouldIgnore(path: "/\(relativeFilePath)") {
                        queue.addOperation {
                            self.parseFile(fileUrl: fileURL, relativeFilePath: relativeFilePath)
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
