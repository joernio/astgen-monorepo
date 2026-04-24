# .NET AST Generator

Generates JSON Abstract Syntax Trees (ASTs) for `.cs` source files using the [Roslyn](https://github.com/dotnet/roslyn) .NET compiler. Also extracts symbol summaries from `.dll` assemblies when paired with their `.pdb` files.

## Supported languages

| Language   | Tool used | Notes |
| ---------- | --------- | ----- |
| C# / .NET  | Roslyn    | WIP   |

## Building

```bash
dotnet build
```

To build a single all-in-one executable, use `./publish-release.sh <os> <arch>`:

```bash
./publish-release.sh linux x64
./publish-release.sh linux arm
./publish-release.sh osx x64
./publish-release.sh win x64
```

This creates an executable at `./release/<os>/DotNetAstGen`.

## Testing

```bash
dotnet test
```

## Usage

```
Options:
  -d, --debug     Enable verbose output.
  -i, --input     Input file or directory. Ingested file types are `.cs`, `.dll`, and `.pdb`. (required)
  -o, --output    Output directory. (default: `./.ast`)
  -e, --exclude   Exclusion regex for files to filter out.
```

For parsing symbols of DLL files, both the `.dll` and `.pdb` file (with matching names) must live in the same directory. Symbol summaries are written to the output directory with a `_Symbols.json` suffix.

## Example

Run directly via `dotnet run`:

```bash
dotnet run --project DotNetAstGen -- -i <target_source> -o <target_output>
```

Run the published binary:

```bash
./release/linux/DotNetAstGen -i <target_source> -o <target_output>
```
