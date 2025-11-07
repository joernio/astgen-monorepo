import Foundation

@testable import class SwiftAstGenLib.TreeNode

protocol TestUtils {
    func loadJson(file: URL) -> TreeNode?
    func withCode(code: String, testFunction: (URL, URL, URL) throws -> Void) throws
}
extension TestUtils {

    private func createUniqueName() -> String {
        return "SwiftAstGenTests\(UUID().uuidString)"
    }

    func createTemporaryDirectory() -> URL {
        let tempDir = URL(fileURLWithPath: NSTemporaryDirectory())
            .appendingPathComponent(createUniqueName(), isDirectory: true)
        try! FileManager.default.createDirectory(atPath: tempDir.path, withIntermediateDirectories: true)
        return tempDir
    }
    
    func cleanup(directory: URL) {
        try? FileManager.default.removeItem(at: directory)
    }
    
    func createFile(at baseDir: URL, path: String, content: String) {
        let fileUrl = baseDir.appendingPathComponent(path)
        let dirUrl = fileUrl.deletingLastPathComponent()
        
        if !FileManager.default.fileExists(atPath: dirUrl.path) {
            try! FileManager.default.createDirectory(
                atPath: dirUrl.path,
                withIntermediateDirectories: true
            )
        }
        
        try! content.write(to: fileUrl, atomically: true, encoding: .utf8)
    }

    private func temporaryFileURL(fileName: String) -> URL? {
        return URL(fileURLWithPath: NSTemporaryDirectory(), isDirectory: true).appendingPathComponent(
            fileName,
            isDirectory: true
        )
    }

    func loadJson(file: URL) -> TreeNode? {
        let decoder = JSONDecoder()
        guard
            let content = try? String(contentsOf: file, encoding: .utf8),
            let treeNode = try? decoder.decode(TreeNode.self, from: content.data(using: .utf8)!)
        else {
            return nil
        }
        return treeNode
    }

    func withCode(code: String, testFunction: (URL, URL, URL) throws -> Void) throws {
        let srcTmpDir = temporaryFileURL(fileName: createUniqueName())!
        let outTmpDir = srcTmpDir.appendingPathComponent("out", isDirectory: true)
        let srcFile = srcTmpDir.appendingPathComponent("source.swift")
        let jsonFile = outTmpDir.appendingPathComponent("source.swift.json")

        try FileManager.default.createDirectory(
            atPath: srcTmpDir.path,
            withIntermediateDirectories: true,
            attributes: nil
        )
        _ = FileManager.default.createFile(
            atPath: srcFile.path,
            contents: nil,
            attributes: nil
        )
        try code.write(
            to: srcFile,
            atomically: true,
            encoding: String.Encoding.utf8
        )

        try testFunction(srcTmpDir, outTmpDir, jsonFile)
        try FileManager.default.removeItem(at: srcTmpDir)
    }

}
