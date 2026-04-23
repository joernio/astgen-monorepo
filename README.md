# Joern AST Generators

A monorepo containing language-specific AST generation tools for the Joern code analysis platform.

## Available Tools

Each tool is independently versioned and released:

- **[JavaScript AST Generator](./javascript-astgen/)** - Generate ASTs for JavaScript/TypeScript code
- **[Rust AST Generator](./rust-astgen/)** - Generate ASTs for Rust code  
- **[Swift AST Generator](./swift-astgen/)** - Generate ASTs for Swift code
- **[.NET AST Generator](./dotnet-astgen/)** - Generate ASTs for C#/F#/.NET code
- **[Go AST Generator](./go-astgen/)** - Generate ASTs for Go code
- **[Ruby AST Generator](./ruby-astgen/)** - Generate ASTs for Ruby code
- **[ABAP AST Generator](./abap-astgen/)** - Generate ASTs for ABAP code _(deferred)_

## Regression Testing

- **[AST Generator Regression Framework](./astgen-regression/)** - Testing framework for all AST generators

## Repository Structure

Each language tool is self-contained with its own:
- Build system and dependencies
- Tests and documentation
- CI/CD workflows
- Independent versioning and releases

## Releases

Each tool uses prefixed tags for releases:
- `javascript-astgen/v1.0.0`
- `rust-astgen/v2.0.0`
- etc.

See the [Releases page](../../releases) for downloadable binaries.

## Contributing

Each tool has its own contribution guidelines. See the README in each directory.

## License

See individual tool directories for license information.
