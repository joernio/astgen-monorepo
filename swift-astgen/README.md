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

## Getting Help

```bash
> SwiftAstGen -h
Options:
  -i, --src      Source directory                                 [default: "."]
  -o, --output   Output directory for generated AST json files
                                                            [default: "ast_out"]
  -h             Show help                                             [boolean]
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
