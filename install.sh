#!/bin/bash
set -e

# loft installation script
# Usage: curl -fsSL https://loft.fargone.sh/install.sh | sh

REPO="tascord/twang"
BINARY_NAME="loft"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

echo "Installing loft..."

# Map OS and architecture to release binary names
case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64)
                PLATFORM="linux-x86_64"
                ;;
            aarch64|arm64)
                PLATFORM="linux-aarch64"
                ;;
            *)
                echo "Error: Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            x86_64)
                PLATFORM="macos-x86_64"
                ;;
            arm64)
                PLATFORM="macos-aarch64"
                ;;
            *)
                echo "Error: Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    MINGW*|MSYS*|CYGWIN*)
        PLATFORM="windows-x86_64"
        BINARY_NAME="loft.exe"
        ;;
    *)
        echo "Error: Unsupported operating system: $OS"
        exit 1
        ;;
esac

# Determine installation directory
INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
    echo "Note: /usr/local/bin is not writable, installing to $INSTALL_DIR"
    echo "Make sure $INSTALL_DIR is in your PATH"
fi

# Download the binary
DOWNLOAD_URL="https://github.com/$REPO/releases/latest/download/loft-$PLATFORM"
TEMP_FILE="/tmp/$BINARY_NAME"

echo "Downloading from $DOWNLOAD_URL..."
if command -v curl > /dev/null 2>&1; then
    curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_FILE"
elif command -v wget > /dev/null 2>&1; then
    wget -q "$DOWNLOAD_URL" -O "$TEMP_FILE"
else
    echo "Error: Neither curl nor wget is available. Please install one and try again."
    exit 1
fi

# Make binary executable
chmod +x "$TEMP_FILE"

# Move to installation directory
mv "$TEMP_FILE" "$INSTALL_DIR/$BINARY_NAME"

echo "Successfully installed loft to $INSTALL_DIR/$BINARY_NAME"
echo ""
echo "To get started, run:"
echo "  loft --help"
echo ""
echo "To verify installation:"
echo "  loft --version"
