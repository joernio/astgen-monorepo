# JavaScript AST Generator

Generates JSON Abstract Syntax Trees (ASTs) for JavaScript, TypeScript, Vue, JSX, and TSX sources using the [Babel](https://babeljs.io/) parser. Type maps are generated using the TypeScript compiler / type checker API.

## Supported languages

| Language   | Tool used | Notes         |
| ---------- | --------- | ------------- |
| JavaScript | Babel     | types via tsc |
| TypeScript | Babel     | types via tsc |
| Vue        | Babel     |               |
| JSX        | Babel     |               |
| TSX        | Babel     |               |

## Requirements

- Node.js >= 18.0.0 (for development/testing)
- No Node.js required for standalone binaries

## Key Dependencies

- **@babel/parser** 7.29.x - JavaScript/TypeScript parsing
- **TypeScript** 6.0.x - Type checking and analysis
- **readdirp** 5.0.x - Recursive directory traversal
- **yargs** 17.7.x - Command-line argument parsing

## Building

```bash
yarn install
yarn build
```

Platform-specific standalone binaries can be built using [esbuild](https://esbuild.github.io/) and [pkg](https://github.com/yao-pkg/pkg):

```bash
yarn binary
```

This creates standalone executables with embedded Node.js runtime for:
- Linux (x64, arm64)
- macOS (x64, arm64) 
- Windows (x64)

**Note:** During the build process, you may see warnings about dynamic `require()` calls. These are harmless and do not affect the functionality of the binaries.

## Testing

```bash
yarn install
yarn build
yarn test
```

This uses `jest` with `ts-jest` to run the tests in `test/`.

**Note:** Tests require Node.js with experimental VM modules support to handle ES module dynamic imports.

## Usage

```
Options:
  -v, --version  Print version number                                  [boolean]
  -i, --src      Source directory                                 [default: "."]
  -o, --output   Output directory for generated AST json files
                                                            [default: "ast_out"]
  -t, --type     Project type. Default auto-detect
  -r, --recurse  Recurse mode suitable for mono-repos  [boolean] [default: true]
  -h             Show help                                             [boolean]
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

## Regression Testing

Regression testing compares AST and type-map output between two versions of the generator (base branch vs. PR) across real-world TypeScript corpora (e.g., [typeorm](https://github.com/typeorm/typeorm), [fastify](https://github.com/fastify/fastify)).

Run locally (compares the current branch against `main`):

```bash
astgen-regression local
```

See [`astgen-regression/`](../astgen-regression/) for framework setup, CI integration, and configuration details.
