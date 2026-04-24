# Swift AST Generator

Generates JSON Abstract Syntax Trees (ASTs) for Swift source files using [SwiftSyntax](https://github.com/swiftlang/swift-syntax).

## Supported languages

| Language | Tool used    | Notes                           |
| -------- | ------------ | ------------------------------- |
| Swift    | SwiftSyntax  | no types / call full names etc. |

## Building

```bash
swift build
```

## Testing

```bash
swift test
```

## Usage

```
USAGE: swift-ast-gen [--src <src>] [--output <output>] [--prettyPrint] [--scalaAstOnly]

OPTIONS:
  -i, --src <src>         Source directory (default: `.`).
  -o, --output <output>   Output directory for generated AST json files (default: `./ast_out`).
  -p, --prettyPrint       Pretty print the generated AST json files.
  -s, --scalaAstOnly      Only print the generated Scala SwiftSyntax AST nodes.
  -h, --help              Show help information.
```

## Example

Navigate to the project and run `SwiftAstGen`:

```bash
cd <path to project>
SwiftAstGen
```

To specify the path to the project or the output directory:

```bash
SwiftAstGen -i <path to project>
SwiftAstGen -i <path to project> -o <path to output directory>
```

## Regression Testing

Regression testing compares AST output between two versions of the generator (base branch vs. PR) across real-world Swift codebases.

Run locally (compares the current branch against `main`):

```bash
astgen-regression local
```

See [`astgen-regression/`](../astgen-regression/) for framework setup, CI integration, and configuration details.
