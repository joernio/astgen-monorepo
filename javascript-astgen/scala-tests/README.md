# scala-tests

Round-trip tests that exercise the generated `BabelNodeSyntax.scala` API against a real `astgen`
binary. The tests:

1. Spawn the `astgen` binary on temporary JS/TS/TSX projects (and on the shared syntax fixtures in
   `../test/fixtures/syntax`).
2. Read the resulting JSON ASTs.
3. Decode them through `BabelNodeSyntax.createBabelNode(...)` (the Scala wrapper produced by
   `astgen --scala-ast-only`).
4. Assert the structure with ScalaTest matchers, and force **every** generated accessor on **every**
   node of the fixtures, so a field that is required in the schema but absent in real output fails
   the build.

## Prerequisites

These tests run an **external `astgen`** and compile the **generated** `../BabelNodeSyntax.scala`;
`sbt test` on its own fails unless that file exists.

From the `javascript-astgen/` directory:

```bash
bun install

# 1. Generate the Scala wrapper into the project root (./BabelNodeSyntax.scala).
bun src/astgen.ts --scala-ast-only

# 2. Make an `astgen` executable available. Either build the release binaries ...
bun run binary
mv astgen-linux-x64 astgen-linux         # or astgen-macos / astgen-macos-arm / astgen-win.exe

# ... or, for local development, point ASTGEN_BIN at a tiny wrapper around the sources:
printf '#!/bin/sh\nexec bun "%s/src/astgen.ts" "$@"\n' "$PWD" > /tmp/astgen && chmod +x /tmp/astgen
export ASTGEN_BIN=/tmp/astgen
```

## Running the tests

```bash
cd scala-tests
sbt "testFull"
```

The build copies `../BabelNodeSyntax.scala` into its managed sources on every compilation, so a
regenerated file is always picked up.

Use `testOnly *` (or `testFull`) rather than `test`: sbt 2's `test` skips suites that passed before, and
it cannot see that the generated file, the fixtures or the `astgen` binary changed.

## Layout

- `build.sbt` &mdash; sbt project; compiles the generated `../BabelNodeSyntax.scala` together with the tests.
- `project/build.properties` &mdash; sbt version pin.
- `src/test/scala/BabelNodeSyntaxTest.scala` &mdash; the actual test suite.

## CI

The `javascript-astgen-ci.yml` workflow builds the binaries, generates the wrapper, and runs
`sbt testFull` here.
