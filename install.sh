#!/bin/bash
set -e

REPO="cyhndaugust/bsr"
BIN_NAME="bsr"

# Detect OS
if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "Error: This script currently only supports macOS."
    exit 1
fi

# Detect Arch
ARCH=$(uname -m)
if [[ "$ARCH" == "x86_64" ]]; then
    TARGET="x86_64-apple-darwin"
elif [[ "$ARCH" == "arm64" ]]; then
    TARGET="aarch64-apple-darwin"
else
    echo "Error: Unsupported architecture $ARCH."
    exit 1
fi

echo "Fetching latest version..."
# Use GitHub API to get the latest release tag
LATEST_TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [[ -z "$LATEST_TAG" ]]; then
    echo "Error: Could not find latest release. Please ensure a release exists on GitHub."
    exit 1
fi

echo "Latest version: $LATEST_TAG"

ASSET_NAME="${BIN_NAME}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/$ASSET_NAME"

echo "Downloading $DOWNLOAD_URL..."
curl -L -o "$ASSET_NAME" "$DOWNLOAD_URL"

echo "Extracting..."
tar -xzf "$ASSET_NAME"
rm "$ASSET_NAME"

chmod +x "$BIN_NAME"

INSTALL_DIR="/usr/local/bin"

if [ ! -d "$INSTALL_DIR" ]; then
    echo "Directory $INSTALL_DIR does not exist. Creating..."
    sudo mkdir -p "$INSTALL_DIR"
fi

echo "Installing to $INSTALL_DIR (requires sudo)..."
sudo mv "$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"

echo "Success! $BIN_NAME installed to $INSTALL_DIR/$BIN_NAME"
echo "Run '$BIN_NAME --version' to verify."
