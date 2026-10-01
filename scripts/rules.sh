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

looked_through() {
  case $2 in
    0) amiss "$1"; return 1 ;;
    1) return 0 ;;
    *) amiss "$3"; return 1 ;;
  esac
}

no_prose_blocks() {
  local found
  found=$(find crates/*/src app/src-tauri/src -name '*.rs' -print0 \
    | xargs -0 awk '
        FNR == 1 { run = 0 }
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
  local measured skip test_modules now at was before
  measured=$(mktemp)
  skip='node_modules|/tests/|\.test\.|locales\.ts|glyphs\.ts|marks\.ts|model/mark\.rs'
  test_modules=$(grep -rhA1 '^#\[cfg(test)\]$' crates/*/src app/src-tauri/src --include='*.rs' \
    | grep -oE '#\[path = "[^"]+"\]' | grep -oE '"[^"]+"' | tr -d '"' | sort -u | paste -sd'|' -)
  if [ -n "$test_modules" ]; then
    skip="$skip|$test_modules"
  fi
  find crates/*/src app/src app/src-tauri/src \( -name '*.rs' -o -name '*.ts' -o -name '*.tsx' \) \
    | grep -vE "$skip" \
    | sort | while read -r one; do
      printf '%s %s\n' "$(wc -l < "$one" | tr -d ' ')" "$one"
    done > "$measured"

  if [ ! -s "$measured" ]; then
    rm -f "$measured"
    amiss "no source file was found to measure, so the ceiling was never looked at"
    return
  fi

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
  local where=(
    app/src/locales.ts
    crates/tisty-cli/locales
    README.es.md
    app/src-tauri/resources/guide/es/guia.md
  )
  if [ ! -f .github/not-this-spanish.txt ]; then
    amiss "the words the Spanish may not use are not here, so nobody looked for them"
    return
  fi
  grep -rniEf .github/not-this-spanish.txt "${where[@]}"
  looked_through \
    "the Spanish a person reads is neutral and not peninsular" $? \
    "the Spanish a person reads could not be looked through where it is written" \
    && went_well "the Spanish a person reads"
}

nothing_the_core_prints() {
  grep -rn 'println!\|eprintln!\|print!' crates/tisty-core/src --include='*.rs'
  looked_through \
    "tisty-core must not print: the GUI inherits it as garbage" $? \
    "tisty-core could not be looked through for what it prints" \
    && went_well "the core produces no terminal output"
}

cd "$(dirname "$0")/.." || exit 2
no_prose_blocks
nothing_past_what_a_person_holds
read_by_a_person
nothing_the_core_prints
exit $status
