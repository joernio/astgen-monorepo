import XCTest

@testable import class SwiftAstGenLib.ScalaAstGenerator

final class ScalaAstGenTests: XCTestCase {

  static var allTests = [
    ("testScalaSourceFileOutput", testScalaSourceFileOutput)
  ]

  func testScalaSourceFileOutput() throws {
    let scalaOutFileUrl = URL(fileURLWithPath: "./SwiftNodeSyntax.scala")
    try ScalaAstGenerator().generate()
    XCTAssertTrue(FileManager.default.fileExists(atPath: scalaOutFileUrl.path))
    if let content = try? String(contentsOf: scalaOutFileUrl, encoding: .utf8) {
      XCTAssertTrue(content.contains("object SwiftNodeSyntax {"))
    } else {
      XCTFail("Could not create the SwiftNodeSyntax.scala file containing the Scala Swift AST.")
    }
  }

}
