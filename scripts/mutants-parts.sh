#!/usr/bin/env bash
set -euo pipefail

most_crates=8
most_window=12
lines_one_window_part_takes=8
machinery='^(\.cargo/mutants\.toml|app/stryker\.config\.json|app/vite\.mutants\.config\.ts|\.github/workflows/mutants\.yml|scripts/mutants-parts\.sh)$'

widest_window_ranges_first() {
  (cd app && git diff --unified=0 --relative --diff-filter=d "$1" -- src) \
    | awk '
        /^\+\+\+ b\// { f = substr($2, 3); next }
        /^@@/ {
          if (f !~ /\.tsx?$/) next
          if (f ~ /^src\/tests\//) next
          if (f ~ /^src\/(marks|glyphs|locales|synonyms)\.ts$/) next
          if (f ~ /^src\/main\.tsx$/) next
          if (f ~ /\.d\.ts$/) next
          split($3, a, ",")
          start = substr(a[1], 2) + 0
          len = (a[2] == "" ? 1 : a[2] + 0)
          if (len > 0) print len, f ":" start "-" start + len - 1
        }' \
    | sort -rn
}

window_share() {
  widest_window_ranges_first "$1" \
    | awk -v part="$2" -v many="$3" '
        { range[NR] = $2; wide[NR] = $1; count = NR }
        END {
          for (i = 1; i <= count; i++) {
            least = 1
            for (s = 2; s <= many; s++) if (load[s] < load[least]) least = s
            load[least] += wide[i]
            if (least == part) print range[i]
          }
        }' \
    | paste -sd, -
}

# Every part pays two minutes of unmutated build and tests before its first mutant, so a small diff takes few.
crates_parts() {
  local written
  if git diff --quiet "$1" -- crates; then
    echo 0
    return
  fi
  written=$(git diff --numstat --diff-filter=d "$1" -- 'crates/*.rs' ':(exclude)crates/*/tests/*' \
    | awk '{ lines += $1 } END { print lines + 0 }')
  if [ "$written" -le 20 ]; then
    echo 1
  elif [ "$written" -le 80 ]; then
    echo 2
  elif [ "$written" -le 250 ]; then
    echo 4
  else
    echo "$most_crates"
  fi
}

window_parts() {
  local summed ranges lines wanted
  summed=$(widest_window_ranges_first "$1" | awk '{ lines += $1 } END { print NR, lines + 0 }')
  ranges=${summed% *}
  lines=${summed#* }
  wanted=$(((lines + lines_one_window_part_takes - 1) / lines_one_window_part_takes))
  [ "$wanted" -le "$ranges" ] || wanted=$ranges
  [ "$wanted" -le "$most_window" ] || wanted=$most_window
  echo "$wanted"
}

listed() {
  local i out=""
  for ((i = 1; i <= $1; i++)); do out+="${out:+,}$i"; done
  printf '[%s]' "$out"
}

sizes() {
  local changed crates window packages=false
  changed=$(git diff --name-only --diff-filter=d "$1")
  crates=$(crates_parts "$1")
  window=$(window_parts "$1")
  if grep -qE '^packages/[^/]+/src/.*\.tsx?$' <<< "$changed"; then packages=true; fi
  # A change to what runs the mutants starts one job of each kind, or it would merge without ever having run.
  if grep -qE "$machinery" <<< "$changed"; then
    [ "$crates" -gt 0 ] || crates=1
    [ "$window" -gt 0 ] || window=1
    packages=true
  fi
  printf 'crates=%s\ncrates_of=%s\nwindow=%s\nwindow_of=%s\npackages=%s\n' \
    "$(listed "$crates")" "$crates" "$(listed "$window")" "$window" "$packages"
}

case ${1:-} in
  sizes)
    [ $# -eq 2 ] || { echo "usage: mutants-parts.sh sizes <from>" >&2; exit 2; }
    sizes "$2"
    ;;
  window)
    [ $# -eq 4 ] || { echo "usage: mutants-parts.sh window <from> <part> <of>" >&2; exit 2; }
    window_share "$2" "$3" "$4"
    ;;
  *)
    echo "usage: mutants-parts.sh sizes <from> | window <from> <part> <of>" >&2
    exit 2
    ;;
esac
