#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
lock="$root/app/src-tauri/skills.lock"
into="$root/app/src-tauri/resources/skills"

said() { sed -n "s/^$1 //p" "$lock"; }
named() { awk '$1 == "file" { print $3 }' "$lock"; }

if [ "${1:-}" = files ]; then
  listed=$(named)
  if [ -z "$listed" ]; then
    echo "::error::skills.lock names no file to ship" >&2
    exit 1
  fi
  echo "$listed" | sed 's|^|resources/skills/|'
  exit 0
fi

sha256() {
  if command -v sha256sum > /dev/null 2>&1; then
    sha256sum "$1" | cut -d' ' -f1
  else
    shasum -a 256 "$1" | cut -d' ' -f1
  fi
}

repo=$(said repo)
commit=$(said commit)
from=$(said from)
if [ -z "$repo" ] || [ "${#commit}" -ne 40 ] || [ -z "$from" ]; then
  echo "::error::skills.lock names no repository, full commit and folder"
  exit 1
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
git init -q "$work/repo"
git -C "$work/repo" fetch -q --depth 1 "$repo" "$commit"
mkdir "$work/out"
# With autocrlf on, as Windows runners have it, git archive hands back CRLF bytes the commit never held.
git -C "$work/repo" -c core.autocrlf=false -c core.eol=lf archive "$commit" "$from" | tar -x -C "$work/out"
got="$work/out/$from"

found=$(cd "$got" && find . -type f | sed 's|^\./||' | LC_ALL=C sort)
wanted=$(named | LC_ALL=C sort)
if [ "$found" != "$wanted" ]; then
  echo "::error::$commit holds other files than skills.lock names"
  diff <(echo "$wanted") <(echo "$found") || true
  exit 1
fi

wrong=0
while read -r sum name; do
  if [ "$(sha256 "$got/$name")" != "$sum" ]; then
    echo "::error::$name is not the file skills.lock names"
    wrong=1
  fi
done < <(awk '$1 == "file" { print $2, $3 }' "$lock")
[ "$wrong" = 0 ] || exit 1

rm -rf "$into"
mkdir -p "$into"
cp -R "$got/." "$into/"
echo "$(echo "$wanted" | wc -l | tr -d ' ') files of $repo at $commit -> $into"
