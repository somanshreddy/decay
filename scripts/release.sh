#!/usr/bin/env bash
# Cut a release: bump the version, commit, tag, push.
#
# Installed copies of decay poll this repo's tags, so pushing the tag is what
# actually ships the release.
#
#   ./scripts/release.sh 0.2.0
#   ./scripts/release.sh 0.2.0 --dry-run

set -euo pipefail

cd "$(dirname "$0")/.."

VERSION="${1:-}"
DRY_RUN="${2:-}"

if [[ -z "$VERSION" ]]; then
    echo "usage: $0 <version> [--dry-run]   e.g. $0 0.2.0" >&2
    exit 1
fi

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
    echo "error: '$VERSION' is not semver (expected X.Y.Z, no leading 'v')" >&2
    exit 1
fi

TAG="v$VERSION"
CURRENT=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)

if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
    echo "error: tag $TAG already exists" >&2
    exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
    echo "error: working tree is dirty — commit or stash first" >&2
    exit 1
fi

BRANCH=$(git rev-parse --abbrev-ref HEAD)

# Asking for the version already in Cargo.toml just tags what's there — that's
# how the first release gets cut.
if [[ "$VERSION" == "$CURRENT" ]]; then
    BUMP=false
    echo "  Tagging decay $VERSION as-is (tag $TAG, branch $BRANCH)"
else
    BUMP=true
    echo "  Releasing decay $CURRENT -> $VERSION (tag $TAG, branch $BRANCH)"
fi
echo

if [[ "$BUMP" == true ]]; then
    # Bump the manifest, then let cargo refresh the lockfile's version entry.
    perl -0pi -e "s/^version = \"$CURRENT\"/version = \"$VERSION\"/m" Cargo.toml
    cargo check --quiet
fi

echo "  Running tests…"
cargo test --quiet

if [[ "$DRY_RUN" == "--dry-run" ]]; then
    echo
    echo "  Dry run — would have tagged $TAG and pushed."
    [[ "$BUMP" == true ]] && git checkout -- Cargo.toml Cargo.lock
    exit 0
fi

if [[ "$BUMP" == true ]]; then
    git add Cargo.toml Cargo.lock
    git commit -m "release: $TAG"
fi
git tag -a "$TAG" -m "decay $VERSION"
git push origin "$BRANCH"
git push origin "$TAG"

echo
echo "  ✅ Released $TAG. Installed copies will pick it up on their next run."
