#!/usr/bin/env bash
# A Windows machine without the Visual C++ redistributable could not start such a binary, and would say nothing.
set -euo pipefail

if grep -aio 'vcruntime[0-9_]*\.dll\|msvcp[0-9_]*\.dll' "$1"; then
  echo "::error::$1 imports the Visual C++ runtime; crates/tisty-cli/build.rs links it statically"
  exit 1
fi
