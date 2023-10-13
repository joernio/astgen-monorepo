import XCTest

@testable import class SwiftAstGenLib.SwiftAstGenerator

final class SwiftAstGenTests: XCTestCase, TestUtils {

  static var allTests = [
    ("testJsonOutput", testJsonOutput)
  ]

  func testJsonOutput() throws {
    try withCode(
      code: """
          print("Hello World!")
        """
    ) { srcDir, outputDir, jsonFile in

      try SwiftAstGenerator(
        srcDir: srcDir,
        outputDir: outputDir,
        prettyPrint: false
      ).generate()

      XCTAssertTrue(FileManager.default.fileExists(atPath: jsonFile.path))
      if let treeNode = loadJson(file: jsonFile) {
        XCTAssertEqual(treeNode.nodeType, "SourceFileSyntax")
      } else {
        XCTFail("Could not create the JSON containing the Swift AST.")
      }
    }
  }

}
