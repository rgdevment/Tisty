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
  declare -A ceiling
  measured=$(mktemp)
  skip='node_modules|/tests/|\.test\.|locales\.ts|glyphs\.ts|marks\.ts|model/mark\.rs'
  test_modules=$(grep -rhA1 '^#\[cfg(test)\]$' crates/*/src app/src-tauri/src --include='*.rs' \
    | grep -oE '#\[path = "[^"]+"\]' | grep -oE '"[^"]+"' | tr -d '"' | sort -u | paste -sd'|' -)
  if [ -n "$test_modules" ]; then
    skip="$skip|$test_modules"
  fi
  find crates/*/src app/src app/src-tauri/src \( -name '*.rs' -o -name '*.ts' -o -name '*.tsx' \) \
    | grep -vE "$skip" \
    | sort | tr '\n' '\0' | xargs -0 wc -l \
    | awk '$2 != "total" { print $1, $2 }' > "$measured"

  if [ ! -s "$measured" ]; then
    rm -f "$measured"
    amiss "no source file was found to measure, so the ceiling was never looked at"
    return
  fi

  while read -r now at; do
    [ -n "${at:-}" ] && ceiling["$at"]=$now
  done < <(tr -d '\015' < .github/oversized.txt)

  before=$status
  while read -r now at; do
    was=${ceiling["$at"]:-}
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

written_in_english() {
  grep -rnE '\b(fn|let|const|struct|enum|type|mod|function|interface) +[A-Za-z_]*(^|_)(que|de|la|el|los|las|una|con|por|para|sin|cuando|donde|hasta|desde|tarea|fecha|titulo|nombre|usuario|archivo|carpeta)(_|\b)' \
    crates/*/src app/src app/src-tauri/src --include='*.rs' --include='*.ts' --include='*.tsx'
  looked_through \
    "an identifier is in Spanish; what a person reads lives in locales/" $? \
    "the source could not be looked through for Spanish identifiers" \
    || return
  grep -rnE '^\s*//.*\b(que|porque|pero|aunque|cuando|donde|seg[uú]n|tambi[eé]n|aqu[ií]|ah[ií]|entonces)\b' \
    crates/*/src app/src app/src-tauri/src --include='*.rs' --include='*.ts' --include='*.tsx'
  looked_through \
    "a comment is in Spanish; code, comments and test names are English" $? \
    "the source could not be looked through for Spanish comments" \
    || return
  grep -rnE '(bail|anyhow|panic|expect|format)!?\(\s*"[^"]*\b(que|porque|pero|cuando|seg[uú]n|tambi[eé]n)\b' \
    crates/*/src app/src-tauri/src --include='*.rs' | grep -vE 'locales|_test'
  looked_through \
    "an error message is in Spanish; what a person reads lives in locales/" $? \
    "the source could not be looked through for Spanish error messages" \
    && went_well "code, comments and error messages in English"
}

nothing_the_window_cannot_translate() {
  grep -rnE 'println!\("[a-záéíóúñ]|bail!\("[a-záéíóúñ]' \
    crates/tisty-cli/src --include='*.rs'
  looked_through \
    "interface text belongs in locales/, not in the source" $? \
    "the command line could not be looked through for interface text" \
    && went_well "no interface strings hardcoded"
}

nothing_that_takes_the_window_down() {
  local test_modules looked found
  test_modules=$(grep -rhA1 '^#\[cfg(test)\]$' crates/*/src --include='*.rs' \
    | grep -oE '#\[path = "[^"]+"\]' | grep -oE '"[^"]+"' | tr -d '"' | sort -u | paste -sd'|' -)
  looked=$(find crates/tisty-core/src crates/tisty-sync/src -name '*.rs' \
    | grep -vE "_tests?\.rs$|${test_modules:-^$}")
  if [ -z "$looked" ]; then
    amiss "the core and the round could not be looked through for what panics"
    return
  fi
  found=$(printf '%s\n' "$looked" | tr '\n' '\0' \
    | xargs -0 grep -nE '\.unwrap\(\)|\.expect\(|panic!\(|unreachable!\(|todo!\(')
  if [ -n "$found" ]; then
    printf '%s\n' "$found"
    amiss "the core and the round answer, they never stop: a panic reaches the window as a closed window"
  else
    went_well "nothing in the core or the round panics"
  fi
}

nothing_the_core_prints() {
  grep -rn 'println!\|eprintln!\|print!\|dbg!\|io::stdin\|io::stdout\|io::stderr' \
    crates/tisty-core/src --include='*.rs'
  looked_through \
    "tisty-core must not print: the GUI inherits it as garbage" $? \
    "tisty-core could not be looked through for what it prints" \
    && went_well "the core produces no terminal output"
}

nothing_new_reaches_the_command_line() {
  local kept now added
  kept=.github/frozen-cli.txt
  if [ ! -f "$kept" ]; then
    amiss "what the command line already carries is not written down, so nobody saw it grow"
    return
  fi
  now=$( { awk '/^pub enum Command/,/^}/' crates/tisty-cli/src/main.rs \
            | awk '/^    [A-Z]/ { kept = ($0 ~ /^    (Demo|Sync|Doctor|Export|Mcp|Leave|Agent)\b/) } !kept'
          awk '/^pub struct (AddArgs|SetArgs)/,/^}/' crates/tisty-cli/src/main.rs; } \
          | grep -oE '^    [A-Z][A-Za-z]*|^ +(pub )?[a-z_]+:' | sed 's/pub //; s/[ :]//g' | sort -u)
  if [ -z "$now" ]; then
    amiss "the command line could not be read, so nobody saw whether it grew"
    return
  fi
  added=$(comm -13 <(sort "$kept") <(printf '%s\n' "$now"))
  if [ -n "$added" ]; then
    printf '%s\n' "$added"
    amiss "the command line is frozen: a feature is done when the core and the window have it"
  else
    went_well "nothing new reached the command line"
  fi
}

cd "$(dirname "$0")/.." || exit 2
no_prose_blocks
nothing_past_what_a_person_holds
read_by_a_person
written_in_english
nothing_the_window_cannot_translate
nothing_the_core_prints
nothing_that_takes_the_window_down
nothing_new_reaches_the_command_line
exit $status
