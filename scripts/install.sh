#!/usr/bin/env bash
# Install the latest computer-use-mcp release.
#
#   curl -fsSL https://raw.githubusercontent.com/smithery-ai/computerbase/master/scripts/install.sh | bash
#
# Env vars:
#   VERSION    pin a specific tag (default: latest)
#   INSTALL_DIR  target dir (default: $HOME/.local/bin)

set -euo pipefail

REPO_SLUG="smithery-ai/computerbase"
BIN_NAME="computer-use-mcp"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

OS=$(uname -s)
if [[ "$OS" != "Darwin" ]]; then
  echo "error: only macOS is supported (got $OS)." >&2
  exit 1
fi

case "$(uname -m)" in
  arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
  x86_64)        TARGET="x86_64-apple-darwin" ;;
  *) echo "error: unsupported arch $(uname -m)" >&2; exit 1 ;;
esac

VERSION="${VERSION:-}"
if [[ -z "$VERSION" ]]; then
  VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO_SLUG/releases/latest" \
    | awk -F'"' '/"tag_name"/ {print $4; exit}')
  if [[ -z "$VERSION" ]]; then
    echo "error: could not determine latest release. Set VERSION=vX.Y.Z to pin." >&2
    exit 1
  fi
fi

TARBALL="$BIN_NAME-$VERSION-$TARGET.tar.gz"
URL="https://github.com/$REPO_SLUG/releases/download/$VERSION/$TARBALL"

echo "==> Downloading $URL"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

curl -fsSL "$URL" -o "$TMP/$TARBALL"
tar -xzf "$TMP/$TARBALL" -C "$TMP"

mkdir -p "$INSTALL_DIR"
mv "$TMP/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
chmod +x "$INSTALL_DIR/$BIN_NAME"

echo "==> Installed $INSTALL_DIR/$BIN_NAME ($VERSION)"

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo ""
    echo "Add $INSTALL_DIR to PATH, e.g.:"
    echo "  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.zshrc && exec zsh"
    ;;
esac

echo ""
echo "Run: $BIN_NAME"
echo "Grant Accessibility + Screen Recording permissions when prompted."
