#!/usr/bin/env bash
# Bump the crate version for the next release. Single-crate counterpart of
# aura's scripts/prepare-release.sh: fetch the latest vX.Y.Z tag, set
# Cargo.toml to the next version, refresh Cargo.lock. Committing, tagging
# and pushing the tag (which triggers release.yml) stay manual.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

BUMP_KIND="${BUMP_KIND:-patch}"
REMOTE="${REMOTE:-origin}"
TAG_PATTERN='v[0-9]*.[0-9]*.[0-9]*'
MANIFEST=Cargo.toml

usage() {
    cat <<'EOF'
Usage: scripts/prepare-release.sh [options]

Fetches the latest remote release tag and sets the crate version in
Cargo.toml to the next release version.

Options:
  --bump <kind>      patch|minor|major. Default: patch
  --remote <name>    Git remote to fetch tags from. Default: origin
  --no-fetch         Skip 'git fetch --tags'
  --dry-run          Show the planned version without editing files
  -h, --help         Show this help

Environment overrides:
  BUMP_KIND
  REMOTE

Notes:
  - Release tags are expected to look like vX.Y.Z.
  - With no tags yet, the current Cargo.toml version is the release.
  - Commit creation, tagging, and tag push remain manual.
EOF
}

die() {
    echo "error: $*" >&2
    exit 1
}

require_tool() {
    command -v "$1" >/dev/null 2>&1 || die "required tool not found: $1"
}

bump_version() {
    local major minor patch
    IFS=. read -r major minor patch <<<"$1"
    [[ "$major" =~ ^[0-9]+$ && "$minor" =~ ^[0-9]+$ && "$patch" =~ ^[0-9]+$ ]] \
        || die "invalid semver version: $1"
    case "$2" in
        patch) patch=$((patch + 1)) ;;
        minor) minor=$((minor + 1)); patch=0 ;;
        major) major=$((major + 1)); minor=0; patch=0 ;;
        *) die "unsupported bump kind: $2" ;;
    esac
    printf '%s.%s.%s\n' "$major" "$minor" "$patch"
}

# Prints -1, 0 or 1 for left <, =, > right.
compare_versions() {
    local l r i
    IFS=. read -r -a l <<<"$1"
    IFS=. read -r -a r <<<"$2"
    for i in 0 1 2; do
        if (( l[i] < r[i] )); then echo -1; return; fi
        if (( l[i] > r[i] )); then echo 1; return; fi
    done
    echo 0
}

manifest_version() {
    sed -nE 's/^version[[:space:]]*=[[:space:]]*"(.*)"[[:space:]]*$/\1/p' "$MANIFEST" | head -n1
}

crate_name() {
    sed -nE 's/^name[[:space:]]*=[[:space:]]*"(.*)"[[:space:]]*$/\1/p' "$MANIFEST" | head -n1
}

FETCH_TAGS=true
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --bump)
            [[ $# -ge 2 ]] || die "--bump requires a value"
            BUMP_KIND="$2"
            shift 2
            ;;
        --remote)
            [[ $# -ge 2 ]] || die "--remote requires a value"
            REMOTE="$2"
            shift 2
            ;;
        --no-fetch)
            FETCH_TAGS=false
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            die "unknown option: $1"
            ;;
    esac
done

case "$BUMP_KIND" in
    patch|minor|major) ;;
    *) die "invalid bump kind: $BUMP_KIND" ;;
esac

require_tool git
require_tool perl
require_tool cargo

if $FETCH_TAGS; then
    git fetch --tags "$REMOTE"
fi

CURRENT_VERSION="$(manifest_version)"
[[ -n "$CURRENT_VERSION" ]] || die "no version found in $MANIFEST"

LATEST_TAG="$(git tag --list "$TAG_PATTERN" --sort=-version:refname | head -n1)"
if [[ -n "$LATEST_TAG" ]]; then
    TARGET_VERSION="$(bump_version "${LATEST_TAG#v}" "$BUMP_KIND")"
    echo "Latest release tag: $LATEST_TAG"
else
    TARGET_VERSION="$CURRENT_VERSION"
    echo "No release tags yet; releasing the current version."
fi

if [[ "$(compare_versions "$CURRENT_VERSION" "$TARGET_VERSION")" == "1" ]]; then
    die "$MANIFEST already has $CURRENT_VERSION, newer than the target $TARGET_VERSION"
fi

echo "$(crate_name): $CURRENT_VERSION -> $TARGET_VERSION"

if $DRY_RUN; then
    echo "Dry run only. No files were modified."
    exit 0
fi

if [[ "$CURRENT_VERSION" != "$TARGET_VERSION" ]]; then
    # First `version =` only: the [package] one, not a dependency's.
    # /e keeps `$1` from reading as `\1` followed by a version digit.
    perl -0pi -e 'my $v="'"$TARGET_VERSION"'"; s/^(version\s*=\s*")[^"]*(")/my($p,$s)=($1,$2);"$p$v$s"/me' "$MANIFEST"
fi

# Refresh only this crate's entry in Cargo.lock; dependencies stay pinned.
cargo update --workspace

echo ""
echo "Next manual steps:"
echo "  1. Review the Cargo.toml and Cargo.lock changes."
echo "  2. Commit: git commit -am 'chore(release): \`v$TARGET_VERSION\`'"
echo "  3. Tag and push: git tag v$TARGET_VERSION && git push $REMOTE v$TARGET_VERSION"
echo "     (the tag triggers .github/workflows/release.yml)"
