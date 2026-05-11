# ScalaSwiftNodeSyntaxTests

Round-trip tests that exercise the generated `SwiftNodeSyntax.scala` API against a real
`SwiftAstGen` binary. The tests:

1. Spawn the platform-specific `SwiftAstGen` binary on a temporary `main.swift`.
2. Read the resulting JSON AST.
3. Decode it through `SwiftNodeSyntax.createSwiftNode(...)` (the Scala wrapper produced by
   `swift run SwiftAstGen --scala-ast-only`).
4. Assert the structure with ScalaTest matchers.

## Prerequisites

These tests run an **external pre-built binary**; `sbt test` on its own will fail unless the
binary and the generated Scala source are in place.

From the `swift-astgen/` directory:

```bash
# 1. Build the native binary and rename it for the current platform.
swift build
mv .build/debug/SwiftAstGen SwiftAstGen-mac          # macOS
# or
mv .build/debug/SwiftAstGen SwiftAstGen-linux        # Linux
# or
mv .build/debug/SwiftAstGen.exe SwiftAstGen-win.exe  # Windows

# 2. Generate the Scala wrapper into the repository root.
./SwiftAstGen-mac --scala-ast-only
# This writes ./SwiftNodeSyntax.scala (which sbt copies into src/main/scala on `sbt compile`).
```

## Running the tests

```bash
cd Tests/ScalaSwiftNodeSyntaxTests
sbt test
```

The `copyFile` task in `build.sbt` runs as a `compile` dependency and pulls
`swift-astgen/SwiftNodeSyntax.scala` into `src/main/scala/SwiftNodeSyntax.scala`.

## Layout

- `build.sbt` &mdash; sbt project + `copyFile` task.
- `project/build.properties` &mdash; sbt version pin.
- `src/main/scala/.keep` &mdash; placeholder; the real source is copied in at build time.
- `src/test/scala/SwiftNodeSyntaxTest.scala` &mdash; the actual test suite.

## CI

The `swift-astgen-ci.yml` workflow performs steps (1)+(2) before invoking `sbt test` here.
