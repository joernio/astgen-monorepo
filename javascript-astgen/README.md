# JavaScript AST Generator

Generates JSON Abstract Syntax Trees (ASTs) for JavaScript, TypeScript, Vue, JSX, and TSX sources using the [Babel](https://babeljs.io/) parser. Type maps are generated using the TypeScript compiler / type checker API.

This package is **CLI-only** — install it for the `astgen` binary; programmatic imports of `@joernio/astgen` are not supported.

## Supported languages

| Language   | Tool used | Notes         |
| ---------- | --------- | ------------- |
| JavaScript | Babel     | types via tsc |
| TypeScript | Babel     | types via tsc |
| Vue        | Babel     |               |
| JSX        | Babel     |               |
| TSX        | Babel     |               |

## Requirements

- [Bun](https://bun.sh) >= 1.4 (for development/testing)
- No runtime required for standalone binaries

## Key Dependencies

- **@babel/parser** 7.29.x — JavaScript/TypeScript parsing
- **TypeScript** 6.0.x — Type checking and analysis
- **readdirp** 5.0.x — Recursive directory traversal
- **yargs** 17.7.x — Command-line argument parsing

## Building

```bash
bun install
```

Bun executes TypeScript directly — no separate compile step is needed.

Platform-specific standalone binaries can be built using `bun build --compile`:

```bash
bun run binary
```

This cross-compiles standalone executables with an embedded Bun runtime for all targets in parallel:
- Linux (x64, arm64)
- macOS (x64, arm64)
- Windows (x64, arm64)

## Testing

```bash
bun install
bun run test
```

## Usage

```
Options:
  -i, --src            Source directory                           [default: "."]
  -o, --output         Output directory for generated AST json files
                       (default: <src>/ast_out)                         [string]
  -t, --type           Project type. Default auto-detect                [string]
  -r, --recurse        Recurse mode suitable for mono-repos
                                                       [boolean] [default: true]
      --tsTypes        Generate type mappings using the Typescript Compiler API
                                                       [boolean] [default: true]
      --exclude-file   Exclude this file. Can be specified multiple times.
                       Default is empty.                   [array] [default: []]
      --exclude-regex  Exclude files matching this regex (matches the absolute
                       path).
  -s, --scala-ast-only Only write the Scala wrapper for the Babel AST nodes
                       (./BabelNodeSyntax.scala) and exit; no sources are
                       parsed.                        [boolean] [default: false]
      --version        Show version number                             [boolean]
  -h                   Show help                                       [boolean]
```

## Example

Navigate to the project and run `astgen`:

```bash
cd <path to project>
astgen
```

To specify the project type and the path to the project:

```bash
astgen -t js -i <path to project>
astgen -t vue -i <path containing .vue files>
```

Exclude individual files or whole directories with `--exclude-file` (repeatable, accepts both file and directory paths) or with `--exclude-regex` (matched against the absolute path):

```bash
astgen --exclude-file generated/ --exclude-file build/legacy.js
astgen --exclude-regex '\.generated\.[jt]s$'
```

## Scala accessors

The JSON AST follows the node definitions of `@babel/types`. `astgen --scala-ast-only` (or `-s`) writes `./BabelNodeSyntax.scala`, a Scala 3 file (depends on `ujson`) with one case class per Babel node type, so consumers do not have to navigate the JSON by key:

```scala
import BabelNodeSyntax.*

val file = BabelAstFile(ujson.read(os.read(path))).ast       // File
file.program.body.foreach {
  case fn: FunctionDeclaration => println(fn.id.map(_.name))  // Option[Identifier]
  case _                       =>
}
```

Alias groups (`Expression`, `Statement`, ...) are sealed traits, union-typed fields use Scala 3 union types, optional fields are `Option`, and array fields are always a `Seq` (a `Vector`, so indexing is cheap).

Things to know when using it:

- **Unknown node types** (say, from a newer parser than the one the file was generated for) decode to `UnknownNode`, which extends every alias trait so it fits wherever an alias such as `Expression` is expected. Consequently, an unknown statement also matches `case e: Expression`; test for `UnknownNode` first if that matters. Fields typed as concrete node classes (or unions of them) cannot hold it: reading one throws an `IllegalStateException` naming the node type and field. Missing or null required fields fail the same way.
- **Unexpected parser output never crashes with anything but a descriptive error.** Accessors throw only an `IllegalStateException` (a value of the wrong kind, such as a number where a string is expected, or an unexpected node) or a `NoSuchElementException` (a required key is missing); both name the node type and field. No `ujson` exception escapes. `BabelNodeSyntaxTest` fuzzes this with thousands of mutated ASTs.
- **Equality** is identity of the wrapped JSON object: nodes can be used as map keys, and reading the same child twice yields equal nodes. Structurally identical nodes at different places are not equal.
- `NumericLiteral.value` is `None` for numbers JSON cannot hold (`1e999`); the source text is in `extra("raw")`.
- Alias traits such as `Class`, `Function` or `File` shadow the standard names under `import BabelNodeSyntax.*`; import the ones you need by name.

The generated file is attached to each release as `BabelNodeSyntax.scala`; `scala-tests/` contains an sbt project that checks it against real `astgen` output.

## File skip rules

Some files are skipped by design with a warning on stderr. None of these are configurable from the CLI today; if you need to scan one of the categories below, vendor the source or amend `src/Defaults.ts`.

| Trigger                                                  | Constant                          | Default        |
|----------------------------------------------------------|-----------------------------------|----------------|
| File size on disk                                         | `MAX_FILE_SIZE_BYTES`             | 5 MB           |
| Total lines in a file                                     | `MAX_LOC_IN_FILE`                 | 50 000         |
| Length of any single line                                 | `MAX_LINE_LENGTH`                 | 10 000 bytes   |
| File contains the marker `// EMSCRIPTEN_START_ASM`        | hard-coded                        | always skipped |
| File name matches `IGNORE_FILE_PATTERN`                   | `IGNORE_FILE_PATTERN`             | tests, specs, mocks, `*.min.*`, `*.d.ts`, `chunk-vendors`, etc. Keywords must be a whole word: `foo.test.js` and `foo_test.js` are skipped, `latest.js` is not. |
| File name matches `IGNORE_TOOLING_FILE_PATTERN`           | `IGNORE_TOOLING_FILE_PATTERN`     | bundler output (`bundle.js`, `*.chunk.js`), tooling configuration (`webpack.*.js`, `jest.config*`, `rollup.config*`, `babel.config.js`, `Gruntfile.js`, ...), `*.types.js`, `*.cjs.js`, `*.t.ts` |
| Containing directory's basename is in `IGNORE_DIRS`       | `IGNORE_DIRS`                     | `node_modules`, `dist`, `build`, `test`, `tests`, `e2e`, `examples`, `cypress`, `i18n`, `flow-typed`, `vendor`, `www`, `venv`, `eslint-rules`, `codemods`, `docs`, `jest-cache`, `e2e-beta` |
| Hidden directories or files (`.` or `__` prefix)          | hard-coded                        | always skipped |

The full list of `IGNORE_DIRS` and the regexes used by `IGNORE_FILE_PATTERN` and `IGNORE_TOOLING_FILE_PATTERN` are documented in [`src/Defaults.ts`](src/Defaults.ts).

## Regression Testing

Regression testing compares AST and type-map output between two versions of the generator (base branch vs. PR) across real-world TypeScript corpora (e.g., [typeorm](https://github.com/typeorm/typeorm), [fastify](https://github.com/fastify/fastify)).

Run locally (compares the current branch against `main`):

```bash
astgen-regression local
```

See [`astgen-regression/`](../astgen-regression/) for framework setup, CI integration, and configuration details.

## Architecture

The CLI entry point is [`src/astgen.ts`](src/astgen.ts), which delegates to [`src/Pipeline.ts`](src/Pipeline.ts). Other modules:

- [`Parsing.ts`](src/Parsing.ts) — Babel-based JS/TS/Vue parsing.
- [`Writers.ts`](src/Writers.ts) — derives output paths and writes via a `WriteSink`.
- [`WriteSink.ts`](src/WriteSink.ts) — abstraction over filesystem writes; the default `FsWriteSink` deduplicates `mkdir -p` calls and streams JSON.
- [`FileUtils.ts`](src/FileUtils.ts) — recursive directory traversal, exclusion rules, content validation.
- [`TscUtils.ts`](src/TscUtils.ts) — TypeScript type extraction.
- [`VueCodeCleaner.ts`](src/VueCodeCleaner.ts) — strips Vue SFC wrapping so the script body parses as JS.
- [`JsonUtils.ts`](src/JsonUtils.ts) — buffered streaming JSON serializer with circular-reference handling.
- [`Logger.ts`](src/Logger.ts) — module-level logging seam (default forwards to `console`); tests can swap implementations.
- [`Errors.ts`](src/Errors.ts) — typed errors thrown by the pipeline (the CLI translates them to exit codes).

## Releasing

The version lives in `package.json` and is mirrored into `src/version.ts` by `scripts/sync-version.ts` (run automatically on `bun run binary` via the `prebinary` hook). Use `bump-version.sh` to bump:

```bash
./bump-version.sh <bumped-version-tag>
```
