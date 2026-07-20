# CodeGeneration (vendored from swift-syntax)

The Swift sources under `SyntaxSupport/` are **vendored verbatim** from the
[`swiftlang/swift-syntax`](https://github.com/swiftlang/swift-syntax) repository's
`CodeGeneration/Sources/SyntaxSupport/` package. They define the metadata model used by the
upstream code generators (node hierarchy, traits, tokens, keywords, grammar, etc.).

We depend on this metadata at build time so that
[`ScalaAstGenerator`](../SwiftAstGenLib/ScalaAstGenerator.swift) can introspect the SwiftSyntax
node hierarchy and emit a strongly typed Scala wrapper (`SwiftNodeSyntax.scala`) for the AST
JSON consumed by the Joern Swift frontend.

## Provenance

| | |
| --- | --- |
| Upstream repository | <https://github.com/swiftlang/swift-syntax> |
| Upstream pin | matches `swift-syntax` in [`../../Package.swift`](../../Package.swift) (currently `from: "603.0.1"`) |
| Upstream path | `CodeGeneration/Sources/SyntaxSupport/` |
| License | Apache 2.0 with Runtime Library Exception (header preserved on every file) |

## Local modifications

None. Files are intended to be byte-identical with upstream so they can be refreshed by
copying. If you need to diverge, document the change inline with a comment block beginning
`// LOCAL CHANGE:` so future updates can re-apply it.

## Refreshing

When bumping the `swift-syntax` dependency in `Package.swift`:

1. Run `swift package resolve` to update `Package.resolved` to the new version.
2. Run the refresh script from the `swift-astgen/` directory:
   ```bash
   scripts/refresh-codegen.sh
   ```
   The script reads the pinned version from `Package.resolved`, clones that tag from
   `swiftlang/swift-syntax`, and syncs `CodeGeneration/Sources/SyntaxSupport/` into
   `Sources/CodeGeneration/SyntaxSupport/`. Requires `git` and `jq` on `PATH`.
3. Run `swift build && swift test`.
4. Regenerate `SwiftNodeSyntax.scala` (`swift run SwiftAstGen --scala-ast-only`) and verify
   `Tests/ScalaSwiftNodeSyntaxTests/` still passes against the new wrapper.
