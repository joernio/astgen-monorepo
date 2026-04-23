package io.joern.rust2cpg.parser

import io.joern.rust2cpg.parser.RustNodeSyntax.Fn
import org.scalatest.Inside.inside
import org.scalatest.funsuite.AnyFunSuite
import org.scalatest.matchers.should.Matchers.shouldBe

class RustAstGenTraversalTests extends AnyFunSuite with RustAstGenTestFixture {

  test("fn main() {}") {
    val srcFile = code(
      """
        |fn main() {}
        |""".stripMargin)

    inside(srcFile.item) {
      case Seq(fn: Fn) => {
        inside(fn.name.identToken) {
          case Some(indentToken) =>
            // TODO: have an extension to fetch the corresponding code.
            (indentToken.startLine, indentToken.startColumn) shouldBe(Some(1), Some(3))
        }
      }
    }
  }

}
