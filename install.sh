#!/bin/bash
# mcuforge installer for Linux/macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/solitasroh/mcuforge/main/install.sh | bash
set -e

REPO="solitasroh/mcuforge"
INSTALL_DIR="$HOME/.embtool/bin"

echo "mcuforge installer"
echo ""

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
case "$OS" in
    linux)  ASSET_PATTERN="linux" ;;
    darwin) ASSET_PATTERN="macos" ;;
    *)      echo "Unsupported OS: $OS"; exit 1 ;;
esac

# Get latest release
echo "  Fetching latest release..."
RELEASE=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest")
TAG=$(echo "$RELEASE" | grep -o '"tag_name":"[^"]*"' | head -1 | cut -d'"' -f4)
VERSION=${TAG#v}

echo "  Latest version: $TAG"

# Find binary asset URL
ASSET_URL=$(echo "$RELEASE" | grep -o "\"browser_download_url\":\"[^\"]*${ASSET_PATTERN}[^\"]*\"" | head -1 | cut -d'"' -f4)
if [ -z "$ASSET_URL" ]; then
    echo "  No $ASSET_PATTERN binary found in release $TAG"
    exit 1
fi

# Create install directory
mkdir -p "$INSTALL_DIR"
DEST="$INSTALL_DIR/mcuforge"

# Download
echo "  Downloading..."
curl -fsSL "$ASSET_URL" -o "$DEST"
chmod +x "$DEST"

echo "  Installed to: $DEST"

# Check PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "  Add to PATH by appending to your shell profile:"
    SHELL_NAME=$(basename "$SHELL")
    case "$SHELL_NAME" in
        zsh)  RC="$HOME/.zshrc" ;;
        bash) RC="$HOME/.bashrc" ;;
        *)    RC="$HOME/.profile" ;;
    esac
    echo "    echo 'export PATH=\"\$HOME/.embtool/bin:\$PATH\"' >> $RC"
fi

echo ""
echo "  mcuforge $VERSION installed successfully!"
echo ""
echo "  Quick start:"
echo "    mcuforge --version"
echo "    mcuforge new my-project --mcu k64 --claude"
echo "    mcuforge claude install"
