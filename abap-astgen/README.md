# ABAP AST Generator

Generates JSON AST output for ABAP sources using [@abaplint/core](https://github.com/abaplint/abaplint). Intended as the AST provider for the `abap2cpg` frontend in [joern](https://joern.io).

## What it does

Reads `.abap` files from an input directory and writes, for each file, a JSON document containing:

- `file` — the input filename
- `objectType` — the abaplint object kind (e.g. `CLAS`, `PROG`)
- `statements` — a flat list of `{ type, tokens, start, end }` records, one per ABAP statement

No semantic interpretation is performed here; consumers (for example `AbapJsonParser.scala`) are responsible for turning statements into a higher-level AST.

## Building

```bash
npm install
npm run binary           # build binaries for all supported platforms
npm run binary:current   # build only for the current platform (macOS arm64 example)
```

Binaries are written to the project root.

## Usage

```bash
./abapgen-macos-arm64 <input-dir> <output-dir>
# or
node parse-abap.js <input-dir> <output-dir>
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
