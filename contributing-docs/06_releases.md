Releases
========

Every tool is versioned and released independently. A release is triggered by pushing a tag, and it publishes
binaries (and, where it applies, the generated Scala wrapper) to the
[Releases page](https://github.com/joernio/astgen-monorepo/releases). Joern then bumps the version it uses.

Contributors normally do not release. A maintainer does, after a PR is merged. This page is for maintainers and for
anyone who wants to understand what happens.

Tags
----

Tags are prefixed with the tool directory: `{tool}-astgen/v{version}`, for example `javascript-astgen/v3.53.0` or
`rust-astgen/v0.27.7`. The release workflow of a tool only reacts to its own prefix, for example
`javascript-astgen/v*`.

Where the version lives
-----------------------

The version is stored in a different place for each tool. Update it in the same commit that you tag.

| Tool | Source of truth | How to bump |
|------|-----------------|-------------|
| `javascript-astgen` | `package.json`, mirrored to `src/version.ts` (generated) | `./bump-version.sh X.Y.Z`, then `bun install` to refresh `bun.lock` |
| `abap-astgen` | `package.json` | edit by hand |
| `swift-astgen` | `VERSION`, mirrored to `Sources/SwiftAstGenLib/Version.swift` (generated) | `./bump-version.sh X.Y.Z` |
| `rust-astgen` | `version` in `[workspace.package]` of `Cargo.toml` | edit by hand and let cargo update `Cargo.lock`. The release fails if the tag differs |
| `ruby-astgen` | `lib/ruby_ast_gen/version.rb` | `./update_version.rb vX.Y.Z`. The release fails if the tag differs |
| `go-astgen` | the tag itself, passed with `-ldflags "-X main.Version=..."` | nothing to edit |
| `dotnet-astgen` | the tag itself, passed to `build-with-version.sh` | nothing to edit |

Never edit a generated version file (`src/version.ts`, `Version.swift`) by hand. Use the bump script.

Creating a release
------------------

1. Make sure the change you want to release is merged to `main` and CI is green.
2. Bump the version as in the table above, and commit it:
   `[javascript-astgen] Bump version to 3.54.0`.
3. Tag and push:
   ```bash
   git tag javascript-astgen/v3.54.0
   git push origin javascript-astgen/v3.54.0
   ```
4. The release workflow builds the binaries for all platforms, runs the tests, and creates a **draft** release.
   Review the assets and checksums, then publish the draft by hand.

To rebuild an existing release, run the release workflow from the Actions tab (`workflow_dispatch`). It discovers
the latest tag of that tool and updates its release with fresh binaries.

Verifying a release
-------------------

Releases carry `checksums/SHA256SUMS` and `checksums/SHA512SUMS`. See the
[README](../README.md#verifying-releases) for the commands.

Rules for agents
----------------

Do not create or push tags, publish releases, or run release workflows unless the user explicitly asks for it.
Bumping a version in a PR is fine if the user asks for it, and tagging stays a separate, deliberate step.
