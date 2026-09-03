# ABAP AST Generator

Generates JSON AST output for ABAP sources using [@abaplint/core](https://github.com/abaplint/abaplint). Intended as the AST provider for the `abap2cpg` frontend in [joern](https://joern.io).

## What it does

Reads `.abap` files from an input directory and writes, for each file, a JSON document containing:

- `file` — the input filename
- `objectType` — the abaplint object kind (e.g. `CLAS`, `PROG`)
- `statements` — a flat list of `{ type, tokens, start, end }` records, one per ABAP statement

No semantic interpretation is performed here; consumers (for example `AbapJsonParser.scala`) are responsible for turning statements into a higher-level AST.

## Requirements

- [Bun](https://bun.sh) >= 1.4 (for development/testing)
- No runtime required for standalone binaries

## Building

```bash
bun install
```

Bun executes TypeScript directly — no separate compile step is needed.

Platform-specific standalone binaries can be built using `bun build --compile`:

```bash
bun run binary
```

This cross-compiles standalone executables with an embedded Bun runtime for all targets in parallel. Binaries are written to the project root.

## Testing

```bash
bun install
bun run test
```

## Usage

```bash
./abapgen-macos-arm64 <input-dir> <output-dir>
# or
bun src/parse-abap.ts <input-dir> <output-dir>
```

Each parsed file produces `<output-dir>/<filename>.json`. The process prints one `OK <path>` or `ERR <path>` line per input file.

## Supported platforms

| OS      | Architecture | Binary                   |
| ------- | ------------ | ------------------------ |
| Linux   | x64          | `abapgen-linux-x64`      |
| Linux   | arm64        | `abapgen-linux-arm64`    |
| macOS   | x64          | `abapgen-macos-x64`      |
| macOS   | arm64        | `abapgen-macos-arm64`    |
| Windows | x64          | `abapgen-win-x64.exe`    |
| Windows | arm64        | `abapgen-win-arm64.exe`  |
