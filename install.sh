#!/bin/sh
set -e

# agt installer — downloads pre-built binary from GitHub releases
# Usage: curl -fsSL https://raw.githubusercontent.com/kaiwen-wang/agent-todo-list/main/install.sh | sh

REPO="kaiwen-wang/agent-todo-list"
INSTALL_DIR="${AGT_INSTALL_DIR:-$HOME/.local/bin}"
SHARE_DIR="${AGT_SHARE_DIR:-$HOME/.local/share/agt}"

# Detect OS and architecture
detect_platform() {
  OS="$(uname -s)"
  ARCH="$(uname -m)"

  case "$OS" in
    Darwin) OS="apple-darwin" ;;
    Linux)  OS="unknown-linux-gnu" ;;
    *)      echo "Unsupported OS: $OS" >&2; exit 1 ;;
  esac

  case "$ARCH" in
    x86_64|amd64)  ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
    *)             echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
  esac

  PLATFORM="${ARCH}-${OS}"
}

# Get the latest release tag from GitHub
get_latest_version() {
  if command -v curl > /dev/null 2>&1; then
    VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"//;s/".*//')"
  elif command -v wget > /dev/null 2>&1; then
    VERSION="$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"//;s/".*//')"
  else
    echo "Error: curl or wget is required" >&2
    exit 1
  fi

  if [ -z "$VERSION" ]; then
    echo "Error: could not determine latest version" >&2
    exit 1
  fi
}

download() {
  URL="https://github.com/${REPO}/releases/download/${VERSION}/agt-${PLATFORM}.tar.gz"
  echo "Downloading agt ${VERSION} for ${PLATFORM}..."

  TMPDIR="$(mktemp -d)"
  trap 'rm -rf "$TMPDIR"' EXIT

  if command -v curl > /dev/null 2>&1; then
    curl -fsSL "$URL" -o "$TMPDIR/agt.tar.gz"
  else
    wget -qO "$TMPDIR/agt.tar.gz" "$URL"
  fi

  tar -xzf "$TMPDIR/agt.tar.gz" -C "$TMPDIR"

  # Install binary
  mkdir -p "$INSTALL_DIR"
  cp "$TMPDIR/agt" "$INSTALL_DIR/agt"
  chmod +x "$INSTALL_DIR/agt"

  # Install web assets if present in the tarball
  if [ -d "$TMPDIR/web" ]; then
    rm -rf "$SHARE_DIR/web"
    mkdir -p "$SHARE_DIR"
    cp -r "$TMPDIR/web" "$SHARE_DIR/web"
  fi
}

check_path() {
  case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
      echo ""
      echo "  Add to your PATH:"
      echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
      ;;
  esac
}

main() {
  detect_platform
  get_latest_version
  download
  echo "Installed agt ${VERSION} to ${INSTALL_DIR}/agt"
  check_path
  echo ""
  echo "Run 'agt init' in a git repo to get started."
}

main
