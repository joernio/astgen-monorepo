# DotNetAstGen

This script creates an Abstract Syntax Tree (AST) of all `.cs` files in JSON format. The AST is created by using the Roslyn .NET compiler.

## Supported languages & dialects

| Language    | Tool used                   | Notes                           |
| ----------- | --------------------------- | ------------------------------- |
| .NET        | Roslyn                      | WIP |

## Getting Started

Use `dotnet run --project DotNetAstGen -i <target_source> -o <target_output>` to compile and run the
project. This will generate JSON AST files from the given source code.

For parsing symbols of DLL files, one needs the DLL and PDB file (of the same name) to live together
in the same directory. The names of the symbol summaries will be the name of the DLL file with the
prefix `_Symbols.json` and will live in the output directory.

## Releasing

To build a single all-in-one executable, make use of `./publish-release.sh <os> <arch>`, e.g.

```Bash
./publish-release.sh linux x64
./publish-release.sh linux arm
./publish-release.sh osx x64
./publish-release.sh win x64
```

This will create an executable, e.g., `./release/linux/DotNetAstGen`.

## Getting Help

TODO

## Example

TODO
