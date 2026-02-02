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

ASSET_NAME="${BIN_NAME}-${TARGET}"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/$ASSET_NAME"

echo "Downloading $DOWNLOAD_URL..."
curl -L -o "$BIN_NAME" "$DOWNLOAD_URL"

chmod +x "$BIN_NAME"

INSTALL_DIR="$HOME/.local/bin"

if [ ! -d "$INSTALL_DIR" ]; then
    echo "Directory $INSTALL_DIR does not exist. Creating..."
    mkdir -p "$INSTALL_DIR"
fi

echo "Installing to $INSTALL_DIR..."
mv "$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"

# Check if INSTALL_DIR is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    SHELL_CONFIG=""
    case "$SHELL" in
      */zsh)
        SHELL_CONFIG="$HOME/.zshrc"
        ;;
      */bash)
        if [ -f "$HOME/.bashrc" ]; then
          SHELL_CONFIG="$HOME/.bashrc"
        elif [ -f "$HOME/.bash_profile" ]; then
          SHELL_CONFIG="$HOME/.bash_profile"
        fi
        ;;
    esac

    if [ -n "$SHELL_CONFIG" ]; then
        if grep -q "$INSTALL_DIR" "$SHELL_CONFIG"; then
            echo "It seems $INSTALL_DIR is already configured in $SHELL_CONFIG."
        else
            echo "Adding $INSTALL_DIR to PATH in $SHELL_CONFIG..."
            echo "" >> "$SHELL_CONFIG"
            echo "# bsr" >> "$SHELL_CONFIG"
            echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_CONFIG"
            echo "Configuration added. Please run 'source $SHELL_CONFIG' or restart your terminal."
        fi
    else
        echo "Warning: $INSTALL_DIR is not in your PATH."
        echo "Please add the following line to your shell configuration file (e.g., ~/.zshrc or ~/.bash_profile):"
        echo "export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
fi

echo "Success! $BIN_NAME installed to $INSTALL_DIR/$BIN_NAME"
echo "Run '$BIN_NAME --version' to verify."
