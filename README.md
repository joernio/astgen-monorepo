# Joern AST Generators

A monorepo containing language-specific AST generation tools for the Joern code analysis platform.

## Available Tools

Each tool is independently versioned and released:

- **[JavaScript AST Generator](./javascript-astgen/)** - Generate ASTs for JavaScript/TypeScript code
- **[Rust AST Generator](./rust-astgen/)** - Generate ASTs for Rust code  
- **[Swift AST Generator](./swift-astgen/)** - Generate ASTs for Swift code
- **[.NET AST Generator](./dotnet-astgen/)** - Generate ASTs for C#/.NET code
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

See the [Releases page](https://github.com/joernio/astgen-monorepo/releases) for downloadable binaries.

### Creating a Release

**Via Git Tag (recommended):**

1. Update version in the language-specific files (e.g., `package.json`, `Cargo.toml`, etc.)
2. Commit the version change
3. Create and push a prefixed tag:
   ```bash
   git tag javascript-astgen/v3.43.0
   git push origin javascript-astgen/v3.43.0
   ```
4. The release workflow will automatically build and publish binaries

**Manual Release (GitHub Actions):**

If you need to re-release or rebuild an existing tag:

1. Go to the [Actions tab](https://github.com/joernio/astgen-monorepo/actions)
2. Select the language-specific release workflow (e.g., "JavaScript AST Generator Release")
3. Click "Run workflow"
4. Select the branch (usually `main`)
5. Click "Run workflow"

The workflow will automatically discover the latest tag for that language and create/update the release with fresh binaries.

## Contributing

Each tool has its own contribution guidelines. See the README in each directory.

## License

See individual tool directories for license information.
