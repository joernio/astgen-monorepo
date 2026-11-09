ThisBuild / version := "0.1.0-SNAPSHOT"

ThisBuild / scalaVersion := "3.8.4"

// The generated `../../SwiftNodeSyntax.scala` (see README.md) is the code under test: copy it into the build on every
// compilation. The generator must not be cached (sbt 2 would skip it as its inputs do not change).
Compile / sourceGenerators += Def.uncached(Def.task {
  val from = baseDirectory.value / ".." / ".." / "SwiftNodeSyntax.scala"
  val to   = (Compile / sourceManaged).value / "SwiftNodeSyntax.scala"
  IO.copyFile(from, to)
  Seq(to)
})

lazy val root = (project in file("."))
  .settings(name := "ScalaSwiftNodeSyntaxTests")

libraryDependencies ++= Seq(
  "com.lihaoyi"          %% "ujson"        % "4.4.3",
  "com.github.pathikrit" %% "better-files" % "3.9.2"  % Test,
  "org.scalatest"        %% "scalatest"    % "3.2.20" % Test
)
