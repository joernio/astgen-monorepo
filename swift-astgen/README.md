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
USAGE: swift-ast-gen [--src <src>] [--output <output>] [--prettyPrint]

OPTIONS:
  -i, --src <src>         Source directory (default: `.`).
  -o, --output <output>   Output directory for generated AST json files (default: `./ast_out`).
  -p, --prettyPrint       Pretty print the generated AST json files (default: `false`).
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
