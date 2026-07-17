# ScalaPerlNodeSyntaxTests

Round-trip tests that exercise the generated `PerlNodeSyntax.scala` API against a real
`perl-astgen` binary. The tests:

1. Spawn the platform-specific `perl-astgen` binary on a temporary `.pl` file.
2. Read the resulting JSON AST.
3. Decode it through `PerlNodeSyntax.createPerlNode(...)` (the Scala wrapper produced by
   `perl-astgen --scala-ast-only`).
4. Assert the structure with ScalaTest matchers.

## Prerequisites

These tests run an **external pre-built binary**; `sbt test` on its own will fail unless the
binary and the generated Scala source are in place.

From the `perl-astgen/` directory:

```bash
# 1. Build the native binary and rename it for the current platform.
cargo build --release
cp target/release/perl-astgen perl-astgen-macos    # macOS
# or
cp target/release/perl-astgen perl-astgen-linux    # Linux
# or
cp target/release/perl-astgen.exe perl-astgen-win.exe  # Windows

# 2. Generate the Scala wrapper into the repository root.
cargo run --release -- --scala-ast-only
# This writes ./PerlNodeSyntax.scala (which sbt copies into src/main/scala on `sbt compile`).
```

## Running the tests

```bash
cd tests/ScalaPerlNodeSyntaxTests
sbt test
```

The `copyFile` task in `build.sbt` runs as a `compile` dependency and pulls
`perl-astgen/PerlNodeSyntax.scala` into `src/main/scala/PerlNodeSyntax.scala`.

## Layout

- `build.sbt` &mdash; sbt project + `copyFile` task.
- `project/build.properties` &mdash; sbt version pin.
- `src/main/scala/.keep` &mdash; placeholder; the real source is copied in at build time.
- `src/test/scala/PerlNodeSyntaxTest.scala` &mdash; the actual test suite.

## CI

The `perl-astgen-ci.yml` workflow performs steps (1)+(2) before invoking `sbt test` here.
