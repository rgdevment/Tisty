#!/usr/bin/env bash
set -uo pipefail

status=0

amiss() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    printf '::error::%s\n' "$1"
  else
    printf 'x  %s\n' "$1"
  fi
  status=1
}

went_well() {
  [ -n "${GITHUB_ACTIONS:-}" ] || printf 'ok %s\n' "$1"
}

no_prose_blocks() {
  local found
  found=$(find crates/*/src app/src-tauri/src -name '*.rs' -print0 \
    | xargs -0 awk '
        /^[[:space:]]*\/\/\// { run = 0; next }
        /^[[:space:]]*\/\// && !/\/\/!|TODO|FIXME|SAFETY|noqa|https?:\/\// { run++; if (run == 4) print FILENAME ":" FNR; next }
        { run = 0 }')
  if [ -n "$found" ]; then
    printf '%s\n' "$found"
    amiss "four lines of comment is prose; that goes in planning/, not in the source"
  else
    went_well "no prose blocks in the code"
  fi
}

nothing_past_what_a_person_holds() {
  local measured tables_not_code test_modules now at was before
  measured=$(mktemp)
  tables_not_code='locales\.ts|glyphs\.ts|marks\.ts|model/mark\.rs'
  test_modules=$(grep -rhA1 '^#\[cfg(test)\]$' crates/*/src app/src-tauri/src --include='*.rs' \
    | grep -oE '#\[path = "[^"]+"\]' | grep -oE '"[^"]+"' | tr -d '"' | sort -u | paste -sd'|' -)
  find crates/*/src app/src app/src-tauri/src \( -name '*.rs' -o -name '*.ts' -o -name '*.tsx' \) \
    | grep -vE "node_modules|/tests/|\.test\.|$tables_not_code|$test_modules" \
    | sort | while read -r one; do
      printf '%s %s\n' "$(wc -l < "$one" | tr -d ' ')" "$one"
    done > "$measured"

  before=$status
  while read -r now at; do
    was=$(tr -d '\015' < .github/oversized.txt | awk -v at="$at" '$2 == at { print $1 }')
    if [ -z "$was" ]; then
      if [ "$now" -gt 1500 ]; then
        amiss "$at is $now lines of code; 1500 is the ceiling. Split it along a seam, or say why it belongs in .github/oversized.txt"
      fi
    elif [ "$now" -gt "$was" ]; then
      amiss "$at was already over the ceiling at $was lines and grew to $now. What is above it only shrinks"
    elif [ "$now" -lt "$was" ]; then
      amiss "$at is down to $now lines from $was. Write that into .github/oversized.txt so it cannot grow back"
    fi
  done < "$measured"

  while read -r _ at; do
    if [ ! -f "$at" ]; then
      amiss "$at is in .github/oversized.txt and no longer exists; take the line out"
    fi
  done < <(tr -d '\015' < .github/oversized.txt)

  rm -f "$measured"
  [ "$status" != "$before" ] || went_well "no file grows past what a person can hold"
}

read_by_a_person() {
  [ -f .github/not-this-spanish.txt ] || return 0
  local where=(
    app/src/locales.ts
    crates/tisty-cli/locales
    README.es.md
    app/src-tauri/resources/guide/es/guia.md
  )
  if grep -rniEf .github/not-this-spanish.txt "${where[@]}"; then
    amiss "the Spanish a person reads is neutral and not peninsular"
  else
    went_well "the Spanish a person reads"
  fi
}

nothing_the_core_prints() {
  if grep -rn 'println!\|eprintln!\|print!' crates/tisty-core/src --include='*.rs'; then
    amiss "tisty-core must not print: the GUI inherits it as garbage"
  else
    went_well "the core produces no terminal output"
  fi
}

cd "$(dirname "$0")/.."
no_prose_blocks
nothing_past_what_a_person_holds
read_by_a_person
nothing_the_core_prints
exit $status
