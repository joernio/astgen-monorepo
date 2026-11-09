Architecture
============

An **astgen** is a small command line program. It walks a source directory, parses every file of one language
with an existing parser, and writes one JSON document per file (plus extras such as type maps). A
[Joern](https://github.com/joernio/joern) frontend runs the astgen binary, reads the JSON, and builds the Code
Property Graph from it. This repository builds and releases those binaries.

```
source files ──> astgen binary ──> JSON ASTs ──> Joern frontend ──> CPG
                 (this repo)                     (joernio/joern)
```

Joern downloads released binaries from the [Releases page](https://github.com/joernio/astgen-monorepo/releases).
The astgen version a frontend uses is pinned on the Joern side, so a release here is not picked up automatically.

Tools
-----

Every directory below is independent: its own toolchain, dependencies, tests, version, CI and release workflow.
Nothing is shared at build time.

| Tool | Directory | Language | Parser / backend | Binary |
|------|-----------|----------|------------------|--------|
| ABAP | `abap-astgen/` | TypeScript on Bun | `@abaplint/core` | `abapgen` |
| C# / .NET | `dotnet-astgen/` | C# | Roslyn | `DotNetAstGen` |
| Go | `go-astgen/` | Go | Go standard library (`go/parser`) | `goastgen` |
| JavaScript / TypeScript / Vue | `javascript-astgen/` | TypeScript on Bun | Babel, plus the TypeScript compiler API for types | `astgen` |
| Ruby | `ruby-astgen/` | Ruby, runs on JRuby | `prism`, with a fallback to the `parser` gem | `ruby_ast_gen` |
| Rust | `rust-astgen/` | Rust | `rust-analyzer` libraries | `rust_ast_gen` |
| Swift | `swift-astgen/` | Swift | SwiftSyntax | `SwiftAstGen` |
| Regression framework | `astgen-regression/` | Python | n/a | `astgen-regression` |

`astgen-regression` is not an astgen. It is the shared test tool described in
[regression tests](05_regression_tests.md).

Output contract
---------------

The JSON that an astgen writes is read by code in another repository. The shape of the output is therefore an
interface:

- Do not rename or remove keys, node types or fields without coordinating with the Joern frontend. Prefer additions.
- Output must be **deterministic**: the same input and the same version produce byte-identical output. Sort anything
  that comes from a hash map or a directory listing. The [regression tests](05_regression_tests.md) diff the output
  of two builds and rely on this.
- Parsers are fed untrusted, often broken code. A syntax error in one file should not abort the run or drop the
  other files.

Generated Scala accessors
-------------------------

Three tools can also emit a Scala 3 wrapper (case classes, one per node type) so that Joern frontends do not have to
navigate the JSON by key:

| Tool | Generated file | How it is produced | Committed? |
|------|----------------|--------------------|------------|
| `javascript-astgen` | `BabelNodeSyntax.scala` | `astgen --scala-ast-only` | No (git-ignored) |
| `swift-astgen` | `SwiftNodeSyntax.scala` | `SwiftAstGen --scala-ast-only` | No (git-ignored) |
| `rust-astgen` | `scala_tests/src/main/scala/io/joern/rust2cpg/parser/RustNodeSyntax.scala` | `cargo run -p scala_bindings_gen`, from `crates/scala_bindings_gen/rust.ungram` (copied from `ra_ap_syntax` by `build.rs`) | **Yes** |

Each of them has an sbt project that compiles the generated file and decodes real astgen output with it:
`javascript-astgen/scala-tests`, `swift-astgen/Tests/ScalaSwiftNodeSyntaxTests` and `rust-astgen/scala_tests`.
The release workflows regenerate the wrappers and publish them as release artifacts next to the binaries.
See [tests](04_tests.md).

Repository layout
-----------------

```
.github/
  workflows/                 {tool}-astgen-ci.yml, -release.yml, -regression.yml, monorepo-ci.yml
  monorepo-ci-config.yml     which paths trigger which workflow
  scripts/detect-changes.js  reads the config and the git diff, emits the workflows to run
contributing-docs/           this guide
AGENTS.md, CLAUDE.md         entry point for coding agents (CLAUDE.md is a symlink)
{tool}-astgen/               one self-contained tool each
astgen-regression/           regression testing framework (Python)
```

CI routing
----------

[monorepo-ci.yml](../.github/workflows/monorepo-ci.yml) runs on every pull request. Its first job,
`detect-changes`, runs [detect-changes.js](../.github/scripts/detect-changes.js), which compares the diff against
the base branch with the path globs in [monorepo-ci-config.yml](../.github/monorepo-ci-config.yml). Only the
workflows of the touched tools are called. Changes to `monorepo-ci.yml` or the config run **all** of them, and
changes that only touch `**/*.md`, `docs/**`, `LICENSE` or `.gitignore` run **none**.

The per-tool `*-ci.yml` workflows are reusable (`workflow_call`) and can also be started by hand
(`workflow_dispatch`). Release workflows are triggered by tags, see [releases](06_releases.md).

All GitHub Actions are pinned to a commit SHA, and Dependabot updates them weekly.
