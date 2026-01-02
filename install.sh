#!/bin/bash
set -e

INSTALL_DIR="$HOME/.local/bin"

mkdir -p "$INSTALL_DIR"
cp target/release/clazydbm "$INSTALL_DIR/"

echo "Installed clazydbm to $INSTALL_DIR/clazydbm"
