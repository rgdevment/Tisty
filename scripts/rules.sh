#!/usr/bin/env bash
set -uo pipefail

status=0
missed=0

amiss() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    printf '::error::%s\n' "$1"
  else
    printf 'x  %s\n' "$1"
  fi
  status=1
  missed=$((missed + 1))
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
  local measured kept skip test_modules now at was before
  measured=$(mktemp)
  skip='node_modules|/tests/|\.test\.|locales\.ts|glyphs\.ts|marks\.ts|model/mark\.rs'
  test_modules=$(grep -rhA1 '^#\[cfg(test)\]$' crates/*/src app/src-tauri/src --include='*.rs' \
    | grep -oE '#\[path = "[^"]+"\]' | grep -oE '"[^"]+"' | tr -d '"' | sort -u | paste -sd'|' -)
  if [ -n "$test_modules" ]; then
    skip="$skip|$test_modules"
  fi
  find crates/*/src app/src app/src-tauri/src \( -name '*.rs' -o -name '*.ts' -o -name '*.tsx' \) \
    | grep -vE "$skip" \
    | sort | tr '\n' '\0' | xargs -0r wc -l \
    | awk '$2 != "total" { print $1, $2 }' > "$measured"

  if [ ! -s "$measured" ]; then
    rm -f "$measured"
    amiss "no source file was found to measure, so the ceiling was never looked at"
    return
  fi

  kept=$(mktemp)
  tr -d '\015' < .github/oversized.txt > "$kept"

  before=$missed
  while read -r now at; do
    was=$(awk -v at="$at" '$2 == at { last = $1 } END { print last }' "$kept")
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

  rm -f "$measured" "$kept"
  [ "$missed" != "$before" ] || went_well "no file grows past what a person can hold"
}

written_in_english() {
  local spanish='(tarea|fecha|limite|prioridad|filtro|titulo|etiqueta|nombre|usuario|archivo|carpeta|cuando|donde|hasta|desde|porque|aunque)'
  grep -rnE "\b(fn|let|const|struct|enum|type|mod|function|interface) +([A-Za-z_]*_)?$spanish(_|\b)" \
    crates/*/src app/src app/src-tauri/src --include='*.rs' --include='*.ts' --include='*.tsx'
  looked_through \
    "an identifier is in Spanish; what a person reads lives in locales/" $? \
    "the source could not be looked through for Spanish identifiers" \
    || return
  grep -rnE "\b$spanish *:" \
    crates/*/src app/src app/src-tauri/src --include='*.rs' --include='*.ts' --include='*.tsx' \
    | grep -v '"'
  looked_through \
    "a field is named in Spanish; what a person reads lives in locales/" $? \
    "the source could not be looked through for Spanish field names" \
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
  local test_modules looked found one
  local -a files
  test_modules=$(grep -rhA1 '^#\[cfg(test)\]$' crates/*/src --include='*.rs' \
    | grep -oE '#\[path = "[^"]+"\]' | grep -oE '"[^"]+"' | tr -d '"' | sort -u | paste -sd'|' -)
  looked=$(find crates/tisty-core/src crates/tisty-sync/src crates/tisty-carrier/src -name '*.rs' \
    | grep -vE "_tests?\.rs$|${test_modules:-^$}")
  if [ -z "$looked" ]; then
    amiss "the core and the round could not be looked through for what panics"
    return
  fi
  files=()
  while IFS= read -r one; do
    files+=("$one")
  done <<< "$looked"
  found=$(grep -nHE '\.unwrap\(\)|\.expect\(|panic!\(|unreachable!\(|todo!\(' "${files[@]}")
  case $? in
    0)
      printf '%s\n' "$found"
      amiss "the core and the round answer, they never stop: a panic reaches the window as a closed window"
      ;;
    1) went_well "nothing in the core or the round panics" ;;
    *) amiss "the core and the round could not be looked through for what panics" ;;
  esac
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
            | awk '/^    [A-Z]/ { kept = ($0 ~ /^    (Demo|Sync|Doctor|Export|Mcp|Leave|Agent)([^A-Za-z]|$)/) } !kept'
          awk '/^pub (enum [A-Za-z]*Action|struct (Cli|AddArgs|SetArgs))/,/^}/' \
            crates/tisty-cli/src/main.rs; } \
          | grep -oE '^    [A-Z][A-Za-z]*|^ +(pub )?[a-z_]+:|alias = "[a-z-]+"' \
          | sed 's/pub //; s/alias = //; s/[ :"]//g' | sort -u)
  if [ -z "$now" ]; then
    amiss "the command line could not be read, so nobody saw whether it grew"
    return
  fi
  added=$(comm -13 <(tr -d '\015' < "$kept" | sort) <(printf '%s\n' "$now"))
  if [ -n "$added" ]; then
    printf '%s\n' "$added"
    amiss "the command line is frozen: a feature is done when the core and the window have it"
  else
    went_well "nothing new reached the command line"
  fi
}

every_spawn_pins_its_language() {
  local at spawns pinned bad=0
  if [ ! -d crates/tisty-cli/tests ]; then
    amiss "crates/tisty-cli/tests is not here, so no spawned binary was looked at"
    return
  fi
  while IFS= read -r at; do
    spawns=$(grep -ac 'Command::new(env!("CARGO_BIN_EXE_tisty"))' "$at")
    pinned=$(grep -aA9 'Command::new(env!("CARGO_BIN_EXE_tisty"))' "$at" \
      | grep -cE 'env_clear\(\)|env_remove\("LC_ALL"\)')
    [ "$spawns" = "$pinned" ] \
      || { amiss "$at spawns the binary $spawns time(s) and pins the language $pinned"; bad=1; }
  done < <(grep -ral 'CARGO_BIN_EXE_tisty' crates/tisty-cli/tests)
  [ "$bad" = 1 ] || went_well "every spawned binary has its language pinned"
}

both_languages_carry_the_same_documents() {
  local pair one mark a b bad=0
  for pair in "README.md README.es.md" \
    "app/src-tauri/resources/guide/en/guide.md app/src-tauri/resources/guide/es/guia.md"; do
    set -- $pair
    for one in "$1" "$2"; do
      if [ ! -f "$one" ]; then
        amiss "$one is not here, so its other language was not compared"
        bad=1
        continue 2
      fi
    done
    for mark in '^#' '```' '^|'; do
      a=$(grep -c -- "$mark" "$1")
      b=$(grep -c -- "$mark" "$2")
      [ "$a" = "$b" ] || { amiss "$1 and $2 differ in '$mark': $a vs $b"; bad=1; }
    done
  done
  [ "$bad" = 1 ] || went_well "both languages carry the same documents"
}

# Only the type and the factory may name a way of syncing, so no match on it can grow back.
modes_named_outside() {
  local test_modules found naming
  naming='(^|[^[:alnum:]_])Sync[[:space:]]*::[[:space:]]*[A-Z{*]|(^|[^[:alnum:]_])Sync[[:space:]]+as[[:space:]]'
  naming="$naming|impl[[:space:]]*(<[^>]*>[[:space:]]*)?([A-Za-z_]+::)*Sync[[:space:]]*[{<]|for[[:space:]]+([A-Za-z_]+::)*Sync[[:space:]]*[{<]"
  naming="$naming|=[[:space:]]*([A-Za-z_]+::)*Sync[[:space:]]*;"
  test_modules=$(grep -rA1 '^#\[cfg(test)\]$' "$@" --include='*.rs' \
    | sed -nE 's/^(.*\.rs)-#\[path = "([^"]+)"\]$/\1 \2/p' \
    | while read -r declared module; do printf '%s/%s\n' "$(dirname "$declared")" "$module"; done | sort -u)
  found=$(grep -rnE "$naming" "$@" --include='*.rs')
  case $? in 0 | 1) ;; *) return 2 ;; esac
  [ -n "$found" ] || return 0
  printf '%s\n' "$found" | NAMES="$test_modules" awk '
    BEGIN { count = split(ENVIRON["NAMES"], list, "\n"); for (i = 1; i <= count; i++) skip[list[i]] = 1 }
    {
      file = $0
      sub(/:[0-9]+:.*/, "", file)
      parts = split(file, at, "/")
      base = at[parts]
      if (file ~ /(^|\/)tests\// || base ~ /_tests?\.rs$/ || (file in skip)) next
      if (file == "crates/tisty-core/src/config.rs" || file == "crates/tisty-carrier/src/lib.rs") next
      print
    }'
}

the_way_of_syncing_is_named_once() {
  local found
  found=$(modes_named_outside crates app/src-tauri/src)
  case $? in
    0) ;;
    *) amiss "the sources could not be looked through for a way of syncing named outside its place"; return ;;
  esac
  if [ -n "$found" ]; then
    printf '%s\n' "$found"
    amiss "a way of syncing is named only in config.rs and in chosen(): ask the carrier, or add a method to Sync"
  else
    went_well "the way of syncing is named only where it is chosen"
  fi
}

the_rule_on_the_way_of_syncing_can_fail() {
  local room out broke allowed
  room=$(mktemp -d)
  mkdir -p "$room/crates/other/src" "$room/crates/tisty-carrier/src"
  printf '%s\n' 'fn f(s: Sync) { match s { Sync::Folder(_) => {}, _ => {} } }' \
    '#[cfg(test)]' '#[path = "checks.rs"]' 'mod checks;' > "$room/crates/other/src/lib.rs"
  printf '%s\n' 'fn g(s: Sync) { match s { Sync::Folder(_) => {}, _ => {} } }' > "$room/crates/other/src/checks.rs"
  printf '%s\n' 'impl<T> Sync { }' 'impl crate::config::Sync { }' 'impl Trait for tisty_core::config::Sync { }' \
    'use tisty_core::config::Sync as Way;' 'use tisty_core::config::Sync::*;' 'type Way = tisty_core::config::Sync;' \
    'use tisty_core::config::Sync::{Folder, Local};' > "$room/crates/other/src/forms.rs"
  printf '%s\n' 'unsafe impl Sync for Cell {}' 'unsafe impl<T: Send> Sync for Holder<T> {}' \
    'fn f<T: Send + Sync>() {}' 'type Shared = Arc<dyn Carrier + Send + Sync>;' 'impl<T: Sync> Wrapper<T> {}' \
    'trait Carrier: Send + std::marker::Sync {}' > "$room/crates/other/src/standard.rs"
  printf '%s\n' 'fn f(s: Sync) { match s { Sync::Folder(_) => {}, _ => {} } }' > "$room/crates/tisty-carrier/src/lib.rs"
  out=$(cd "$room" && modes_named_outside crates)
  rm -rf "$room"
  broke=$(printf '%s\n' "$out" | grep -cE 'crates/other/src/(lib|forms)\.rs')
  allowed=$(printf '%s\n' "$out" | grep -cE 'crates/(other/src/(checks|standard)|tisty-carrier/src/lib)\.rs')
  if [ "$broke" != 8 ] || [ "$allowed" != 0 ]; then
    amiss "the rule on the way of syncing no longer tells what breaks it from a test, the standard Sync or the factory"
  else
    went_well "the rule on the way of syncing fails on a match outside the factory"
  fi
}

cd "$(dirname "$0")/.." || exit 2
no_prose_blocks
nothing_past_what_a_person_holds
written_in_english
nothing_the_window_cannot_translate
nothing_the_core_prints
nothing_that_takes_the_window_down
nothing_new_reaches_the_command_line
every_spawn_pins_its_language
the_way_of_syncing_is_named_once
the_rule_on_the_way_of_syncing_can_fail
both_languages_carry_the_same_documents
exit $status
