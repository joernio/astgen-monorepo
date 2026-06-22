package io.joern.rust2cpg.parser

import io.joern.rust2cpg.parser.RustNodeSyntax.{Borrow, Cast, Deref}
import org.scalatest.funsuite.AnyFunSuite
import org.scalatest.matchers.should.Matchers.*

class RustAstGenAdjustmentsTests extends AnyFunSuite with RustAstGenTestFixture {

  test("a method receiver carries a borrow adjustment with source and target") {
    val srcFile = code(
      """
        |struct Foo;
        |impl Foo {
        |  fn value(&self) -> bool { true }
        |}
        |fn main() {
        |  let receiver = Foo;
        |  let _ = receiver.value();
        |}
        |""".stripMargin)

    val adjusted = allNodes(srcFile).filter(_.adjustments.isDefined)
    adjusted should have size 1

    val steps = adjusted.head.adjustments.getOrElse(fail("expected adjustments"))
    steps should have size 1

    val borrow = steps.head
    borrow shouldBe a[Borrow]
    borrow.source shouldBe "rust_ast_gen_scala_test::Foo"
    borrow.target shouldBe "&rust_ast_gen_scala_test::Foo"
  }

  test("a function-item-to-fn-pointer coercion carries a cast adjustment") {
    val srcFile = code(
      """
        |fn g() {}
        |fn main() {
        |  let _f: fn() = g;
        |}
        |""".stripMargin)

    val adjusted = allNodes(srcFile).filter(_.adjustments.isDefined)
    adjusted should have size 1

    val steps = adjusted.head.adjustments.getOrElse(fail("expected adjustments"))
    steps should have size 1

    val cast = steps.head
    cast shouldBe a[Cast]
    cast.source shouldBe "fn() -> ()"
    cast.target shouldBe "fn() -> ()"
  }

  test("a reborrow carries a deref followed by a borrow") {
    val srcFile = code(
      """
        |fn take_ref(_s: &i32) {}
        |fn main() {
        |  let mut n = 1i32;
        |  take_ref(&mut n);
        |}
        |""".stripMargin)

    val adjusted = allNodes(srcFile).filter(_.adjustments.isDefined)
    adjusted should have size 1

    val steps = adjusted.head.adjustments.getOrElse(fail("expected adjustments"))
    steps should have size 2

    val deref = steps.head
    deref shouldBe a[Deref]
    deref.source shouldBe "&mut i32"
    deref.target shouldBe "i32"

    val borrow = steps(1)
    borrow shouldBe a[Borrow]
    borrow.source shouldBe "i32"
    borrow.target shouldBe "&i32"
  }
}
