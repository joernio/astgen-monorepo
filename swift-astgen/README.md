# AST generator

This script creates Abstract Syntax Tree (AST) of all .swift files in JSON format.
The AST is created by using SwiftSyntax.

## Supported languages

| Language    | Tool used                   | Notes                           |
| ----------- | --------------------------- | ------------------------------- |
| Swift       | SwiftSyntax                 | no types / call full names etc. |

## Building

```bash
> swift build
```

## Testing

```bash
> swift test
```

## Getting Help

```bash
> SwiftAstGen -h
USAGE: swift-ast-gen [--src <src>] [--output <output>] [--prettyPrint] [--scalaAstOnly]

OPTIONS:
  -i, --src <src>         Source directory (default: `.`).
  -o, --output <output>   Output directory for generated AST json files (default: `./ast_out`).
  -p, --prettyPrint       Pretty print the generated AST json files.
  -s, --scalaAstOnly      Only print the generated Scala SwiftSyntax AST nodes.
  -h, --help              Show help information.
```

## Example

Navigate to the project and run `SwiftAstGen`.

```bash
> cd <path to project>
> SwiftAstGen
```

To specify the path to the project or the output directory.

```bash
> SwiftAstGen -i <path to project>
> SwiftAstGen -i <path to project> -o <path to output directory>
```

## Regression Testing

Regression testing compares AST output between two versions of SwiftAstGen (base branch vs. PR) across real-world Swift codebases to ensure output stability across code changes.

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
# From the swift-astgen directory
astgen-regression local
```

This command automatically:
- Creates a git worktree for the base branch
- Builds both versions (base and PR) in debug mode
- Runs SwiftAstGen on configured test corpora
- Generates a detailed Markdown report with diffs
- Cleans up the worktree

**Manual comparison** of two pre-built distributions:

```bash
astgen-regression compare \
  --base-dist .worktrees/master/.build/debug \
  --pr-dist .build/debug \
  --base-ref "main @ abc1234" \
  --pr-ref "feature @ def5678" \
  --output-diffs ./regression-diffs
```

The report shows:
- AST file counts and total sizes
- Wall-clock execution time
- Per-file content diffs (collapsible)

**CI Integration:**

Regression tests run automatically on all pull requests via GitHub Actions (`.github/workflows/swift-astgen-regression.yml`):
- Builds debug binaries for both main and PR branches
- Compares AST output against configured Swift open-source projects
- Posts results as PR comments
- Uploads diff artifacts on failure

**Configuration:**

Test corpora are configured in `regression.yaml`. To add or modify test cases:
1. Edit `regression.yaml` to add new corpus entries
2. Specify git repository URL and tag/commit
3. Configure any required build or input subdirectories
