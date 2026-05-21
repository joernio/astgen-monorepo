package io.joern.rust2cpg.parser

import io.joern.rust2cpg.parser.RustNodeSyntax.{
  CallExpr,
  Fn,
  IdentPat,
  LetStmt
}
import org.scalatest.funsuite.AnyFunSuite
import org.scalatest.matchers.should.Matchers.*

class RustAstGenFullNameTests extends AnyFunSuite with RustAstGenTestFixture {

  test("methodFullName/typeFullName from JSON") {
    val srcFile = code(
      """
        |fn add(a: i32, b: i32) -> i32 { a + b }
        |fn main() { let x = add(1, 2); }
        |""".stripMargin)

    val letStmt: LetStmt = srcFile.item
      .collect { case fn: Fn => fn }
      .flatMap(_.blockExpr)
      .flatMap(_.stmtList.stmt)
      .collectFirst { case ls: LetStmt => ls }
      .getOrElse(fail("expected let statement"))

    val identPat: IdentPat = letStmt.pat match {
      case ident: IdentPat => ident
      case other           => fail(s"expected IdentPat, got ${other.getClass.getSimpleName}")
    }
    identPat.typeFullName shouldBe Some("i32")

    val callExpr: CallExpr = letStmt.expr.getOrElse(fail("expected let initializer")) match {
      case ce: CallExpr => ce
      case other        => fail(s"expected CallExpr, got ${other.getClass.getSimpleName}")
    }
    callExpr.methodFullName shouldBe Some("rust_ast_gen_scala_test::add")
    callExpr.typeFullName shouldBe Some("i32")

    val lParen = callExpr.argList.lParenToken
    lParen.methodFullName shouldBe None
    lParen.typeFullName shouldBe None
  }

  test("LetStmt.expr is None for an uninitialized binding") {
    val srcFile = code(
      """
        |fn main() { let x: i32; }
        |""".stripMargin)

    val letStmt: LetStmt = srcFile.item
      .collect { case fn: Fn => fn }
      .flatMap(_.blockExpr)
      .flatMap(_.stmtList.stmt)
      .collectFirst { case ls: LetStmt => ls }
      .getOrElse(fail("expected let statement"))

    letStmt.expr shouldBe None
  }

}
