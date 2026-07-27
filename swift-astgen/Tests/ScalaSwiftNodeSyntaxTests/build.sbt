ThisBuild / version := "0.1.0-SNAPSHOT"

ThisBuild / scalaVersion := "3.8.4"

lazy val copyFile = taskKey[Unit]("Copy SwiftNodeSyntax.scala")

copyFile := {
  val from = baseDirectory.value / ".." / ".." / "SwiftNodeSyntax.scala"
  val to   = baseDirectory.value / "src" / "main" / "scala" / "SwiftNodeSyntax.scala"
  IO.delete(to)
  IO.copyFile(from, to)
}

lazy val root = (project in file("."))
  .settings(name := "ScalaSwiftNodeSyntaxTests")

libraryDependencies ++= Seq(
  "com.lihaoyi"          %% "ujson"        % "4.4.3",
  "com.github.pathikrit" %% "better-files" % "3.9.2"  % Test,
  "org.scalatest"        %% "scalatest"    % "3.2.20" % Test
)
