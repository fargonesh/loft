#!/bin/bash
set -e

# scripts/release.sh
# Usage: ./scripts/release.sh <version> [--tag]
# Example: ./scripts/release.sh 0.1.0-rc-3
#          ./scripts/release.sh 1.0.0 --tag

VERSION=$1
TAG_MODE=false

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version> [--tag]"
    echo "Example: $0 0.1.0-rc-3"
    exit 1
fi

if [ "$2" == "--tag" ]; then
    TAG_MODE=true
fi

# Fetch and checkout origin/main to be up to date
echo "Updating from origin/main..."
git fetch origin
git checkout main
git reset --hard origin/main

echo "Preparing release $VERSION..."

# Update versions in Cargo.toml files
echo "Updating Cargo.toml files..."
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" registry/Cargo.toml
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" loft_builtin_macros/Cargo.toml

# Update Cargo.lock
echo "Updating Cargo.lock..."
cargo check > /dev/null 2>&1

# Create branch or tag
if [ "$TAG_MODE" = true ]; then
    echo "Creating tag v$VERSION..."
    git add Cargo.toml registry/Cargo.toml loft_builtin_macros/Cargo.toml Cargo.lock
    git commit -m "release: v$VERSION"
    git tag -a "v$VERSION" -m "Release v$VERSION"
    echo "Pushing tag to origin..."
    git push origin "v$VERSION"
    echo "Release v$VERSION tagged and pushed!"
else
    BRANCH_NAME="release/v$VERSION"
    echo "Creating branch $BRANCH_NAME..."
    git checkout -b "$BRANCH_NAME"
    
    # Generate release notes from commits since last tag
    REPO="fargonesh/loft"
    LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
    if [ -n "$LAST_TAG" ]; then
        echo "Generating release notes since $LAST_TAG..."
        # Format: [@author](https://github.com/author): Title ([short_hash](https://github.com/owner/repo/commit/hash))
        RELEASE_NOTES=$(git log "$LAST_TAG..HEAD" --pretty=format:"[@%an](https://github.com/%an): %s ([%h](https://github.com/$REPO/commit/%H))")
    else
        echo "No previous tag found. Generating release notes from all commits..."
        RELEASE_NOTES=$(git log --pretty=format:"[@%an](https://github.com/%an): %s ([%h](https://github.com/$REPO/commit/%H))")
    fi

    git add Cargo.toml registry/Cargo.toml loft_builtin_macros/Cargo.toml Cargo.lock
    git commit -m "release: v$VERSION"
    echo "Pushing branch to origin..."
    git push origin "$BRANCH_NAME"
    
    if command -v gh &> /dev/null; then
        echo "Creating Pull Request..."
        PR_BODY="Release candidate $VERSION

### Changes:
$RELEASE_NOTES"
        gh pr create --title "release: v$VERSION" --body "$PR_BODY" --base main --head "$BRANCH_NAME"
    else
        echo "GitHub CLI (gh) not found. Please create the PR manually."
    fi
    echo "Release branch $BRANCH_NAME pushed and PR created!"
fi
