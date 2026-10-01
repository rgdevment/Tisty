#!/usr/bin/env bash
set -uo pipefail

range=${1:-}
[ -n "$range" ] || { echo "usage: commits.sh <range>"; exit 2; }

shape='^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\([a-z0-9._-]+\))?!?: .+'
most=90
status=0

amiss() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    printf '::error::%s\n' "$1"
  else
    printf 'x  %s\n' "$1"
  fi
  status=1
}

seen=0
while read -r sha; do
  [ -n "$sha" ] || continue
  seen=$((seen + 1))
  said=$(git log -1 --format=%s "$sha" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
  if ! printf '%s' "$said" | grep -qE "$shape"; then
    amiss "${sha:0:8} does not follow conventional commits"
    printf '  %s\n' "$said"
    continue
  fi
  if [ "${#said}" -gt "$most" ]; then
    amiss "${sha:0:8} subject is ${#said} characters, keep it under $most"
    printf '  %s\n' "$said"
  fi
done < <(git rev-list --no-merges "$range")

if [ "$status" -eq 1 ]; then
  echo ""
  echo "Expected: type(optional-scope): description"
  echo "Types:    feat fix docs style refactor perf test build ci chore revert"
  echo "Example:  feat(parser): recognise weekday names in Spanish"
  exit 1
fi

[ -n "${GITHUB_ACTIONS:-}" ] || printf 'ok %s commit subject(s) well formed\n' "$seen"
