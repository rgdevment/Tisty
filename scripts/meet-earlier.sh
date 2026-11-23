#!/usr/bin/env bash
set -euo pipefail

[ $# -eq 5 ] || { echo "usage: meet-earlier.sh <release> <schema> <shape> <accepts> <built>" >&2; exit 2; }
release=$1 schema=$2 shape=$3 accepts=$4 built=$5

built_from_its_tree() {
  local tree="earlier/$release" its_own="earlier/built/$release" shared_by_every_build=earlier/target/debug/examples
  mkdir -p "$tree/crates/tisty-cli/examples" "$its_own"
  cp crates/tisty-cli/examples/earlier.rs "$tree/crates/tisty-cli/examples/earlier.rs"
  sed -i.bak -e "s/^version = .*/version = \"$release\"/" "$tree/Cargo.toml"
  rm -f "$tree/Cargo.toml.bak"
  grep -q "^version = \"$release\"\$" "$tree/Cargo.toml" \
    || { echo "::error::could not stamp $release into its tree"; exit 1; }
  rm -f "$shared_by_every_build/earlier" "$shared_by_every_build/earlier.exe"
  # A newer compiler's warnings in a release's own tree are not this build's to fix.
  env -u GH_TOKEN RUSTFLAGS="" cargo build --manifest-path "$tree/Cargo.toml" -p tisty-cli \
    --example earlier --target-dir earlier/target
  find "$shared_by_every_build" -maxdepth 1 -type f \( -name earlier -o -name earlier.exe \) -exec cp {} "$its_own/" \;
  bin=$(find "$its_own" -type f | sed -n '1p')
  [ -n "$bin" ] || { echo "::error::v$release built no earlier"; exit 1; }
}

released() {
  local file kept=earlier/downloaded unpacked="earlier/released/$release"
  case "$RUNNER_OS" in
    Windows) file="tisty-cli-$release-windows-x86_64.zip" ;;
    macOS) file="tisty-cli-$release-macos-universal.tar.gz" ;;
    *) echo "::error::no command line is released for $RUNNER_OS"; exit 1 ;;
  esac
  mkdir -p "$kept" "$unpacked"
  if [ -f "$kept/$file" ]; then
    echo "$file is kept from an earlier run"
  else
    gh release download "v$release" --repo "$GITHUB_REPOSITORY" --pattern "$file" --dir "$kept"
  fi
  # A kept file is checked like a fresh one: the cache is not trusted more than the release.
  gh attestation verify "$kept/$file" --repo "$GITHUB_REPOSITORY" \
    --signer-workflow "$GITHUB_REPOSITORY/.github/workflows/release.yml"
  case "$file" in
    *.zip) unzip -q "$kept/$file" -d "$unpacked" ;;
    *) tar xzf "$kept/$file" -C "$unpacked" ;;
  esac
  bin=$(find "$unpacked" -type f \( -name tisty -o -name tisty.exe \) | sed -n '1p')
  [ -n "$bin" ] || { echo "::error::$file holds no tisty"; exit 1; }
}

if [ "$built" = built ]; then built_from_its_tree; else released; fi

bin="$PWD/$bin"
if command -v cygpath > /dev/null; then bin=$(cygpath -w "$bin"); fi

env -u GH_TOKEN TISTY_PREVIOUS="$bin" TISTY_PREVIOUS_SCHEMA="$schema" TISTY_PREVIOUS_SHAPE="$shape" \
  TISTY_PREVIOUS_ACCEPTS="$accepts" cargo test -p tisty-cli --test versions -- --ignored
