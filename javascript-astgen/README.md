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

- Node.js >= 24.0.0 (for development/testing)
- No Node.js required for standalone binaries

## Key Dependencies

- **@babel/parser** 7.29.x — JavaScript/TypeScript parsing
- **TypeScript** 6.0.x — Type checking and analysis
- **readdirp** 5.0.x — Recursive directory traversal
- **yargs** 17.7.x — Command-line argument parsing

## Building

```bash
pnpm install
pnpm build
```

Platform-specific standalone binaries ([cross-platform SEAs](https://nodejs.org/api/single-executable-applications.html)) can be built using [pkg](https://github.com/yao-pkg/pkg):

```bash
pnpm binary
```

This creates standalone executables with embedded Node.js runtime for:
- Linux (x64, arm64)
- macOS (x64, arm64)
- Windows (x64)

**Note:** During the build process, you may see warnings about dynamic `require()` calls. These are harmless and do not affect the functionality of the binaries.

## Testing

```bash
pnpm install
pnpm build
pnpm test
```

This uses `jest` with `ts-jest` to run the tests in `test/`.

**Note:** Tests require Node.js with experimental VM modules support to handle ES module dynamic imports.

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

## File skip rules

Some files are skipped by design with a warning on stderr. None of these are configurable from the CLI today; if you need to scan one of the categories below, vendor the source or amend `src/Defaults.ts`.

| Trigger                                                  | Constant                          | Default        |
|----------------------------------------------------------|-----------------------------------|----------------|
| File size on disk                                         | `MAX_FILE_SIZE_BYTES`             | 5 MB           |
| Total lines in a file                                     | `MAX_LOC_IN_FILE`                 | 50 000         |
| Length of any single line                                 | `MAX_LINE_LENGTH`                 | 10 000 bytes   |
| File contains the marker `// EMSCRIPTEN_START_ASM`        | hard-coded                        | always skipped |
| File name matches `IGNORE_FILE_PATTERN`                   | `IGNORE_FILE_PATTERN`             | tests, specs, mocks, `*.min.*`, `*.d.ts`, `chunk-vendors`, etc. |
| Containing directory's basename is in `IGNORE_DIRS`       | `IGNORE_DIRS`                     | `node_modules`, `dist`, `build`, `test`, `tests`, `e2e`, `examples`, `cypress`, `i18n`, `flow-typed`, `vendor`, `www`, `venv`, `eslint-rules`, `codemods`, `docs`, `jest-cache`, `e2e-beta` |
| Hidden directories or files (`.` or `__` prefix)          | hard-coded                        | always skipped |

The full list of `IGNORE_DIRS` and the regex used by `IGNORE_FILE_PATTERN` are documented in [`src/Defaults.ts`](src/Defaults.ts).

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

The version lives in `package.json` and is mirrored into `src/version.ts` by `scripts/sync-version.mjs` (run automatically on `pnpm build` via the `prebuild` hook). Use `bump-version.sh` to bump:

```bash
./bump-version.sh 3.44.0
```
