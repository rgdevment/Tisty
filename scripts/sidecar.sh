#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

target=$(rustc -vV | sed -n 's/^host: //p')
suffix=""
case "$target" in *windows*) suffix=".exe" ;; esac

cargo build -p tisty-cli --bin tisty
mkdir -p app/src-tauri/binaries
cp "${CARGO_TARGET_DIR:-target}/debug/tisty$suffix" "app/src-tauri/binaries/tisty-$target$suffix"
echo "app/src-tauri/binaries/tisty-$target$suffix"
