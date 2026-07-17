# Perl AST Generator

Generates JSON Abstract Syntax Trees (ASTs) for Perl source files using
[tree-sitter-perl](https://github.com/tree-sitter-grammars/tree-sitter-perl). Built primarily
as the AST provider for the Perl frontend in [Joern](https://joern.io).

## Supported languages

| Language | Tool used          | Notes                                                     |
| -------- | ------------------ | --------------------------------------------------------- |
| Perl     | tree-sitter-perl   | Syntactic only — no name resolution, no type information. |

The output JSON mirrors the tree-sitter parse tree. Only named nodes are emitted. Each node
records its `node_type`, byte offsets (`start_byte`, `end_byte`), source position
(`start_position`, `end_position`), the raw source `text`, and a `children` array of named
child nodes.

## Project layout

| Path | Purpose |
| --- | --- |
| `src/main.rs` | CLI entry point, argument parsing, parallel dispatch. |
| `src/collect.rs` | File discovery with extension filtering and exclusion logic. |
| `src/parse.rs` | tree-sitter parsing and JSON serialization. |
| `src/ast.rs` | AST node types and tree-sitter → JSON mapping. |
| `src/scala_gen.rs` | Scala `PerlNodeSyntax` code generation artifact. |

## Building

```bash
cargo build              # debug build
cargo build --release    # release build
```

## Testing

```bash
cargo test
```

## Usage

```
USAGE: perl-astgen [OPTIONS]

OPTIONS:
  -i, --src <src>                   Source directory or file (default: `.`).
  -o, --output <output>             Output directory for generated AST json files (default: `./ast_out`).
  -p, --pretty-print                Pretty-print the generated AST JSON files.
  -s, --scala-ast-only              Only generate the Scala PerlNodeSyntax AST types file
                                    (writes `./PerlNodeSyntax.scala`).
      --exclude-file <PATH>         Exclude a specific file by absolute path. Can be repeated.
      --exclude-regex <PATTERN>     Exclude files whose absolute path matches this regex.
  -V, --version                     Print version.
  -h, --help                        Print help.
```

### `--scala-ast-only`

Writes a single Scala source file (`./PerlNodeSyntax.scala`) describing the Perl node type
hierarchy as a sealed trait family. Downstream Scala consumers (notably the Joern Perl
frontend) use this wrapper to navigate the JSON ASTs produced in normal mode.

### File exclusion

Pass `--exclude-file` (repeatable) to skip exact paths, and `--exclude-regex` to skip files
whose absolute path matches a pattern:

```bash
perl-astgen --exclude-file /absolute/path/to/generated.pl
perl-astgen --exclude-regex '/vendor/'
perl-astgen --exclude-regex '\.t$'
```

An unresolvable `--exclude-file` path and an invalid `--exclude-regex` pattern both produce a
warning and are ignored — the run continues.

## Examples

From a release binary on `PATH`:

```bash
cd <path to project>
perl-astgen
perl-astgen -i <path to project>
perl-astgen -i <path to project> -o <path to output directory>
perl-astgen -i /path/to/single/script.pl -o /tmp/ast_out
perl-astgen --scala-ast-only
```

From a checkout (debug build, no install needed):

```bash
cargo run -- -i <path to project> -o <path to output directory>
cargo run -- --scala-ast-only
```
