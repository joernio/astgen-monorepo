import better.files.*

import scala.sys.process.*
import org.scalatest.matchers.should.Matchers
import org.scalatest.wordspec.AnyWordSpec
import SwiftNodeSyntax.SourceFileSyntax
import SwiftNodeSyntax._

import java.util.concurrent.ConcurrentLinkedQueue
import scala.jdk.CollectionConverters._

class SwiftNodeSyntaxTest extends AnyWordSpec with Matchers {

  private val shellPrefix: Seq[String] =
    if (scala.util.Properties.isWin) "cmd" :: "/c" :: Nil else "sh" :: "-c" :: Nil

  private def run(command: String, cwd: String): Unit = {
    val stdOutOutput     = new ConcurrentLinkedQueue[String]
    val stdErrOutput     = new ConcurrentLinkedQueue[String]
    val processLogger = ProcessLogger(stdOutOutput.add, stdErrOutput.add)
    Process(shellPrefix :+ command, new java.io.File(cwd)).!(processLogger) match {
      case 0      => println("Exit Code '0': " + stdOutOutput.asScala.mkString(System.lineSeparator()))
      case other  => println(s"Error Code '$other': " + stdErrOutput.asScala.mkString(System.lineSeparator()))
    }
  }

  private val executableName: String = {
    if (scala.util.Properties.isWin)
      "SwiftAstGen-win.exe"
    else if (scala.util.Properties.isMac)
      "SwiftAstGen-mac"
    else
      "SwiftAstGen-linux"
  }

  private def runSwiftAstGen(projectUnderTest: File): Unit = {
    val path = (File(".").parent.parent / executableName).toJava.toPath.normalize.toAbsolutePath.toString
    println("Running: " + path)
    run(path, projectUnderTest.pathAsString)
  }

  "Using the SwiftNodeSyntax API" should {

    "allow to grab a SourceFileSyntax node and its content" in {
      val projectUnderTest: File = File.newTemporaryDirectory("swiftastgentests")
      val testFile = projectUnderTest / "main.swift"
      val testContent = "var x = 1"
      testFile.createIfNotExists(createParents = true)
      testFile.write(testContent)
      runSwiftAstGen(projectUnderTest)

      val lines            = (projectUnderTest / "ast_out" / s"${testFile.name}.json").contentAsString
      val json             = ujson.read(lines)
      val sourceFileSyntax = SwiftNodeSyntax.createSwiftNode(json).asInstanceOf[SourceFileSyntax]

      val Seq(codeBlock)   = sourceFileSyntax.statements.children
      codeBlock.item match {
        case v: VariableDeclSyntax =>
          v.bindings.children.head.pattern match {
            case ident: IdentifierPatternSyntax =>
              ident.identifier match {
                case identifier(json) => json("tokenKind").str shouldBe "identifier(\"x\")"
                case other            => fail("Should have a token identifier here but got: " + other)
              }
            case other => fail("Should have a IdentifierPatternSyntax here but got: " + other)
          }
        case other => fail("Should have a VariableDeclSyntax here but got: " + other)
      }

      projectUnderTest.delete(swallowIOExceptions = true)
    }

    "allow to grab a binary expression with operator folding" in {
      val projectUnderTest: File = File.newTemporaryDirectory("swiftastgentests")
      val testFile = projectUnderTest / "main.swift"
      val testContent = "1 + 2 * 3"
      testFile.createIfNotExists(createParents = true)
      testFile.write(testContent)
      runSwiftAstGen(projectUnderTest)

      val lines            = (projectUnderTest / "ast_out" / s"${testFile.name}.json").contentAsString
      val json             = ujson.read(lines)
      val sourceFileSyntax = SwiftNodeSyntax.createSwiftNode(json).asInstanceOf[SourceFileSyntax]
      
      val Seq(codeBlock)   = sourceFileSyntax.statements.children
      codeBlock.item match {
        case v: InfixOperatorExprSyntax =>
          val leftExpr = v.leftOperand
          val op = v.operator
          val rightExpr = v.rightOperand

          leftExpr shouldBe a[IntegerLiteralExprSyntax]
          leftExpr.asInstanceOf[IntegerLiteralExprSyntax].literal match {
            case integerLiteral(json) => json("tokenKind").str shouldBe """integerLiteral("1")"""
            case other => fail("Should have a integerLiteral here but got: " + other)
          }
          op shouldBe a[BinaryOperatorExprSyntax]
          op.asInstanceOf[BinaryOperatorExprSyntax].operator match {
            case binaryOperator(json) => json("tokenKind").str shouldBe """binaryOperator("+")"""
            case other => fail("Should have a binaryOperator here but got: " + other)
          }
          rightExpr match {
            case v: InfixOperatorExprSyntax =>
              val leftExpr = v.leftOperand
              val op = v.operator
              val rightExpr = v.rightOperand

              leftExpr shouldBe a[IntegerLiteralExprSyntax]
              leftExpr.asInstanceOf[IntegerLiteralExprSyntax].literal match {
                case integerLiteral(json) => json("tokenKind").str shouldBe """integerLiteral("2")"""
                case other => fail("Should have a integerLiteral here but got: " + other)
              }
              op shouldBe a[BinaryOperatorExprSyntax]
              op.asInstanceOf[BinaryOperatorExprSyntax].operator match {
                case binaryOperator(json) => json("tokenKind").str shouldBe """binaryOperator("*")"""
                case other => fail("Should have a binaryOperator here but got: " + other)
              }
              rightExpr shouldBe a[IntegerLiteralExprSyntax]
              rightExpr.asInstanceOf[IntegerLiteralExprSyntax].literal match {
                case integerLiteral(json) => json("tokenKind").str shouldBe """integerLiteral("3")"""
                case other => fail("Should have a integerLiteral here but got: " + other)
              }
            case other => fail("Should have a InfixOperatorExprSyntax here but got: " + other)
          }
        case other => fail("Should have a InfixOperatorExprSyntax here but got: " + other)
      }

      projectUnderTest.delete(swallowIOExceptions = true)
    }

  }

}
