#!/bin/sh
set -e

REPO="A-Fisk/ruv"
TAG="${1:-latest}"

if [ "$TAG" = "latest" ]; then
  echo "Fetching latest release..."
  TAG=$(curl -s https://api.github.com/repos/$REPO/releases | grep '"tag_name"' | head -1 | sed -E 's/.*"([^"]+)".*/\1/')
  if [ -z "$TAG" ]; then
    echo "Error: Could not determine latest release tag"
    exit 1
  fi
fi

echo "Installing ruv $TAG..."

# Detect OS and architecture
case "$(uname -s)" in
  Darwin)
    case "$(uname -m)" in
      arm64)
        TARGET="aarch64-apple-darwin"
        ;;
      x86_64)
        TARGET="x86_64-apple-darwin"
        ;;
      *)
        echo "Error: Unsupported macOS architecture: $(uname -m)"
        exit 1
        ;;
    esac
    ;;
  Linux)
    case "$(uname -m)" in
      x86_64)
        TARGET="x86_64-unknown-linux-musl"
        ;;
      aarch64)
        TARGET="aarch64-unknown-linux-musl"
        ;;
      *)
        echo "Error: Unsupported Linux architecture: $(uname -m)"
        exit 1
        ;;
    esac
    ;;
  *)
    echo "Error: Unsupported OS: $(uname -s)"
    exit 1
    ;;
esac

# Determine install directory
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$INSTALL_DIR"

# Download and extract
URL="https://github.com/$REPO/releases/download/$TAG/ruv-$TARGET.tar.gz"
echo "Downloading from: $URL"

TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

if ! curl -LsSf "$URL" -o "$TEMP_DIR/ruv.tar.gz"; then
  echo "Error: Failed to download ruv. Check your internet connection and that the tag exists."
  exit 1
fi

echo "Extracting..."
tar -xzf "$TEMP_DIR/ruv.tar.gz" -C "$TEMP_DIR"

# Find the binary (it's in ruv-$TARGET/bin/ruv)
BINARY_PATH="$TEMP_DIR/ruv-$TARGET/bin/ruv"
if [ ! -f "$BINARY_PATH" ]; then
  echo "Error: Binary not found at expected location: $BINARY_PATH"
  ls -la "$TEMP_DIR/"
  exit 1
fi

cp "$BINARY_PATH" "$INSTALL_DIR/ruv"
chmod +x "$INSTALL_DIR/ruv"

echo ""
echo "ruv $TAG installed successfully!"
echo ""

echo "Binary location: $INSTALL_DIR/ruv"
echo ""

# Check if install dir is in PATH (POSIX-compatible)
case ":$PATH:" in
  *":$INSTALL_DIR:"*)
    echo "$INSTALL_DIR is already in your \$PATH"
    echo ""
    echo "You can now run: ruv --help"
    ;;
  *)
    echo "WARNING: $INSTALL_DIR is NOT in your \$PATH"
    echo ""
    echo "Add this line to your shell profile (~/.zshrc, ~/.bashrc, etc):"
    echo ""
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
    echo "Then restart your terminal or run: source ~/.zshrc"
    ;;
esac
