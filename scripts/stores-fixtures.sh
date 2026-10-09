#!/usr/bin/env bash
set -euo pipefail

repo="rgdevment/Tisty"
here=$(cd "$(dirname "$0")/.." && pwd)
into="$here/fixtures/stores"
force=0
pairs=()

usage() {
  cat <<'EOF'
Writes the reference stores under fixtures/stores/ with the command line of earlier
releases, so what a test reads is what Tisty really wrote and not what a test thinks it wrote.

  bash scripts/stores-fixtures.sh [--force] [--into DIR] [VERSION:SCHEMA ...]

With no pairs it writes 1.23.1:15 and 1.24.4:16. Each release is downloaded, its attestation
is checked with `gh attestation verify`, and it runs against folders of its own, never against
the data of whoever runs this. It needs gh, and unzip on Windows or tar on macOS.
EOF
}

fail() {
  printf 'x  %s\n' "$1" >&2
  exit 1
}

while [ $# -gt 0 ]; do
  case $1 in
    --force) force=1 ;;
    --into)
      shift
      [ $# -gt 0 ] || fail "--into wants a folder"
      into=$1
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *:*) pairs+=("$1") ;;
    *) fail "$1 is neither an option nor VERSION:SCHEMA" ;;
  esac
  shift
done
[ ${#pairs[@]} -gt 0 ] || pairs=("1.23.1:15" "1.24.4:16")

case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*)
    exe="tisty.exe"
    asset() { printf 'tisty-cli-%s-windows-x86_64.zip' "$1"; }
    ;;
  Darwin)
    exe="tisty"
    asset() { printf 'tisty-cli-%s-macos-universal.tar.gz' "$1"; }
    ;;
  *) fail "the releases carry no command line for this system" ;;
esac

command -v gh > /dev/null || fail "gh is needed to download the releases and check them"

native() {
  if command -v cygpath > /dev/null 2>&1; then
    cygpath -m "$1"
  else
    printf '%s' "$1"
  fi
}

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

bin=""

fetch() {
  local version=$1 file dir
  file=$(asset "$version")
  dir="$work/$version"
  mkdir -p "$dir/unpacked"
  gh release download "v$version" --repo "$repo" --pattern "$file" --dir "$dir" \
    || fail "$file could not be downloaded"
  gh attestation verify "$dir/$file" --repo "$repo" > /dev/null \
    || fail "$file is not what $repo built, so nothing was run"
  case $file in
    *.zip) unzip -q "$dir/$file" -d "$dir/unpacked" || fail "$file would not unpack" ;;
    *) tar xzf "$dir/$file" -C "$dir/unpacked" || fail "$file would not unpack" ;;
  esac
  bin=$(find "$dir/unpacked" -type f -name "$exe" | sed -n '1p')
  [ -n "$bin" ] || fail "$file holds no $exe"
}

old() {
  printf '  tisty %s\n' "$*"
  "$bin" "$@" < /dev/null || fail "tisty $* failed"
}

write_store() {
  local version=$1 schema=$2 run target said seen
  target="$into/v$schema"
  if [ -e "$target" ] && [ "$force" -eq 0 ]; then
    fail "$target is already there; --force writes it again, with new ids and new times"
  fi

  printf '== %s (schema %s)\n' "$version" "$schema"
  fetch "$version"
  run="$work/$version-run"
  mkdir -p "$run/shared"

  export TISTY_DATA TISTY_CONFIG TISTY_CACHE TZ NO_COLOR LANG
  TISTY_DATA=$(native "$run/data")
  TISTY_CONFIG=$(native "$run/config")
  TISTY_CACHE=$(native "$run/cache")
  TZ=UTC
  NO_COLOR=1
  LANG=en_US.UTF-8
  unset LC_ALL LC_MESSAGES

  said=$("$bin" --version < /dev/null) || fail "$bin would not say its version"
  case $said in
    *"$version"*) ;;
    *) fail "asked for $version and got: $said" ;;
  esac

  old config set remote "$(native "$run/shared")"
  old "write the report"
  old "call the bank"
  old "buy bread"
  old done bread
  old sync
  old "renew the certificate"
  old sync

  seen=$(grep -rhoE '"v":[0-9]+' "$run/data/store" --include='*.tisty' | cut -d: -f2 | sort -n | tail -n 1) || seen=""
  [ "$seen" = "$schema" ] || fail "wrote schema ${seen:-nothing}, not $schema"

  rm -rf "$target"
  mkdir -p "$target"
  cp -R "$run/data/store/." "$target/"
  find "$target" \( -name '.lock' -o -name '.store-key' \) -delete
  printf '%s\n' "written by tisty $version, schema $schema" > "$target.txt"
  printf '   %s\n' "$(find "$target" -type f | wc -l | tr -d ' ') files in $target"
}

for pair in "${pairs[@]}"; do
  write_store "${pair%%:*}" "${pair##*:}"
done

printf '\nLook through what was written before it is committed:\n'
find "$into" -type f | sort
