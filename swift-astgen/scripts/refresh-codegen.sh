#!/usr/bin/env bash
# Refresh Sources/CodeGeneration/SyntaxSupport/ from the upstream swift-syntax tag
# pinned in Package.resolved.
#
# Run from the swift-astgen/ directory after bumping swift-syntax in Package.swift
# and running `swift package resolve` to update Package.resolved.
#
# After running this script:
#   swift build && swift test
#   ./build/release/SwiftAstGen --scala-ast-only   # regenerate SwiftNodeSyntax.scala

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DEST="$REPO_ROOT/Sources/CodeGeneration/SyntaxSupport"
RESOLVED="$REPO_ROOT/Package.resolved"

if ! command -v jq &>/dev/null; then
  echo "error: jq is required but not found on PATH" >&2
  exit 1
fi

VERSION=$(jq -r '.pins[] | select(.identity == "swift-syntax") | .state.version' "$RESOLVED")
if [[ -z "$VERSION" || "$VERSION" == "null" ]]; then
  echo "error: could not read swift-syntax version from $RESOLVED" >&2
  exit 1
fi

echo "swift-syntax pinned version: $VERSION"

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo "Cloning swift-syntax at tag $VERSION ..."
git clone --quiet --depth 1 --branch "$VERSION" \
  https://github.com/swiftlang/swift-syntax "$TMPDIR/swift-syntax" >/dev/null 2>&1

SRC="$TMPDIR/swift-syntax/CodeGeneration/Sources/SyntaxSupport"
if [[ ! -d "$SRC" ]]; then
  echo "error: expected SyntaxSupport directory not found at $SRC" >&2
  exit 1
fi

echo "Syncing to $DEST ..."
rsync -a --delete \
  --exclude="BuilderInitializableTypes.swift" \
  "$SRC/" "$DEST/"

echo "Done. Next steps:"
echo "  1. swift build && swift test"
echo "  2. swift run SwiftAstGen --scala-ast-only   # regenerate SwiftNodeSyntax.scala"
echo "  3. Review and commit the changes"
