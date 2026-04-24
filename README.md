# Joern AST Generators

A monorepo containing language-specific AST generation tools for the [Joern](https://joern.io) code analysis platform. Each tool is self-contained with its own build system, tests, dependencies, and CI/CD workflows, and is independently versioned and released.

## Available Tools

| Language        | Directory                                  | Parser / Backend    | Status   |
| --------------- | ------------------------------------------ | ------------------- | -------- |
| ABAP            | [`abap-astgen/`](./abap-astgen/)           | —                   | Deferred |
| C# / .NET       | [`dotnet-astgen/`](./dotnet-astgen/)       | Roslyn              | Active   |
| Go              | [`go-astgen/`](./go-astgen/)               | Go standard library | Active   |
| JavaScript / TS | [`javascript-astgen/`](./javascript-astgen/) | Babel + tsc       | Active   |
| Ruby            | [`ruby-astgen/`](./ruby-astgen/)           | `parser` gem        | Active   |
| Rust            | [`rust-astgen/`](./rust-astgen/)           | rust-analyzer       | Active   |
| Swift           | [`swift-astgen/`](./swift-astgen/)         | SwiftSyntax         | Active   |

## Testing framework

[`astgen-regression/`](./astgen-regression/) is a config-driven regression testing framework shared by the tools above. It clones real-world codebases at specific versions, builds the generator from base and PR branches using git worktrees, compares outputs, and produces Markdown reports. See the [framework README](./astgen-regression/README.md) for setup and usage.

## Repository Structure

Each language tool is self-contained with its own:

- build system and dependencies
- tests and documentation
- CI/CD workflows
- independent versioning and releases

## Releases

Each tool uses prefixed tags of the form `{lang}-astgen/v{version}`, for example:

- `javascript-astgen/v3.43.0`
- `rust-astgen/v0.3.0`

See the [Releases page](https://github.com/joernio/astgen-monorepo/releases) for downloadable binaries.

### Creating a release

**Via git tag (recommended)**

1. Update the version in the language-specific files (e.g. `package.json`, `Cargo.toml`).
2. Commit the version change.
3. Create and push a prefixed tag:
   ```bash
   git tag javascript-astgen/v3.43.0
   git push origin javascript-astgen/v3.43.0
   ```
4. The release workflow will automatically build and publish the binaries.

**Manually via GitHub Actions**

To re-release or rebuild an existing tag:

1. Go to the [Actions tab](https://github.com/joernio/astgen-monorepo/actions).
2. Select the language-specific release workflow (e.g. "JavaScript AST Generator Release").
3. Click **Run workflow**, select the branch (usually `main`), and confirm.

The workflow discovers the latest tag for that language and creates or updates the release with fresh binaries.

## Contributing

Contributions are welcome. Please open an issue or pull request on [GitHub](https://github.com/joernio/astgen-monorepo). See each tool's README for language-specific build and test instructions.

## License

Licensed under the [Apache License, Version 2.0](./LICENSE).
