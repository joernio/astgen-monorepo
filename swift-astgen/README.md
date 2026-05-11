# Swift AST Generator

Generates JSON Abstract Syntax Trees (ASTs) for Swift source files using
[SwiftSyntax](https://github.com/swiftlang/swift-syntax). Built primarily as the AST provider
for the Swift frontend in [Joern](https://joern.io).

## Supported languages

| Language | Tool used    | Notes                                                        |
| -------- | ------------ | ------------------------------------------------------------ |
| Swift    | SwiftSyntax  | Syntactic only — no name resolution, no type information.    |

The output JSON mirrors the SwiftSyntax tree (after operator folding) one node at a time.
Each token records its `tokenKind`, each layout node records its `nodeType`, every node
carries its source range, and the root node additionally records the file path, project root,
full source text, and line count.

## Project layout

| Path | Purpose |
| --- | --- |
| `Sources/SwiftAstGenLib/` | Library: file walking, parsing, JSON encoding, Scala AST generation. |
| `Sources/SwiftAstGen/` | Thin CLI wrapper around the library. |
| `Sources/CodeGeneration/` | **Vendored** subset of `swift-syntax`'s `CodeGeneration/SyntaxSupport`. Used by `ScalaAstGenerator` to introspect the SwiftSyntax node hierarchy. See [`Sources/CodeGeneration/README.md`](./Sources/CodeGeneration/README.md). |
| `Tests/SwiftAstGenTests/` | XCTest unit tests for the library. |
| `Tests/ScalaSwiftNodeSyntaxTests/` | sbt integration tests that exercise the generated `SwiftNodeSyntax.scala` API against a real CLI binary. See [`Tests/ScalaSwiftNodeSyntaxTests/README.md`](./Tests/ScalaSwiftNodeSyntaxTests/README.md). |
| `regression.yaml` | Configuration for [`astgen-regression`](../astgen-regression/). |

## Building

```bash
swift build              # debug build
swift build -c release   # release build
```

## Testing

```bash
swift test
```

The Swift unit tests are self-contained. The Scala round-trip tests under
`Tests/ScalaSwiftNodeSyntaxTests/` require a pre-built CLI binary; see their dedicated
[README](./Tests/ScalaSwiftNodeSyntaxTests/README.md).

## Linting

```bash
swift-format lint --strict --recursive \
  Sources/SwiftAstGen Sources/SwiftAstGenLib Tests/SwiftAstGenTests

swift-format format -i --recursive \
  Sources/SwiftAstGen Sources/SwiftAstGenLib Tests/SwiftAstGenTests
```

The same lint command runs in CI ([`swift-astgen-ci.yml`](../.github/workflows/swift-astgen-ci.yml))
and is enforced by `--strict`. The vendored `Sources/CodeGeneration/` tree is intentionally
excluded so it stays byte-identical with upstream `swift-syntax`.

## Usage

```
USAGE: SwiftAstGen [--src <src>] [--output <output>] [--pretty-print] [--scala-ast-only]

OPTIONS:
  -i, --src <src>          Source directory (default: `.`).
  -o, --output <output>    Output directory for generated AST json files (default: `./ast_out`).
  -p, --pretty-print       Pretty print the generated AST json files.
  -s, --scala-ast-only     Only print the generated Scala SwiftSyntax AST nodes
                           (writes `./SwiftNodeSyntax.scala`).
  -h, --help               Show help information.
```

The legacy camelCase flag forms (`--prettyPrint`, `--scalaAstOnly`) are accepted as aliases
for backward compatibility.

### `--scala-ast-only`

Writes a single Scala source file (`./SwiftNodeSyntax.scala`) describing the entire
SwiftSyntax node hierarchy as a sealed trait family. Downstream Scala consumers (notably the
Joern Swift frontend) use this wrapper to navigate the JSON ASTs produced in normal mode.
The generated file is shipped as a release artifact alongside the platform binaries.

The header carries a human-readable timestamp; the rest of the file is fully deterministic
for a given SwiftSyntax version.

### File skipping

`SwiftAstGen` skips paths that look like test/spec directories (`/test/`, `/tests/`,
`/spec/`, `/specs/`) and dot/underscore-prefixed directories (`/.`, `/__`). It also reads
the project's root `Package.swift` and skips every `testTarget(...)` path declared there.

## Examples

From a release binary on `PATH`:

```bash
cd <path to project>
SwiftAstGen
SwiftAstGen -i <path to project>
SwiftAstGen -i <path to project> -o <path to output directory>
SwiftAstGen --scala-ast-only
```

From a checkout (debug build, no install needed):

```bash
swift run SwiftAstGen -i <path to project> -o <path to output directory>
swift run SwiftAstGen --scala-ast-only
```

## Cross-compilation

Release binaries are produced by [`.github/workflows/swift-astgen-release.yml`](../.github/workflows/swift-astgen-release.yml)
for Linux (x86_64 + arm64), macOS (universal), and Windows (x86_64). The Linux builds use
the static [Swift Linux SDK](https://www.swift.org/install/macos/static-linux-sdk/):

```bash
swift sdk install https://download.swift.org/swift-6.1-release/static-sdk/swift-6.1-RELEASE/swift-6.1-RELEASE_static-linux-0.0.1.artifactbundle.tar.gz \
  --checksum 111c6f7d280a651208b8c74c0521dd99365d785c1976a6e23162f55f65379ac6
swift build --swift-sdk x86_64-swift-linux-musl -c release --static-swift-stdlib
swift build --swift-sdk aarch64-swift-linux-musl -c release --static-swift-stdlib
```

For local Mac universal builds:

```bash
swift build -c release --arch arm64 --arch x86_64
```

## Regression Testing

Regression testing compares AST output between two versions of the generator (base branch vs.
PR) across real-world Swift codebases.

Run locally (compares the current branch against `main`):

```bash
astgen-regression local
```

See [`astgen-regression/`](../astgen-regression/) for framework setup, CI integration, and
configuration details.
