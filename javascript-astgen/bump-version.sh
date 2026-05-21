#!/usr/bin/env bash
set -euo pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get current version
CURRENT_VERSION=$(grep '"version"' package.json | head -1 | sed -E 's/.*"version": "(.+)".*/\1/')

echo -e "${GREEN}Current version: ${CURRENT_VERSION}${NC}"
echo ""

# Get new version from argument or prompt
if [ $# -eq 1 ]; then
    NEW_VERSION="$1"
else
    echo "Usage: $0 <new-version>"
    echo ""
    echo "Example: $0 3.44.0"
    exit 1
fi

# Validate semver format (basic check)
if ! echo "$NEW_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo -e "${RED}Error: Version must be in format X.Y.Z (e.g., 3.44.0)${NC}"
    exit 1
fi

echo -e "${YELLOW}Bumping version: ${CURRENT_VERSION} → ${NEW_VERSION}${NC}"
echo ""

# Update package.json (the single source of truth)
sed -i.bak -E "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/" package.json
rm package.json.bak

# Regenerate src/version.ts from package.json
node scripts/sync-version.mjs

echo -e "${GREEN}✓ Updated package.json${NC}"
echo -e "${GREEN}✓ Regenerated src/version.ts${NC}"
echo ""

# Show diff
echo "Changes:"
git diff --no-ext-diff package.json src/version.ts

echo ""
echo -e "${GREEN}Version bumped successfully!${NC}"
echo ""
echo "Next steps:"
echo "  1. Review the changes above"
echo "  2. Run: yarn install  (to update yarn.lock)"
echo "  3. Run: git add package.json src/version.ts yarn.lock"
echo "  4. Run: git commit -m \"[javascript-astgen] Bump version to ${NEW_VERSION}\""
echo "  5. Run: git tag javascript-astgen/v${NEW_VERSION}"
echo "  6. Run: git push && git push --tags"
