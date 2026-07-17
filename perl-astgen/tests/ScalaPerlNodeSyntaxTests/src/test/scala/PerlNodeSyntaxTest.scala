import better.files.*

import scala.sys.process.*
import org.scalatest.matchers.should.Matchers
import org.scalatest.wordspec.AnyWordSpec
import PerlNodeSyntax.SourceFile
import PerlNodeSyntax.*

import java.util.concurrent.ConcurrentLinkedQueue
import scala.jdk.CollectionConverters.*

/** Round-trip tests that exercise the generated `PerlNodeSyntax` Scala API against a
  * pre-built `perl-astgen` binary.
  *
  * The binary must exist inside `perl-astgen/` (named `perl-astgen-macos` /
  * `perl-astgen-linux` / `perl-astgen-win.exe`); see
  * `tests/ScalaPerlNodeSyntaxTests/README.md` for the full setup.
  */
class PerlNodeSyntaxTest extends AnyWordSpec with Matchers {

  private val shellPrefix: Seq[String] =
    if (scala.util.Properties.isWin) "cmd" :: "/c" :: Nil else "sh" :: "-c" :: Nil

  private val executableName: String = {
    if (scala.util.Properties.isWin) "perl-astgen-win.exe"
    else if (scala.util.Properties.isMac) "perl-astgen-macos"
    else "perl-astgen-linux"
  }

  /** Resolves the absolute path of the pre-built `perl-astgen` binary located in
    * the `perl-astgen/` directory.
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

  /** Creates a temporary directory containing a single `test.pl`, runs `perl-astgen` on it,
    * passes the parsed root `SourceFile` to `body`, and removes the temp directory.
    */
  private def withFile(perlSource: String)(body: SourceFile => Unit): Unit = {
    val projectUnderTest: File = File.newTemporaryDirectory("perlastgentests")
    try {
      val testFile  = projectUnderTest / "test.pl"
      val outputDir = projectUnderTest / "ast_out"
      testFile.createIfNotExists(createParents = true)
      testFile.write(perlSource)

      val cmd = s"$executablePath -i ${testFile.pathAsString} -o ${outputDir.pathAsString}"
      run(cmd, projectUnderTest.pathAsString)

      val json     = ujson.read((outputDir / "test.pl.json").contentAsString)
      val rootNode = PerlNodeSyntax.createPerlNode(json).asInstanceOf[SourceFile]
      body(rootNode)
    } finally {
      projectUnderTest.delete(swallowIOExceptions = true)
    }
  }

  "Using the PerlNodeSyntax API" should {

    "allow to grab a SourceFile node and navigate a variable declaration" in {
      withFile("my $x = 1;") { sourceFile =>
        val Seq(exprStmt) = sourceFile.children
        exprStmt match {
          case es: ExpressionStatement =>
            es.children.head match {
              case assign: AssignmentExpression =>
                assign.left match {
                  case Some(varDecl: VariableDeclaration) =>
                    varDecl.variable match {
                      case Some(scalar: Scalar) =>
                        scalar.children.head match {
                          case varname: Varname =>
                            varname.text shouldBe Some("x")
                          case other => fail("Expected Varname but got: " + other)
                        }
                      case other => fail("Expected Scalar but got: " + other)
                    }
                  case other => fail("Expected VariableDeclaration but got: " + other)
                }
              case other => fail("Expected AssignmentExpression but got: " + other)
            }
          case other => fail("Expected ExpressionStatement but got: " + other)
        }
      }
    }

    "allow to grab a binary expression and its operator" in {
      withFile("1 + 2;") { sourceFile =>
        val Seq(exprStmt) = sourceFile.children
        exprStmt match {
          case es: ExpressionStatement =>
            es.children.head match {
              case bin: BinaryExpression =>
                bin.left match {
                  case Some(left: Number) => left.text shouldBe Some("1")
                  case other              => fail("Expected Number on left but got: " + other)
                }
                bin.operator shouldBe "+"
                bin.right match {
                  case Some(right: Number) => right.text shouldBe Some("2")
                  case other               => fail("Expected Number on right but got: " + other)
                }
              case other => fail("Expected BinaryExpression but got: " + other)
            }
          case other => fail("Expected ExpressionStatement but got: " + other)
        }
      }
    }

    "allow to grab node position attributes correctly" in {
      withFile("my $x = 1;") { sourceFile =>
        val Seq(exprStmt) = sourceFile.children
        exprStmt match {
          case es: ExpressionStatement =>
            es.children.head match {
              case assign: AssignmentExpression =>
                assign.left match {
                  case Some(varDecl: VariableDeclaration) =>
                    varDecl.variable match {
                      case Some(scalar: Scalar) =>
                        scalar.startByte    shouldBe 3
                        scalar.endByte      shouldBe 5
                        scalar.startRow     shouldBe 0
                        scalar.startColumn  shouldBe 3
                        scalar.endRow       shouldBe 0
                        scalar.endColumn    shouldBe 5
                      case other => fail("Expected Scalar but got: " + other)
                    }
                  case other => fail("Expected VariableDeclaration but got: " + other)
                }
              case other => fail("Expected AssignmentExpression but got: " + other)
            }
          case other => fail("Expected ExpressionStatement but got: " + other)
        }
      }
    }

  }

}
