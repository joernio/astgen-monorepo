package io.joern.rust2cpg.parser

import io.joern.rust2cpg.parser.RustNodeSyntax.{RustNode, SourceFile, createRustNode}

import java.nio.file.{Files, Path, Paths}
import java.util.Comparator
import scala.sys.process.Process
import scala.util.Using
import ujson.Value

trait RustAstGenTestFixture {

  def code(source: String): SourceFile = {
    val projectDir = Files.createTempDirectory("rust-ast-gen-scala-tests")

    try {
      val srcDir = Files.createDirectories(projectDir.resolve("src"))
      val outputDir = projectDir.resolve("out")

      Files.writeString(
        projectDir.resolve("Cargo.toml"),
        """[package]
          |name = "rust_ast_gen_scala_test"
          |version = "0.1.0"
          |edition = "2021"
          |""".stripMargin
      )
      Files.writeString(srcDir.resolve("main.rs"), source)

      val exitCode = Process(
        Seq(
          "cargo",
          "run",
          "--quiet",
          "--release",
          "--bin",
          "rust_ast_gen",
          "--",
          "-i",
          projectDir.toString,
          "-o",
          outputDir.toString
        ),
        repoRoot.toFile
      ).!

      require(exitCode == 0, s"rust_ast_gen failed with exit code $exitCode")

      val json = ujson.read(Files.readString(outputDir.resolve("src").resolve("main.json")))
      RustNodeSyntax.createRustNode(sourceFileJson(json)).asInstanceOf[SourceFile]
    } finally {
      deleteRecursively(projectDir)
    }
  }

  private def sourceFileJson(json: Value): Value =
    json("children")(0)

  private lazy val repoRoot: Path = Paths.get(".")

  private def deleteRecursively(path: Path): Unit = {
    if (Files.exists(path)) {
      Using.resource(Files.walk(path)) { paths =>
        paths.sorted(Comparator.reverseOrder()).forEach(path => Files.deleteIfExists(path))
      }
    }
  }

  private def childNodes(node: RustNode): Seq[RustNode] = {
    node.json.obj.get("children").map(_.arr.toSeq).getOrElse(Seq.empty).map(createRustNode)
  }

  private def prettyPrintNode(node: RustNode, indent: Int): String = {
    val renderedChildren = childNodes(node).map(child => prettyPrintNode(child, indent + 1))
    (Seq(("  " * indent) + node.json("nodeKind").str) ++ renderedChildren).mkString(System.lineSeparator())
  }

  extension (node: RustNode)
    def prettyPrint: String = prettyPrintNode(node, 0)
}
