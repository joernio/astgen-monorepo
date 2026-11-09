Common tasks
============

Recipes for things that come up regularly. All commands run in the directory of the tool, unless noted.

Change what an astgen outputs
-----------------------------

The JSON is an interface to the Joern frontends, see [architecture](01_architecture.md#output-contract).

1. Write a test that fails: a small source snippet and the JSON you expect, see [tests](04_tests.md).
2. Make the change in the tool. Prefer adding a key over renaming or removing one.
3. If the tool has generated Scala accessors, check whether they change:
   - `javascript-astgen`: the wrapper is derived from `@babel/types` by `src/BabelSchema.ts` and rendered by
     `src/ScalaAstGenerator.ts`. Parser output that deviates from `@babel/types` is described by an explicit overlay in
     `BabelSchema.ts`. `test/BabelSchema.test.ts` validates real output against the model.
   - `swift-astgen`: `Sources/SwiftAstGenLib/ScalaAstGenerator.swift` renders the wrapper from the SwiftSyntax node
     hierarchy.
   - `rust-astgen`: the node kind names come from `crates/scala_bindings_gen/src/json_kind.rs`, which is shared by the
     JSON writer and the Scala generator so that they cannot drift. Regenerate `RustNodeSyntax.scala` (below).
4. Run the unit tests and the Scala round-trip suite of the tool.
5. For `javascript-astgen` and `swift-astgen`, run the [regression tests](05_regression_tests.md) and review the
   report. In the PR description, summarize what changed in the output and whether Joern needs a matching change.

Regenerate the Scala accessors
------------------------------

```bash
# javascript-astgen: writes ./BabelNodeSyntax.scala (git-ignored)
cd javascript-astgen && bun src/astgen.ts --scala-ast-only

# swift-astgen: writes ./SwiftNodeSyntax.scala (git-ignored)
cd swift-astgen && swift run SwiftAstGen --scala-ast-only

# rust-astgen: writes the committed RustNodeSyntax.scala, commit the result
cd rust-astgen && cargo run --locked -p scala_bindings_gen -- --include-date=false \
  --output scala_tests/src/main/scala/io/joern/rust2cpg/parser/RustNodeSyntax.scala
```

`--include-date=false` keeps the Rust file reproducible. CI regenerates it with that flag and fails if the committed
file differs.

Update a parser dependency
--------------------------

A parser upgrade can change the output for every user. Treat it like an output change: run the tests, regenerate the
accessors, and run the regression framework where available.

| Tool | Where | Notes |
|------|-------|-------|
| `javascript-astgen` | `package.json`: `@babel/parser`, `@babel/types`, `typescript` | Keep `@babel/parser` and `@babel/types` on the same version. Run `bun install`, commit `bun.lock`, regenerate `BabelNodeSyntax.scala`. The schema test shows where the parser output no longer matches the overlay in `BabelSchema.ts`. |
| `abap-astgen` | `package.json`: `@abaplint/core` | Run `bun install`, commit `bun.lock`. |
| `rust-astgen` | `[workspace.dependencies]` in `Cargo.toml`: the `ra_ap_*` crates | They are released together, so bump all to the same version in one change. The build script of `scala_bindings_gen` copies `rust.ungram` from the resolved `ra_ap_syntax`. Regenerate `RustNodeSyntax.scala`, and commit `Cargo.lock`, `rust.ungram` and the Scala file. |
| `swift-astgen` | `Package.swift` (`swift-syntax`) | Run `swift package resolve`, then `scripts/refresh-codegen.sh` (needs `jq`) to re-vendor `Sources/CodeGeneration/SyntaxSupport` for the pinned tag, then `swift build && swift test` and regenerate `SwiftNodeSyntax.scala`. Commit `Package.resolved`. |
| `ruby-astgen` | `Gemfile` (`prism`, `parser`) | Keep it working on MRI and JRuby. `spec/integration/prism_fallback_smoke.rb` guards the prism fallback. CI builds `libprism.dll` by hand for JRuby on Windows, so check that job too. |
| `dotnet-astgen` | `DotNetAstGen/DotNetAstGen.csproj` (Roslyn) | `dotnet test`. |
| `go-astgen` | `go.mod` | `go mod tidy`, commit `go.sum`. Go 1.18 is the declared minimum. |
| GitHub Actions | `.github/workflows/*.yml` | Dependabot proposes weekly grouped updates. Pin by commit SHA and keep the version in a trailing comment. |

Add support for another language
--------------------------------

Create a new tool directory, `{lang}-astgen/`, that is self-contained and follows the pattern of the others.

1. **Tool directory.** Add the code, tests, a `.gitignore` and a `README.md` with the same sections as the other tools:
   supported languages, building, testing, usage and an example. Add a `LICENSE` if the license differs from the root.
2. **CI workflow** `.github/workflows/{lang}-astgen-ci.yml` with `workflow_call` and `workflow_dispatch` triggers and
   `defaults.run.working-directory: {lang}-astgen`. Build, test, and add the format or lint check that fits the
   language. Pin actions by SHA.
3. **Release workflow** `.github/workflows/{lang}-astgen-release.yml`, triggered by tags `{lang}-astgen/v*` and by
   `workflow_dispatch`. Build binaries for Linux, macOS and Windows (x64 and arm64 where possible), compute
   checksums, create a **draft** release. Copy the closest existing one.
4. **Register it in CI routing**:
   - add an entry with its `paths` to [monorepo-ci-config.yml](../.github/monorepo-ci-config.yml);
   - add a `call-{lang}-ci` job, an entry in the `summary` job's `needs`, and its result handling to
     [monorepo-ci.yml](../.github/workflows/monorepo-ci.yml).
5. **Docs.** Add the tool to the table in the root [README](../README.md), to the tables in
   [AGENTS.md](../AGENTS.md), [architecture](01_architecture.md), [development environment](02_development_environment.md),
   [tests](04_tests.md), [coding style](03_coding_style.md) and [releases](06_releases.md).
6. **Optionally** add a regression setup, see below.

Add a regression setup for a tool
---------------------------------

```bash
cd {lang}-astgen
astgen-regression init --language {lang}
```

This creates `regression.yaml` and `.github/workflows/{lang}-astgen-regression.yml`. Then:

1. Edit `regression.yaml`: the build command, `execute.command` and the corpora (pinned to tags).
2. Make the generated workflow reusable (`on: workflow_call`) and set its `defaults.run.working-directory` like
   `javascript-astgen-regression.yml` does. It is called from `monorepo-ci.yml`, not triggered directly.
3. Register `{lang}-regression` in `monorepo-ci-config.yml` and add a `call-{lang}-regression` job in
   `monorepo-ci.yml` with `pull-requests: write` permission, as for JavaScript and Swift.
4. Run `astgen-regression local` once to check it.

Changing `astgen-regression`
----------------------------

See [astgen-regression/AGENTS.md](../astgen-regression/AGENTS.md). Add or change a language default in
`commands/init.py:LANGUAGE_DEFAULTS`, and keep the README and the templates in sync.
