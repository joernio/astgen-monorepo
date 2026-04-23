# AST generator

This script creates Abstract Syntax Tree (AST) of all JS/TS files in JSON format.
The AST is created by using the bundled babel parser (for JavaScript, TypeScript).
Type maps are generated using the Typescript compiler / type checker API.

## Supported languages

| Language   | Tool used                   | Notes         |
| ---------- | --------------------------- | ------------- |
| JavaScript | babel                       | types via tsc |
| TypeScript | babel                       | types via tsc |
| Vue        | babel                       |               |
| JSX        | babel                       |               |
| TSX        | babel                       |               |

## Usage

## Building

```bash
yarn install
yarn build
```

Platform-specific binaries can now be build using [pkg](https://github.com/yao-pkg/pkg):

```bash
yarn binary
```

## Testing

```bash
yarn install
yarn build
yarn test
```

This will use `jest` with `ts-jest` to run the tests in `test/`.

## Regression Testing

Regression testing compares AST and type-map output between two versions of astgen (base branch vs. PR) across real-world TypeScript corpora including [typeorm@0.3.21](https://github.com/typeorm/typeorm) and [fastify@v5.3.3](https://github.com/fastify/fastify).

**Prerequisites:**
```bash
# Install the regression testing framework from the monorepo root
cd ../astgen-regression
python -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate
pip install -r requirements.txt
pip install -e .
```

**Run locally** (compares current branch against `main`):

```bash
# From the javascript-astgen directory
astgen-regression local
```

This command automatically:
- Creates a git worktree for the base branch
- Builds both versions (base and PR)
- Runs astgen on configured test corpora
- Generates a detailed Markdown report with diffs
- Cleans up the worktree

**Manual comparison** of two pre-built distributions:

```bash
astgen-regression compare \
  --base-dist ./dist-base \
  --pr-dist ./dist-pr \
  --base-ref "main @ abc1234" \
  --pr-ref "feature @ def5678"
```

The report shows:
- AST and typemap file counts and total sizes
- Wall-clock execution time
- Per-file content diffs (collapsible)

**CI:** The regression workflow runs automatically on every pull request (`.github/workflows/javascript-astgen-regression.yml`) and posts or updates a comment on the PR with the full report.

## Getting Help

```bash
./astgen -h
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

Navigate to the project and run `astgen` command.

```bash
cd <path to project>
astgen
```

To specify the project type and the path to the project.

```bash
astgen -t js -i <path to project>
astgen -t vue -i <path containing .vue files>
```
