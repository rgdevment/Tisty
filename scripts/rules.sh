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
  find crates/*/src app/src app/src-tauri/src packages/*/src \( -name '*.rs' -o -name '*.ts' -o -name '*.tsx' \) \
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
    crates/*/src app/src app/src-tauri/src packages/*/src --include='*.rs' --include='*.ts' --include='*.tsx'
  looked_through \
    "an identifier is in Spanish; what a person reads lives in locales/" $? \
    "the source could not be looked through for Spanish identifiers" \
    || return
  grep -rnE "\b$spanish *:" \
    crates/*/src app/src app/src-tauri/src packages/*/src --include='*.rs' --include='*.ts' --include='*.tsx' \
    | grep -v '"'
  looked_through \
    "a field is named in Spanish; what a person reads lives in locales/" $? \
    "the source could not be looked through for Spanish field names" \
    || return
  grep -rnE '^\s*//.*\b(que|porque|pero|aunque|cuando|donde|seg[uú]n|tambi[eé]n|aqu[ií]|ah[ií]|entonces)\b' \
    crates/*/src app/src app/src-tauri/src packages/*/src --include='*.rs' --include='*.ts' --include='*.tsx'
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

# Each module a file names, after `from`, a bare `import` or `import(`, as: file line module.
named_modules() {
  grep -rnoE "(from|import)[[:space:]]*\(?[[:space:]]*['\"][^'\"]+['\"]" "$@" \
    --include='*.ts' --include='*.tsx' \
    | sed -E "s/^([^:]+):([0-9]+):.*['\"]([^'\"]+)['\"]$/\1 \2 \3/"
}

# A package is meant for other apps too, so it reaches nothing of Tisty's and is entered only by its doors.
keeps_to_itself() {
  local name=$1 said=$2 doors=$3 root=packages/$1/src named out
  named=$(named_modules "$root")
  [ $? -lt 2 ] && [ -n "$named" ] \
    || { amiss "$said could not be looked through for what it reaches"; return; }
  # A relative path may climb only as far as the folders the file sits in below src/.
  out=$(printf '%s\n' "$named" | awk -v root="$root/" '
    $3 ~ /^@tauri-apps\// { print; next }
    $3 ~ /^\.\.\// {
      folders = substr($1, length(root) + 1)
      depth = gsub("/", "/", folders)
      spec = $3
      ups = 0
      while (substr(spec, 1, 3) == "../") { ups++; spec = substr(spec, 4) }
      if (ups > depth) print
    }')
  if [ -n "$out" ]; then
    printf '%s\n' "$out"
    amiss "$said reaches into the app; it takes what it needs from whoever holds it"
    return
  fi
  named=$(named_modules app/src packages/*/src)
  [ $? -lt 2 ] || { amiss "the app could not be looked through for how it enters $said"; return; }
  out=$(printf '%s\n' "$named" | awk -v name="$name" -v doors=" $doors " -v root="$root/" '
    index($1, root) != 1 && ($3 == name || index($3, name "/") == 1) && index(doors, " " $3 " ") == 0')
  if [ -n "$out" ]; then
    printf '%s\n' "$out"
    amiss "$said is entered other than by $doors"
    return
  fi
  went_well "$said and whoever uses it meet only at its doors"
}

the_packages_keep_to_themselves() {
  keeps_to_itself tisty-diagrams "the diagram studio" "tisty-diagrams/read tisty-diagrams/studio"
  keeps_to_itself tisty-reader "the page reader" "tisty-reader/page"
}

no_token_kept_in_the_tree() {
  local token='ya29\.[A-Za-z0-9_-]{20,}|1//0[A-Za-z0-9_-]{20,}|GOCSPX-[A-Za-z0-9_-]{20,}|sl\.[A-Za-z0-9_-]{40,}|eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}'
  grep -rnIE "$token" crates app/src app/src-tauri/src packages scripts .github docs     --exclude='*.lock' --exclude='telling.rs' --exclude-dir=target --exclude-dir=node_modules
  looked_through     "a provider's token or secret is in the tree; fakes make their own, and real ones live in the keychain" $?     "the tree could not be looked through for tokens"     && went_well "no provider token or secret is kept in the tree"
}

no_worker_secrets_kept() {
  local kept
  kept=$(git ls-files -- 'crates/tisty-share-worker/.dev.vars*' 'crates/tisty-share-worker/.env*')
  if [ -n "$kept" ]; then
    printf '%s\n' "$kept"
    amiss "a file wrangler reads secrets from is committed; it stays on the machine and the secret goes up with wrangler secret put"
  else
    went_well "no Worker secrets file is committed"
  fi
}

# A rollback puts back code that reads the tables as they were, so a migration may only add to them.
migrations_only_add() {
  local found
  found=$(grep -niE '\b(drop|rename)\b' crates/tisty-share-worker/migrations/*.sql)
  if [ -n "$found" ]; then
    printf '%s\n' "$found"
    amiss "a migration takes something away; the version a rollback puts back would no longer read D1"
  else
    went_well "the Worker's migrations only add to the tables"
  fi
}

cd "$(dirname "$0")/.." || exit 2
no_prose_blocks
the_packages_keep_to_themselves
nothing_past_what_a_person_holds
written_in_english
nothing_the_window_cannot_translate
nothing_the_core_prints
nothing_that_takes_the_window_down
every_spawn_pins_its_language
the_way_of_syncing_is_named_once
the_rule_on_the_way_of_syncing_can_fail
both_languages_carry_the_same_documents
no_token_kept_in_the_tree
no_worker_secrets_kept
migrations_only_add
exit $status
