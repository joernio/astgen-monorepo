import better.files.*

import scala.sys.process.*
import org.scalatest.matchers.should.Matchers
import org.scalatest.wordspec.AnyWordSpec
import org.scalatest.BeforeAndAfterAll
import SwiftNodeSyntax.SourceFileSyntax
import SwiftNodeSyntax._

import java.util.concurrent.ConcurrentLinkedQueue
import scala.jdk.CollectionConverters._

class SwiftNodeSyntaxTest extends AnyWordSpec with Matchers with BeforeAndAfterAll {

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

  private val testFiles: Map[String, String] = Map("main.swift" -> "var x = 1")

  private val projectUnderTest: File = {
    val dir = File.newTemporaryDirectory("swiftastgentests")
    testFiles.foreach { case (testFile, content) =>
      val file = dir / testFile
      file.createIfNotExists(createParents = true)
      file.write(content)
    }
    dir
  }

  private def runSwiftAstGen(): Unit = {
    val path = (File(".").parent.parent / executableName).toJava.toPath.normalize.toAbsolutePath.toString
    println("Running: " + path)
    run(path, projectUnderTest.pathAsString)
  }

  override def beforeAll(): Unit = runSwiftAstGen()

  override def afterAll(): Unit = projectUnderTest.delete(swallowIOExceptions = true)

  "Using the SwiftNodeSyntax API" should {

    "allow to grab a SourceFileSyntax node and its content" in {
      testFiles.foreach { case (testFile, _) =>
        val lines            = (projectUnderTest / "ast_out" / s"$testFile.json").contentAsString
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
      }
    }

  }

}
