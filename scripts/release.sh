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

# Ensure we are on a clean main branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$CURRENT_BRANCH" != "main" ] && [ "$TAG_MODE" = false ]; then
    echo "Warning: You are not on the main branch. It is recommended to release from main."
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

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
    git commit -m "chore: release v$VERSION"
    git tag -a "v$VERSION" -m "Release v$VERSION"
    echo "Pushing tag to origin..."
    git push origin "v$VERSION"
    echo "Release v$VERSION tagged and pushed!"
else
    BRANCH_NAME="$VERSION"
    echo "Creating branch $BRANCH_NAME..."
    git checkout -b "$BRANCH_NAME"
    git add Cargo.toml registry/Cargo.toml loft_builtin_macros/Cargo.toml Cargo.lock
    git commit -m "chore: bump version to $VERSION"
    echo "Pushing branch to origin..."
    git push origin "$BRANCH_NAME"
    
    if command -v gh &> /dev/null; then
        echo "Creating Pull Request..."
        gh pr create --title "v$VERSION" --body "Release candidate $VERSION" --base main --head "$BRANCH_NAME"
    else
        echo "GitHub CLI (gh) not found. Please create the PR manually."
    fi
    echo "Release branch $BRANCH_NAME pushed and PR created!"
fi
