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

SwiftAstGen includes regression tests to ensure AST output stability across code changes.

### Running Locally

The easiest way to run regression tests locally is with the automated runner:

```bash
python3 scripts/regression-local.py
```

This will automatically:
- Build the current branch in debug mode
- Checkout master in a temporary worktree
- Build master in debug mode
- Run the regression harness comparing both versions
- Clean up the temporary worktree

For manual control:

```bash
# Build both binaries
swift build                                      # Current branch
git worktree add .worktrees/master origin/master
(cd .worktrees/master && swift build)            # Master version

# Run regression tests
python3 scripts/regression.py \
  --base-dist .worktrees/master/.build/debug \
  --pr-dist .build/debug \
  --output-diffs ./regression-diffs
```

### CI Integration

Regression tests run automatically on all pull requests via GitHub Actions:
- Builds debug binaries for both master and PR branches
- Compares AST output against three Swift open-source projects (Alamofire, Vapor, RxSwift)
- Posts results as PR comments
- Uploads diff artifacts on failure

### Adding Test Cases

Add Swift files to `Tests/RegressionCorpus/` to expand coverage. Files should:
- Exercise diverse Swift syntax (operators, generics, macros, etc.)
- Be valid, compilable Swift code
- Cover edge cases or previously fixed bugs
