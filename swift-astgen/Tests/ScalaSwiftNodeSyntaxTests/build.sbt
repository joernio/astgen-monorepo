ThisBuild / version := "0.1.0-SNAPSHOT"

ThisBuild / scalaVersion := "3.3.1"

lazy val copyFile = taskKey[Unit]("Copy SwiftNodeSyntax.scala")

copyFile := {
  val from = baseDirectory.value / ".." / ".." / "SwiftNodeSyntax.scala"
  val to   = baseDirectory.value / "src" / "main" / "scala" / "SwiftNodeSyntax.scala"
  IO.delete(to)
  IO.copyFile(from, to)
}

Compile / compile := ((Compile / compile) dependsOn copyFile).value

lazy val root = (project in file("."))
  .settings(name := "ScalaSwiftNodeSyntaxTests")

libraryDependencies ++= Seq(
  "com.lihaoyi"          %% "ujson"        % "3.1.3",
  "com.github.pathikrit" %% "better-files" % "3.9.2"  % Test,
  "org.scalatest"        %% "scalatest"    % "3.2.17" % Test
)
