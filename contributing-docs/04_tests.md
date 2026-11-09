Tests
=====

Always add a test that verifies your change. For a bug fix, write the failing test first and then fix the code.
Every tool has its own unit tests, and three tools have an additional Scala round-trip suite. There is no root test
command. Run the tests of the tool you changed, in its directory.

Unit tests per tool
-------------------

| Tool | Run (in the tool directory) | Framework | Tests live in |
|------|-----------------------------|-----------|---------------|
| `abap-astgen` | `bun run test` | `bun:test` | `test/abapgen.test.ts` |
| `javascript-astgen` | `bun run test` | `bun:test` | `test/*.test.ts`, fixtures in `test/fixtures/` |
| `dotnet-astgen` | `dotnet test` | xUnit | `DotNetAstGen.Test/` |
| `go-astgen` | `go test ./...` | `testing` | `*_test.go` next to the code, and `main_test.go` |
| `ruby-astgen` | `bundle exec rake` (runs `spec`) | RSpec | `spec/`, `spec/integration/` |
| `rust-astgen` | `cargo test --locked --workspace` | cargo test | unit tests in `src/`, integration tests in `tests/` |
| `swift-astgen` | `swift test` | XCTest | `Tests/SwiftAstGenTests/` |
| `astgen-regression` | `pytest tests/ -v` | pytest | `tests/test_*.py` |

Use `bun run test` and not `bun test`: the package script adds `--parallel`.

Run a single test:

```bash
bun test test/Vue.test.ts                       # javascript-astgen
bun test -t "name of the test"                  # filter by name
dotnet test --filter "FullyQualifiedName~Name"
go test ./goastgen -run TestName
bundle exec rspec spec/ruby_ast_gen_spec.rb:42
cargo test --locked --test cfg_resolution
swift test --filter SwiftAstGenTests
pytest tests/test_config.py::test_name -v
```

### Writing a test

Test through the astgen as a user would: put some source code in a temporary project, run the generator, and assert
on the resulting JSON.

- `javascript-astgen`: `test/helpers.ts` has `withTmpProject` to create a project from source descriptors and
  run the pipeline. Shared syntax samples are in `test/fixtures/syntax`, and they are also used by the Scala suite.
- `rust-astgen`: `tests/common/mod.rs` runs `rust_ast_gen` on a temporary crate and parses the JSON back.
  Most tests use `--no-sysroot` because loading the sysroot is slow.
- Write test inputs from scratch. Do not paste code from other projects unless its license allows it.
- Test one behaviour per test, with a name that says which. Cover the failure path too: broken syntax, empty
  directories, unreadable files, non-UTF-8 input.
- Tests must not depend on the network, on the current directory, or on the order in which tests run.
- Make assertions on the parts you care about. Do not compare the whole JSON of a large file, it makes the test
  brittle.

Scala round-trip tests
----------------------

`javascript-astgen`, `rust-astgen` and `swift-astgen` generate a Scala wrapper for their JSON output
(see [architecture](01_architecture.md)). These sbt projects compile the wrapper and decode real astgen output with
it. They also force **every** accessor on **every** node of the fixtures, so a field that the generated code
considers required but that is missing in real output fails the build.

They need a JDK 21 and sbt. Use `testFull` (or `testOnly *`) instead of `test`: sbt 2 skips suites that passed before
and cannot see that the generated file, a fixture or the astgen binary changed.

### javascript-astgen

Directory `javascript-astgen/scala-tests`. It needs a generated `../BabelNodeSyntax.scala` and an `astgen`
executable.

```bash
cd javascript-astgen
bun install
bun src/astgen.ts --scala-ast-only        # writes ./BabelNodeSyntax.scala
# make an astgen executable available, either a built binary named astgen-linux / astgen-macos /
# astgen-macos-arm / astgen-win.exe in this directory (bun run binary), or a wrapper script:
printf '#!/bin/sh\nexec bun "%s/src/astgen.ts" "$@"\n' "$PWD" > /tmp/astgen && chmod +x /tmp/astgen
export ASTGEN_BIN=/tmp/astgen
cd scala-tests && sbt testFull
```

See [javascript-astgen/scala-tests/README.md](../javascript-astgen/scala-tests/README.md).

### swift-astgen

Directory `swift-astgen/Tests/ScalaSwiftNodeSyntaxTests`. It needs a pre-built binary named for the platform
(`SwiftAstGen-mac`, `SwiftAstGen-linux` or `SwiftAstGen-win.exe`) in `swift-astgen/`, and the generated
`SwiftNodeSyntax.scala`:

```bash
cd swift-astgen
swift build
mv .build/debug/SwiftAstGen SwiftAstGen-mac     # or SwiftAstGen-linux
./SwiftAstGen-mac --scala-ast-only              # writes ./SwiftNodeSyntax.scala
cd Tests/ScalaSwiftNodeSyntaxTests && sbt testFull
```

See [its README](../swift-astgen/Tests/ScalaSwiftNodeSyntaxTests/README.md). The renamed binary is git-ignored.

### rust-astgen

Directory `rust-astgen/scala_tests`. The test fixture calls `cargo run --bin rust_ast_gen` itself, so only the Rust
toolchain needs to be installed. Unlike the other two, the generated `RustNodeSyntax.scala` is **committed**, and CI
regenerates it and fails if it differs:

```bash
cd rust-astgen
cargo run --locked -p scala_bindings_gen -- --include-date=false \
  --output scala_tests/src/main/scala/io/joern/rust2cpg/parser/RustNodeSyntax.scala
git status                                       # must show no unexpected changes
cd scala_tests && sbt test
```

What CI runs
------------

| Tool | CI workflow | Runs |
|------|-------------|------|
| `abap-astgen` | `abap-astgen-ci.yml` | `bun check`, `bun run test`, `bun run binary` |
| `javascript-astgen` | `javascript-astgen-ci.yml` | `bun check`, `bun run test`, `bun run binary`, then the Scala suite |
| `dotnet-astgen` | `dotnet-astgen-ci.yml` | `dotnet build`, `dotnet test`, `dotnet format --verify-no-changes` |
| `go-astgen` | `go-astgen-ci.yml` | `./build.sh` (`go build ./...`, then `go test ./...`) on Linux, macOS and Windows |
| `ruby-astgen` | `ruby-astgen-ci.yml` | `bundle exec rake` on MRI, and `jruby -S bundle exec rake` on JRuby, on all three OSes; plus a JDK mismatch smoke test |
| `rust-astgen` | `rust-astgen-ci.yml` | `cargo test --locked --workspace` on all three OSes, bindings regeneration check, Scala tests (Linux), `cargo fmt --check` |
| `swift-astgen` | `swift-astgen-ci.yml` | `swift build`, `swift test`, `swift-format lint` (macOS), Scala tests, on all three OSes |
| `astgen-regression` | `astgen-regression-ci.yml` | `ruff`, `mypy`, `pytest --cov` |

Real-world regression runs for `javascript-astgen` and `swift-astgen` are described in
[regression tests](05_regression_tests.md).
