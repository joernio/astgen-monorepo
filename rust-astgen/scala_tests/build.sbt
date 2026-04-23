
ThisBuild / scalaVersion := "3.7.4"

lazy val root = (project in file("."))
  .settings(
    name := "scala_tests",
    // Versions chosen to match the versions used by joern.
    libraryDependencies ++= Seq(
      "com.lihaoyi" %% "upickle" % "4.0.2",
      "org.scalatest" %% "scalatest" % "3.2.18" % Test
    )
  )
