Coding style and best practices
==============================

The tools are written in seven languages, and each follows the conventions of its own ecosystem. CI enforces some of
them automatically, the rest is by convention. This page lists both. The rule for everything: **match the
surrounding code, and do not reformat code you did not change.**

Run before you submit
---------------------

Run the commands for the tool you changed, in its directory. These are exactly the checks that CI runs.

| Tool | Check | Fix |
|------|-------|-----|
| `abap-astgen`, `javascript-astgen` | `bun check` | by hand |
| `dotnet-astgen` | `dotnet format --verify-no-changes` | `dotnet format` |
| `rust-astgen` | `cargo fmt --check` | `cargo fmt` |
| `swift-astgen` | `swift-format lint --strict --recursive Sources/SwiftAstGen Sources/SwiftAstGenLib Tests/SwiftAstGenTests` | `swift-format format -i --recursive <same paths>` |
| `astgen-regression` | `ruff check .`, `ruff format --check .`, `mypy .` | `ruff check --fix .`, `ruff format .` |
| `go-astgen`, `ruby-astgen` | none in CI | see below |

Per tool
--------

### TypeScript (`javascript-astgen`, `abap-astgen`)

- Bun runs the TypeScript sources directly, there is no compile step. `bun check` is the type check and is
  required in CI. `tsconfig.json` is strict and also sets `noUnusedLocals`, `noUnusedParameters` and
  `noImplicitReturns`, so unused code fails the check.
- No formatter is configured. Follow the existing style: 4 spaces, no semicolons, double quotes, no spaces inside
  `{}` in imports and object literals, and `import * as X` for namespaces.
- Document functions with TSDoc (`@returns`, ...) where the intent is not obvious.
- Use `node:` prefixes for built-in modules (`node:fs`, `node:path`).
- Tests use `bun:test`, see [tests](04_tests.md).

### Rust (`rust-astgen`)

- `cargo fmt`, default `rustfmt` settings, edition 2024. CI fails on any difference.
- The workspace is `rust-astgen/Cargo.toml` plus `crates/*`. Add dependencies to `[workspace.dependencies]` and
  reference them with `workspace = true`.
- The `ra_ap_*` crates are versioned together. Bump all of them in one change, see
  [common tasks](08_common_tasks.md).
- Builds and tests use `--locked`, so commit `Cargo.lock` changes.

### Swift (`swift-astgen`)

- [`.swift-format`](../swift-astgen/.swift-format) defines the style (4 spaces, 120 columns).
  `swift-format lint --strict` runs in CI on macOS.
- `Sources/CodeGeneration/` is vendored from `swift-syntax` and excluded from linting. Do not edit it, refresh it
  with `swift-astgen/scripts/refresh-codegen.sh` instead.

### C# (`dotnet-astgen`)

- Style comes from [.editorconfig](../dotnet-astgen/.editorconfig). `dotnet format --verify-no-changes` runs in CI.
- Targets .NET 8.

### Go (`go-astgen`)

- Standard Go conventions. No format check runs in CI, and some existing files are not `gofmt`-clean, so do not run
  `gofmt -w` on whole files in an unrelated change. Format the code you write.
- `go.mod` declares Go 1.18. Do not use newer language or library features without raising it in the PR.

### Ruby (`ruby-astgen`)

- No linter is configured. Follow the existing style. Newer files start with `# frozen_string_literal: true`.
- Code must run on both MRI and JRuby. Avoid C extensions and gems that do not support JRuby.

### Python (`astgen-regression`)

- `ruff` for lint and format, `mypy` for types, Python 3.12 features are fine. See
  [astgen-regression/AGENTS.md](../astgen-regression/AGENTS.md) for module responsibilities and error handling
  conventions.

General conventions
-------------------

- **Deterministic output.** Never let hash order, thread timing, the file system listing order, absolute paths or the
  current time reach the JSON output. Sort first. The regression framework diffs outputs, so noise becomes false
  alarms.
- **Stable output shape.** Keys and node type names are part of the contract with Joern, see
  [architecture](01_architecture.md). Keep them stable, and add rather than rename.
- **Never crash on bad input.** Report a problem with the file on stderr, skip that file, and continue.
- **Cross-platform.** Do not hard-code path separators, do not rely on file name case, and be careful with `\r\n`.
  Tools that CI runs on Windows (Go, Ruby, Rust, Swift) must keep working there.
- **No secrets, no machine-specific paths** in code, fixtures or generated files.
- Avoid inline comments that restate the code. Document the intent, and why a workaround exists.
- Keep error and log messages actionable: say what failed, on which file, and what to do.

Generated and vendored code
---------------------------

Do not edit these by hand. Change the generator or the input and regenerate, see
[common tasks](08_common_tasks.md).

- `rust-astgen/crates/scala_bindings_gen/rust.ungram` (copied from the resolved `ra_ap_syntax` crate by that
  crate's `build.rs`) and `rust-astgen/scala_tests/.../RustNodeSyntax.scala` (generated from it). Both are committed.
- `BabelNodeSyntax.scala`, `SwiftNodeSyntax.scala` (git-ignored, produced with `--scala-ast-only`)
- `javascript-astgen/src/version.ts` (from `package.json`) and `swift-astgen/Sources/SwiftAstGenLib/Version.swift`
  (from `VERSION`), see [releases](06_releases.md)
- `swift-astgen/Sources/CodeGeneration/` (vendored from `swift-syntax`)
- Lock files: `bun.lock`, `Cargo.lock`, `Package.resolved` change through the package manager only.
