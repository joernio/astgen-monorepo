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
# This writes ./SwiftNodeSyntax.scala (which the sbt build compiles together with the tests).
```

## Running the tests

```bash
cd Tests/ScalaSwiftNodeSyntaxTests
sbt "testFull"
```

The build copies `../../SwiftNodeSyntax.scala` into its managed sources on every compilation, so a
regenerated file is always picked up.

Use `testOnly *` (or `testFull`) rather than `test`: sbt 2's `test` skips suites that passed before, and
it cannot see that the generated file or the `SwiftAstGen` binary changed.

## Layout

- `build.sbt` &mdash; sbt project; compiles the generated `../../SwiftNodeSyntax.scala` together with the tests.
- `project/build.properties` &mdash; sbt version pin.
- `src/test/scala/SwiftNodeSyntaxTest.scala` &mdash; the actual test suite.

## CI

The `swift-astgen-ci.yml` workflow builds the binaries, generates the wrapper, and runs
`sbt testFull` here.
