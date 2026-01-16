#!/bin/sh
set -e

REPO="imatefx/process-port-check-tui"
VERSION="v1.0.0"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BINARY_NAME="port-checker"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    darwin) OS="macos" ;;
    linux) OS="linux" ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64) ARCH="x64" ;;
    arm64|aarch64) ARCH="arm64" ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

FILENAME="port-checker-${VERSION}-${OS}-${ARCH}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${FILENAME}"

echo "Downloading ${BINARY_NAME} for ${OS}-${ARCH}..."
echo "URL: ${URL}"

# Create temp directory
TMP_DIR=$(mktemp -d)
trap "rm -rf ${TMP_DIR}" EXIT

# Download and extract
curl -sL "${URL}" -o "${TMP_DIR}/${FILENAME}"
tar -xzf "${TMP_DIR}/${FILENAME}" -C "${TMP_DIR}"

# Find the binary (name varies by platform)
BINARY=$(find "${TMP_DIR}" -type f -name "port-checker-*" | head -1)

if [ -z "$BINARY" ]; then
    echo "Error: Binary not found in archive"
    exit 1
fi

# Install
echo "Installing to ${INSTALL_DIR}/${BINARY_NAME}..."
if [ -w "$INSTALL_DIR" ]; then
    mv "$BINARY" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
else
    echo "Need sudo to install to ${INSTALL_DIR}"
    sudo mv "$BINARY" "${INSTALL_DIR}/${BINARY_NAME}"
    sudo chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
fi

echo ""
echo "Successfully installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
echo "Run 'port-checker' to start"
