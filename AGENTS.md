AGENTS instructions
===================

This is a monorepo of AST generators ("astgens") for the [Joern](https://joern.io) code analysis platform. Each
astgen parses source code of one language and writes JSON ASTs that a Joern frontend consumes. Every tool is
self-contained: its own toolchain, tests, CI workflows, version and release. The main developer documentation is the
[Contributor's guide](contributing-docs/README.md).

- [Architecture and repository layout](contributing-docs/01_architecture.md)
- [Development environment](contributing-docs/02_development_environment.md)

Always work inside ONE tool directory and use that tool's own commands. There is no root build, and running
everything is never the right answer.

| Tool | Directory | Toolchain | Build | Test |
|------|-----------|-----------|-------|------|
| ABAP | `abap-astgen/` | Bun | `bun install` | `bun run test` |
| C# / .NET | `dotnet-astgen/` | .NET 8 | `dotnet build` | `dotnet test` |
| Go | `go-astgen/` | Go | `go build ./...` | `go test ./...` |
| JavaScript / TS | `javascript-astgen/` | Bun | `bun install` | `bun run test` |
| Ruby | `ruby-astgen/` | Ruby / JRuby | `bundle install` | `bundle exec rake` |
| Rust | `rust-astgen/` | Rust (edition 2024) | `cargo build --release` | `cargo test --locked --workspace` |
| Swift | `swift-astgen/` | Swift 6.x | `swift build` | `swift test` |
| Regression framework | `astgen-regression/` | Python 3.12+ | `pip install -e .` | `pytest tests/ -v` |

Use `bun run test`, not `bun test`, for the Bun tools. The package script sets `--parallel`.

Pull request guideline
----------------------

- Follow [contributing-docs/07_pull_requests.md](contributing-docs/07_pull_requests.md).
- [ ] Title is `[tool-dir] Description`, for example `[javascript-astgen] Fix TSTypeCastExpression`.
- [ ] Before working on a bug fix, **reproduce the problem** with a failing test.
- [ ] After making the change, confirm that **it builds, the new test passes, and the problem is fixed**.
- [ ] Run the formatter and type checker of the tool you changed (see below).
- Write a short PR description and link the related issue, if any.

Coding style
------------

Follow [Coding style and best practices](contributing-docs/03_coding_style.md). CI enforces these checks:

| Tool | Command (run in the tool directory) |
|------|-------------------------------------|
| `abap-astgen`, `javascript-astgen` | `bun check` (type check) |
| `dotnet-astgen` | `dotnet format --verify-no-changes` (fix with `dotnet format`) |
| `rust-astgen` | `cargo fmt --check` (fix with `cargo fmt`) |
| `swift-astgen` | `swift-format lint --strict --recursive Sources/SwiftAstGen Sources/SwiftAstGenLib Tests/SwiftAstGenTests` |
| `astgen-regression` | `ruff check .`, `ruff format --check .`, `mypy .` |

- Match the style of the surrounding code. Do not reformat unrelated code.
- Do not hand-edit `Sources/CodeGeneration/` in `swift-astgen`. It is vendored from `swift-syntax`, see
  [common tasks](contributing-docs/08_common_tasks.md).
- Avoid inline comments that restate the code. Document intent.
- Code and tools must work on Linux, macOS and Windows where CI runs them on all three.

Tests
-----

Always add a test that verifies your change.

- [contributing-docs/04_tests.md](contributing-docs/04_tests.md)
- [contributing-docs/05_regression_tests.md](contributing-docs/05_regression_tests.md)

Each tool has its own unit tests (see the table above). `javascript-astgen`, `rust-astgen` and `swift-astgen` also
ship a generated Scala accessor file (`BabelNodeSyntax.scala`, `RustNodeSyntax.scala`, `SwiftNodeSyntax.scala`) with
sbt round-trip tests. Changing the JSON output of those tools usually means regenerating the Scala file, and
CI fails when the committed file for Rust is stale.

The output format is a contract with downstream Joern frontends. A change to the JSON shape (renamed or removed keys,
different node types) is a breaking change. Call it out in the PR description. Prefer additive changes.

Architecture
------------

- [contributing-docs/01_architecture.md](contributing-docs/01_architecture.md)
- [contributing-docs/06_releases.md](contributing-docs/06_releases.md)
- [contributing-docs/08_common_tasks.md](contributing-docs/08_common_tasks.md)

CI is routed per tool by [.github/monorepo-ci-config.yml](.github/monorepo-ci-config.yml): a PR only runs the
workflows of the directories it touches. Markdown-only changes run no workflows.
If you add a tool or a workflow, register it in `monorepo-ci-config.yml` and in
[monorepo-ci.yml](.github/workflows/monorepo-ci.yml).

Releases and generated files
----------------------------

- NEVER create or push release tags (`{tool}-astgen/v*`) unless the user explicitly asks. A pushed tag publishes
  a release.
- Version numbers live in a different file in each tool, see [releases](contributing-docs/06_releases.md). Use the
  tool's `bump-version.sh` or `update_version.rb` where one exists.
- NEVER commit built binaries (`astgen-*`, `abapgen-*`, `SwiftAstGen-*`, `build/`, `release/`), `node_modules`,
  `.build` or `target`. They are git-ignored. Check `git status` before committing.
- `BabelNodeSyntax.scala` and `SwiftNodeSyntax.scala` are generated and git-ignored. `RustNodeSyntax.scala` and
  `rust.ungram` are generated but **committed**.

Copyright
---------

- NEVER reproduce copyrighted material. Test fixtures must be written from scratch or be under a compatible license.
