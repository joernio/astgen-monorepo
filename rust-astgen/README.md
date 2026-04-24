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
  -i, --input-dir <INPUT_DIR>    Input directory containing a Rust project
  -o, --output-dir <OUTPUT_DIR>  Output directory where generated files will be written to
  -h, --help                     Print help
```

One `.json` file is produced per `.rs` source file, mirroring the input directory structure.

Set `RUST_LOG=info` (or `debug`/`trace`) for progress output.

## Example

```bash
./target/release/rust_ast_gen --input-dir <path to project> --output-dir <path to output>
```
