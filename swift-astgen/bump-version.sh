#!/usr/bin/env bash
set -euo pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get current version
CURRENT_VERSION=$(cat VERSION)

echo -e "${GREEN}Current version: ${CURRENT_VERSION}${NC}"
echo ""

# Get new version from argument or prompt
if [ $# -eq 1 ]; then
    NEW_VERSION="$1"
else
    echo "Usage: $0 <new-version>"
    echo ""
    echo "Example: $0 0.4.3"
    exit 1
fi

# Validate semver format (basic check)
if ! echo "$NEW_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo -e "${RED}Error: Version must be in format X.Y.Z (e.g., 0.4.3)${NC}"
    exit 1
fi

echo -e "${YELLOW}Bumping version: ${CURRENT_VERSION} → ${NEW_VERSION}${NC}"
echo ""

# Update VERSION file (the single source of truth)
echo -n "$NEW_VERSION" > VERSION

# Regenerate Sources/SwiftAstGenLib/Version.swift from VERSION
swift scripts/sync-version.swift

echo -e "${GREEN}✓ Updated VERSION${NC}"
echo -e "${GREEN}✓ Regenerated Sources/SwiftAstGenLib/Version.swift${NC}"
echo ""

# Show diff
echo "Changes:"
git diff --no-ext-diff VERSION Sources/SwiftAstGenLib/Version.swift

echo ""
echo -e "${GREEN}Version bumped successfully!${NC}"
echo ""
echo "Next steps:"
echo "  1. Review the changes above"
echo "  2. Run: git add VERSION Sources/SwiftAstGenLib/Version.swift"
echo "  3. Run: git commit -m \"[swift-astgen] Bump version to ${NEW_VERSION}\""
echo "  4. Run: git tag swift-astgen/v${NEW_VERSION}"
echo "  5. Run: git push && git push --tags"
