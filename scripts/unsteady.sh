#!/usr/bin/env bash
set -euo pipefail

turns=${TURNS:-turns}
title=${TITLE:-unsteady tests}

failed=$(
  for report in "$turns"/*.xml; do
    [ -e "$report" ] || continue
    awk '
      /<testcase / {
        name = ""; class = ""
        if (match($0, / name="[^"]*"/)) name = substr($0, RSTART + 7, RLENGTH - 8)
        if (match($0, / classname="[^"]*"/)) class = substr($0, RSTART + 12, RLENGTH - 13)
      }
      /<failure|<error/ { if (name != "") print class " " name; name = "" }
    ' "$report"
  done | sort | uniq -c | sort -rn
)
ran=$(find "$turns" -maxdepth 1 -name '*.xml' | wc -l | tr -d ' ')

{
  echo "### $title"
  echo
  if [ -z "$failed" ]; then
    echo "Every test passed in all $ran turns."
  else
    echo "| turns failed of $ran | crate | test |"
    echo "| ---: | --- | --- |"
    echo "$failed" | awk '{ print "| " $1 " | " $2 " | `" $3 "` |" }'
  fi
} >>"${GITHUB_STEP_SUMMARY:-/dev/stdout}"

[ -z "$failed" ]
