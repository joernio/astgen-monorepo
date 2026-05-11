import better.files.*

import scala.sys.process.*
import org.scalatest.matchers.should.Matchers
import org.scalatest.wordspec.AnyWordSpec
import SwiftNodeSyntax.SourceFileSyntax
import SwiftNodeSyntax._

import java.util.concurrent.ConcurrentLinkedQueue
import scala.jdk.CollectionConverters._

/** Round-trip tests that exercise the generated `SwiftNodeSyntax` Scala API against a
  * pre-built `SwiftAstGen` binary.
  *
  * The binary must exist next to `swift-astgen/` (named `SwiftAstGen-mac` /
  * `SwiftAstGen-linux` / `SwiftAstGen-win.exe`); see `Tests/ScalaSwiftNodeSyntaxTests/README.md`
  * for the full setup.
  */
class SwiftNodeSyntaxTest extends AnyWordSpec with Matchers {

  private val shellPrefix: Seq[String] =
    if (scala.util.Properties.isWin) "cmd" :: "/c" :: Nil else "sh" :: "-c" :: Nil

  private val executableName: String = {
    if (scala.util.Properties.isWin) "SwiftAstGen-win.exe"
    else if (scala.util.Properties.isMac) "SwiftAstGen-mac"
    else "SwiftAstGen-linux"
  }

  /** Resolves the absolute path of the pre-built `SwiftAstGen` binary located at the
    * repository root (i.e. `swift-astgen/<executable>`).
    */
  private def executablePath: String =
    (File(".").parent.parent / executableName).toJava.toPath.normalize.toAbsolutePath.toString

  /** Runs `command` in `cwd` and asserts a zero exit code. */
  private def run(command: String, cwd: String): Unit = {
    val stdOut = new ConcurrentLinkedQueue[String]
    val stdErr = new ConcurrentLinkedQueue[String]
    val logger = ProcessLogger(stdOut.add, stdErr.add)
    val exit   = Process(shellPrefix :+ command, new java.io.File(cwd)).!(logger)
    if (exit != 0) {
      val combined =
        s"""--- stdout ---
           |${stdOut.asScala.mkString(System.lineSeparator())}
           |--- stderr ---
           |${stdErr.asScala.mkString(System.lineSeparator())}""".stripMargin
      fail(s"Command `$command` (cwd=$cwd) exited with $exit\n$combined")
    }
  }

  /** Creates a temporary project containing a single `main.swift`, runs `SwiftAstGen` on it,
    * passes the parsed root `SourceFileSyntax` to `body`, and removes the temp directory.
    */
  private def withProject(swiftSource: String)(body: SourceFileSyntax => Unit): Unit = {
    val projectUnderTest: File = File.newTemporaryDirectory("swiftastgentests")
    try {
      val testFile = projectUnderTest / "main.swift"
      testFile.createIfNotExists(createParents = true)
      testFile.write(swiftSource)

      run(executablePath, projectUnderTest.pathAsString)

      val json     = ujson.read((projectUnderTest / "ast_out" / s"${testFile.name}.json").contentAsString)
      val rootNode = SwiftNodeSyntax.createSwiftNode(json).asInstanceOf[SourceFileSyntax]
      body(rootNode)
    } finally {
      projectUnderTest.delete(swallowIOExceptions = true)
    }
  }

  "Using the SwiftNodeSyntax API" should {

    "allow to grab a SourceFileSyntax node and its content" in {
      withProject("var x = 1") { sourceFileSyntax =>
        val Seq(codeBlock) = sourceFileSyntax.statements.children
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
      }
    }

    "allow to grab a binary expression with operator folding" in {
      withProject("1 + 2 * 3") { sourceFileSyntax =>
        val Seq(codeBlock) = sourceFileSyntax.statements.children
        codeBlock.item match {
          case v: InfixOperatorExprSyntax =>
            v.leftOperand shouldBe a[IntegerLiteralExprSyntax]
            v.leftOperand.asInstanceOf[IntegerLiteralExprSyntax].literal match {
              case integerLiteral(json) => json("tokenKind").str shouldBe """integerLiteral("1")"""
              case other                => fail("Should have a integerLiteral here but got: " + other)
            }
            v.operator shouldBe a[BinaryOperatorExprSyntax]
            v.operator.asInstanceOf[BinaryOperatorExprSyntax].operator match {
              case binaryOperator(json) => json("tokenKind").str shouldBe """binaryOperator("+")"""
              case other                => fail("Should have a binaryOperator here but got: " + other)
            }
            v.rightOperand match {
              case inner: InfixOperatorExprSyntax =>
                inner.leftOperand shouldBe a[IntegerLiteralExprSyntax]
                inner.leftOperand.asInstanceOf[IntegerLiteralExprSyntax].literal match {
                  case integerLiteral(json) => json("tokenKind").str shouldBe """integerLiteral("2")"""
                  case other                => fail("Should have a integerLiteral here but got: " + other)
                }
                inner.operator shouldBe a[BinaryOperatorExprSyntax]
                inner.operator.asInstanceOf[BinaryOperatorExprSyntax].operator match {
                  case binaryOperator(json) => json("tokenKind").str shouldBe """binaryOperator("*")"""
                  case other                => fail("Should have a binaryOperator here but got: " + other)
                }
                inner.rightOperand shouldBe a[IntegerLiteralExprSyntax]
                inner.rightOperand.asInstanceOf[IntegerLiteralExprSyntax].literal match {
                  case integerLiteral(json) => json("tokenKind").str shouldBe """integerLiteral("3")"""
                  case other                => fail("Should have a integerLiteral here but got: " + other)
                }
              case other => fail("Should have a InfixOperatorExprSyntax here but got: " + other)
            }
          case other => fail("Should have a InfixOperatorExprSyntax here but got: " + other)
        }
      }
    }

    "allow to grab node attributes correctly" in {
      withProject("var x = 1") { sourceFileSyntax =>
        val Seq(codeBlock) = sourceFileSyntax.statements.children
        codeBlock.item match {
          case v: VariableDeclSyntax =>
            v.bindings.children.head.pattern match {
              case ident: IdentifierPatternSyntax =>
                ident.identifier match {
                  case identifier: SwiftNode =>
                    identifier.startOffset.get shouldBe 4
                    identifier.endOffset.get shouldBe 5
                    identifier.startLine.get shouldBe 1
                    identifier.startColumn.get shouldBe 5
                    identifier.endLine.get shouldBe 1
                    identifier.endColumn.get shouldBe 6
                  case null => fail("Should have a token identifier here but got 'null'")
                }
              case other => fail("Should have a IdentifierPatternSyntax here but got: " + other)
            }
          case other => fail("Should have a VariableDeclSyntax here but got: " + other)
        }
      }
    }

  }

}
