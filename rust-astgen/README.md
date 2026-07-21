# Rust AST Generator

Generates JSON AST representations of Rust source files, built on top of [rust-analyzer](https://github.com/rust-lang/rust-analyzer)'s libraries.

## Supported languages

| Language | Tool used     | Notes |
| -------- | ------------- | ----- |
| Rust     | rust-analyzer |       |

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

## Usage

```
Usage: rust_ast_gen --input-dir <INPUT_DIR> --output-dir <OUTPUT_DIR>

Options:
  -i, --input-dir <INPUT_DIR>       Input directory containing a Rust project
  -o, --output-dir <OUTPUT_DIR>     Output directory where generated files will be written to
      --no-sysroot                  Skip sysroot loading (faster, but will not resolve std symbols)
      --resolve-cfg                 Resolve #[cfg(...)] attributes, dropping inactive items
      --target <TRIPLE>             rustc target triple override (e.g. x86_64-pc-windows-msvc)
      --features <FEAT,...>         Cargo features to enable on the workspace crate
      --no-default-features           Do not enable default features on the workspace crate
  -h, --help                        Print help
```

One `.json` file is produced per `.rs` source file, mirroring the input directory structure.

By default, the host target and the workspace crate's default features are used. Use `--target` to analyze platform-specific stdlib code (e.g. Windows-only modules), and `--features` to activate feature-gated items in the workspace crate.

Set `RUST_LOG=info` (or `debug`/`trace`) for progress output.

## Example

```bash
./target/release/rust_ast_gen --input-dir <path to project> --output-dir <path to output>
```

Cross-target analysis with an extra feature enabled:

```bash
./target/release/rust_ast_gen \
  --input-dir <path to project> \
  --output-dir <path to output> \
  --target x86_64-pc-windows-msvc \
  --features my_feature
```
