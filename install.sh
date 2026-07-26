#!/bin/bash
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

INSTALL_DIR="/usr/local/bin"
BIN_NAME="moviebox-tui"
APP_PATH="$INSTALL_DIR/$BIN_NAME"

if command -v "$BIN_NAME" &> /dev/null; then
    echo -e "${YELLOW}>> Updating MovieBox-Tui to the latest version...${NC}"
else
    echo -e "${BLUE}>> Installing MovieBox-Tui...${NC}"
fi

OS="$(uname -s)"
ARCH="$(uname -m)"

if [ "$OS" = "Darwin" ]; then
    FILE="MovieBox_macOS_Universal.tar.gz"
elif [ "$OS" = "Linux" ]; then
    if [ "$ARCH" = "x86_64" ]; then
        FILE="MovieBox_Linux_x64.tar.gz"
    else
        echo -e "${RED}Error: Unsupported Linux architecture ($ARCH). Only x86_64 is supported.${NC}"
        exit 1
    fi
else
    echo -e "${RED}Error: Unsupported OS ($OS).${NC}"
    exit 1
fi

URL="https://github.com/mesamirh/MovieBox-Tui/releases/latest/download/$FILE"
TMP_DIR=$(mktemp -d)

echo -e "${BLUE}>> Downloading latest release...${NC}"
if ! curl -fsSL --progress-bar "$URL" -o "$TMP_DIR/$FILE"; then
    echo -e "${RED}Error: Failed to download release.${NC}"
    rm -rf "$TMP_DIR"
    exit 1
fi

echo -e "${BLUE}>> Extracting...${NC}"
tar -xzf "$TMP_DIR/$FILE" -C "$TMP_DIR"

echo -e "${BLUE}>> Moving binary to $INSTALL_DIR...${NC}"
if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP_DIR/moviebox" "$APP_PATH"
else
    sudo mv "$TMP_DIR/moviebox" "$APP_PATH"
fi

if [ -w "$APP_PATH" ]; then
    chmod +x "$APP_PATH"
else
    sudo chmod +x "$APP_PATH"
fi

rm -rf "$TMP_DIR"

if command -v "$BIN_NAME" &> /dev/null; then
    echo -e "${GREEN}Success! MovieBox-Tui has been updated. Run 'moviebox-tui' to start.${NC}"
else
    echo -e "${GREEN}Success! MovieBox-Tui is installed. Run 'moviebox-tui' to start.${NC}"
fi
