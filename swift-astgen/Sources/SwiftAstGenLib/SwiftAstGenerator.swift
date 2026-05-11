import Foundation

/// Walks a Swift project on disk, parses every `.swift` source file with ``SyntaxParser``,
/// and writes one JSON document per file to a mirrored directory tree under `outputDir`.
///
/// Files inside common test/spec directories are skipped, as are any directories matching
/// `testTarget(...)` paths declared in the project's root `Package.swift` (see
/// ``PackageTestTargetParser``).
///
/// Per-file failures are logged to stderr and never abort the run; ``generate()`` always
/// completes normally as long as the output directory can be created.
public final class SwiftAstGenerator {

    /// Lower-cased path substrings (matched against `/<relative-path>`) that cause a file
    /// to be skipped. Substring matching is intentionally permissive: any directory whose
    /// lower-cased name matches one of these will be ignored along with all its contents.
    private static let ignoredPathSubstrings: [String] = [
        "/.", "/__", "/tests/", "/specs/", "/test/", "/spec/",
    ]

    private let srcDir: URL
    private let outputDir: URL
    private let prettyPrint: Bool
    private let ignorePathsFromPackageSwift: [String]
    private let availableProcessors: Int = ProcessInfo.processInfo.activeProcessorCount

    /// Creates a new generator.
    ///
    /// - Parameters:
    ///   - srcDir: Project root to scan for `.swift` files.
    ///   - outputDir: Directory under which JSON files will be written. Created lazily by
    ///     ``generate()``; the initializer performs no I/O.
    ///   - prettyPrint: If `true`, JSON output is pretty-printed.
    public init(srcDir: URL, outputDir: URL, prettyPrint: Bool) {
        self.srcDir = srcDir
        self.outputDir = outputDir
        self.prettyPrint = prettyPrint
        self.ignorePathsFromPackageSwift = PackageTestTargetParser(srcDir: srcDir)
            .getTestTargetPaths()
            .map { $0.lowercased() }
    }

    /// Walks the source tree and writes one `<relative-path>.json` file per `.swift` source.
    ///
    /// - Throws: Only if the output directory itself cannot be created. Per-file parse and
    ///   write failures are logged to stderr and do not propagate.
    public func generate() throws {
        try FileManager.default.createDirectory(
            atPath: outputDir.path,
            withIntermediateDirectories: true,
            attributes: nil
        )
        iterateSwiftFiles(at: srcDir)
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
            Log.info("Generated AST for file: `\(fileUrl.path)`")
        } catch {
            Log.warn("Parsing failed for file: `\(fileUrl.path)` (\(error))")
        }
    }

    private func iterateSwiftFiles(at url: URL) {
        let queue = OperationQueue()
        queue.name = "io.joern.swiftastgen.iteratequeue"
        queue.qualityOfService = .userInitiated
        queue.maxConcurrentOperationCount = availableProcessors

        guard
            let enumerator = FileManager.default.enumerator(
                at: url,
                includingPropertiesForKeys: [.isRegularFileKey],
                options: [.skipsHiddenFiles, .skipsPackageDescendants]
            )
        else {
            return
        }

        for case let fileURL as URL in enumerator {
            guard
                let resourceValues = try? fileURL.resourceValues(forKeys: [.isRegularFileKey]),
                resourceValues.isRegularFile == true,
                fileURL.pathExtension == "swift",
                let relativeFilePath = fileURL.pathRelative(to: srcDir)
            else {
                continue
            }
            if shouldIgnore(path: "/\(relativeFilePath)") {
                continue
            }
            queue.addOperation { [self] in
                parseFile(fileUrl: fileURL, relativeFilePath: relativeFilePath)
            }
        }
        queue.waitUntilAllOperationsAreFinished()
    }
}
