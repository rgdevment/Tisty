#!/usr/bin/env bash
set -euo pipefail

window=${WINDOW:-window.json}
crates=${CRATES:-crates}
badge=${BADGE:-mutants.json}

shards=$(find . -name 'mutants.json' -path '*window*' | sort)
if [ -z "$shards" ]; then
  echo "::error::no window report to read, so any score would be made up"
  exit 1
fi

# shellcheck disable=SC2086
jq -s '.[0] * {
    files: (map(.files) | add),
    testFiles: (map(.testFiles // {}) | add)
  }' $shards > "$window"

read -r killed survived < <(
  jq -r '
    [.files[].mutants[].status] as $all
    | [($all | map(select(. == "Killed" or . == "Timeout")) | length),
       ($all | map(select(. == "Survived" or . == "NoCoverage")) | length)]
    | @tsv
  ' "$window"
)

caught=0
missed=0
for one in $(find "$crates" -name 'outcomes.json' | sort); do
  caught=$((caught + $(jq -r '.caught + .timeout' "$one")))
  missed=$((missed + $(jq -r '.missed' "$one")))
done

if [ "$caught" -eq 0 ]; then
  echo "::error::no crate outcome to read, so any score would be made up"
  exit 1
fi

live=$((killed + caught))
dead=$((survived + missed))
total=$((live + dead))
score=$(awk -v a="$live" -v b="$total" 'BEGIN { printf "%.1f", b ? a * 100 / b : 0 }')

colour=red
awk -v s="$score" 'BEGIN { exit !(s >= 60) }' && colour=orange
awk -v s="$score" 'BEGIN { exit !(s >= 80) }' && colour=green

jq -n --arg m "$score%" --arg c "$colour" \
  '{schemaVersion: 1, label: "mutants", message: $m, color: $c}' > "$badge"

{
  echo "### What the sweep came to"
  echo
  echo "| | caught | survived | score |"
  echo "| --- | --: | --: | --: |"
  echo "| crates | $caught | $missed | $(awk -v a="$caught" -v b="$((caught + missed))" 'BEGIN { printf "%.1f%%", b ? a * 100 / b : 0 }') |"
  echo "| window | $killed | $survived | $(awk -v a="$killed" -v b="$((killed + survived))" 'BEGIN { printf "%.1f%%", b ? a * 100 / b : 0 }') |"
  echo "| **all** | **$live** | **$dead** | **$score%** |"
} >> "${GITHUB_STEP_SUMMARY:-/dev/stdout}"

echo "$score"
