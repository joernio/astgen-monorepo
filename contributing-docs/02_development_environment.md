Development environment
=======================

You only need the toolchain of the tool you work on. Install the others when you touch them.

Toolchains
----------

The CI workflows in [.github/workflows](../.github/workflows) are the source of truth for versions. The table lists
what they use at the time of writing.

| Tool | You need | CI uses |
|------|----------|---------|
| `abap-astgen` | [Bun](https://bun.sh) | Bun 1.4.3 |
| `javascript-astgen` | Bun, plus JDK and sbt for the [Scala tests](04_tests.md) | Bun 1.4.3, Temurin JDK 21 |
| `dotnet-astgen` | .NET SDK | 8.x |
| `go-astgen` | Go | 1.18 for CI (matches `go.mod`), 1.21.12 for release builds |
| `ruby-astgen` | Ruby or JRuby, Bundler | Ruby 3.1.4, and JRuby 9.4.15.0 with JDK 21 |
| `rust-astgen` | Rust (stable) with the `rust-src` component, plus JDK and sbt for the Scala tests | stable, Temurin JDK 21 |
| `swift-astgen` | Swift toolchain, `swift-format`, plus JDK and sbt for the Scala tests | Swift 6.1, and 6.3 on macOS to match the Xcode SDK |
| `astgen-regression` | Python 3.12+ | Python 3.12 |
| CI path detection (`.github/scripts`) | Node.js | Node 24 |

### Per tool setup

```bash
# abap-astgen, javascript-astgen
bun install

# dotnet-astgen
dotnet restore && dotnet build

# go-astgen
go build ./...

# ruby-astgen (the release uses JRuby, see the tool's README)
jruby -S bundle install

# rust-astgen
rustup component add rust-src
cargo build --release

# swift-astgen
swift build

# astgen-regression
python -m venv .venv && source .venv/bin/activate
pip install -r requirements-dev.txt && pip install -e .
```

Platform notes
--------------

- `rust-astgen` loads the Rust sysroot to resolve `std` symbols, which is why `rust-src` is required.
  `--no-sysroot` skips it, but then `std` stays unresolved.
- `swift-astgen` builds on all three platforms. On Linux, CI cross-builds with the Swift static Linux SDK (musl) to
  get portable binaries. A plain `swift build` is enough for development.
- `ruby-astgen` runs on both MRI and JRuby, and CI tests both. If you switch between them, delete `vendor/` before
  running `bundle install`, otherwise native extensions clash.
- The binaries of `abap-astgen` and `javascript-astgen` are built with `bun run binary`, which cross-compiles for six
  targets in one go. You do not need it for development, `bun src/astgen.ts` runs the sources directly.
- CI ad-hoc signs the macOS binaries produced by Bun with `ldid`. You do not need that locally.

Running an astgen from source
-----------------------------

```bash
bun javascript-astgen/src/astgen.ts -i <project> -o <out>        # JavaScript / TypeScript / Vue
bun abap-astgen/src/parse-abap.ts <input-dir> <output-dir>       # ABAP
cd go-astgen && go run . -out <out> <project>                    # Go
cd dotnet-astgen && dotnet run --project DotNetAstGen -- -i <project> -o <out>
cd ruby-astgen && jruby -S bundle exec exe/ruby_ast_gen -i <project> -o <out>
cd rust-astgen && cargo run --release -- --input-dir <project> --output-dir <out>
cd swift-astgen && swift run SwiftAstGen -i <project> -o <out>
```

See each tool's README for all options.

Working on several tools
------------------------

Tools do not depend on each other, so you can work on them in parallel in separate directories. For a change that
spans tools (CI config, docs, `astgen-regression`) open the repository root.
