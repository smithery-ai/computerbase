#!/usr/bin/env bash
# Build release binaries, tag the commit, and publish a GitHub release.
#
# Usage:
#   scripts/release.sh            # use version from Cargo.toml
#   scripts/release.sh 0.2.0      # override version (also updates Cargo.toml)
#   scripts/release.sh --dry-run  # build + tarball, skip tag/push/release

set -euo pipefail

BIN_NAME="computer-use-mcp"
REPO_SLUG="smithery-ai/computerbase"
# Prefer rustup's cargo over Homebrew's (Homebrew rustc lags behind).
export PATH="$HOME/.cargo/bin:$PATH"

DRY_RUN=0
VERSION_OVERRIDE=""
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    -h|--help)
      sed -n '2,7p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *) VERSION_OVERRIDE="$arg" ;;
  esac
done

cd "$(dirname "$0")/.."

if [[ -n "$VERSION_OVERRIDE" ]]; then
  VERSION="$VERSION_OVERRIDE"
  # Update Cargo.toml version in place.
  sed -i '' -E "0,/^version = \".*\"/s//version = \"$VERSION\"/" Cargo.toml
  cargo update -p "$BIN_NAME" >/dev/null 2>&1 || true
  if ! git diff --quiet Cargo.toml Cargo.lock; then
    git add Cargo.toml Cargo.lock
    git commit -m "chore: release v$VERSION"
  fi
else
  VERSION=$(grep -E '^version = ' Cargo.toml | head -1 | sed -E 's/version = "(.*)"/\1/')
fi

TAG="v$VERSION"
echo "==> Releasing $TAG"

# Sanity: clean working tree.
if [[ -n "$(git status --porcelain)" ]]; then
  echo "error: working tree is dirty. Commit or stash first." >&2
  git status --short
  exit 1
fi

# Tag must not exist yet.
if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "error: tag $TAG already exists." >&2
  exit 1
fi

# Build per-target tarballs.
HOST_ARCH=$(uname -m)
case "$HOST_ARCH" in
  arm64|aarch64) HOST_TARGET="aarch64-apple-darwin" ;;
  x86_64)        HOST_TARGET="x86_64-apple-darwin" ;;
  *) echo "error: unsupported host arch $HOST_ARCH" >&2; exit 1 ;;
esac

TARGETS=("$HOST_TARGET")
# Add the other macOS target if its stdlib is installed.
OTHER_TARGET="x86_64-apple-darwin"
[[ "$HOST_TARGET" == "x86_64-apple-darwin" ]] && OTHER_TARGET="aarch64-apple-darwin"
if rustup target list --installed 2>/dev/null | grep -q "^$OTHER_TARGET$"; then
  TARGETS+=("$OTHER_TARGET")
fi

DIST="dist/$TAG"
rm -rf "$DIST"
mkdir -p "$DIST"

ARTIFACTS=()
for target in "${TARGETS[@]}"; do
  echo "==> Building $target"
  cargo build --release --target "$target"
  BIN_PATH="target/$target/release/$BIN_NAME"
  strip "$BIN_PATH" || true
  TARBALL="$DIST/$BIN_NAME-$TAG-$target.tar.gz"
  tar -czf "$TARBALL" -C "target/$target/release" "$BIN_NAME"
  ARTIFACTS+=("$TARBALL")
  echo "    -> $TARBALL"
done

# Checksums.
(cd "$DIST" && shasum -a 256 *.tar.gz > SHA256SUMS)
ARTIFACTS+=("$DIST/SHA256SUMS")
echo "==> Checksums:"
cat "$DIST/SHA256SUMS"

if [[ "$DRY_RUN" == "1" ]]; then
  echo "==> Dry run: skipping tag, push, and gh release."
  exit 0
fi

echo "==> Tagging $TAG"
git tag -a "$TAG" -m "release $TAG"
git push origin HEAD
git push origin "$TAG"

echo "==> Creating GitHub release"
gh release create "$TAG" \
  --repo "$REPO_SLUG" \
  --title "$TAG" \
  --generate-notes \
  "${ARTIFACTS[@]}"

echo "==> Done: https://github.com/$REPO_SLUG/releases/tag/$TAG"
