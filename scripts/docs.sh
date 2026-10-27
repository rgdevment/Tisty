#!/usr/bin/env bash
set -uo pipefail

cd "$(dirname "$0")/.." || exit 2

version=${1:?a version is needed, without the leading v}
mark=docs/read-against
read_by_people="README.md, README.es.md, PRIVACY.md, SECURITY.md, the guide in app/src-tauri/resources/guide, the Markdown reference in crates/tisty-markdown/reference and the help in docs/help"

amiss() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    printf '::error::%s\n' "$1"
  else
    printf 'x  %s\n' "$1"
  fi
}

case $version in
  *-*)
    printf 'ok %s is a candidate, and its documents are read against the release\n' "$version"
    exit 0
    ;;
esac

said=$(tr -d '[:space:]' < "$mark" 2> /dev/null)
minor="${version%.*}.0"

if [ "$said" = "$version" ]; then
  printf 'ok what a person reads was read against %s\n' "$version"
  exit 0
fi

if [ "$said" = "$minor" ]; then
  printf 'ok %s is a patch of %s, whose documents were read against it\n' "$version" "$minor"
  exit 0
fi

amiss "$mark says ${said:-nothing}, so nobody read what a person reads against $version: $read_by_people. \
The guide ships inside the app, so read them, bring them up to date and write $version in $mark before the tag goes out"
exit 1
