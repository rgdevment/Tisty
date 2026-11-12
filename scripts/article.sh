#!/usr/bin/env bash
set -uo pipefail

cd "$(dirname "$0")/.." || exit 2

version=${1:?a version is needed, without the leading v}
folder=docs/news/$version

amiss() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    printf '::error::%s\n' "$1"
  else
    printf 'x  %s\n' "$1"
  fi
}

suggest() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    printf '::warning::%s\n' "$1"
  else
    printf '!  %s\n' "$1"
  fi
}

case $version in
  *-*)
    printf 'ok %s is a candidate, and tisty.app tells no news for one\n' "$version"
    exit 0
    ;;
esac

if [ "${version##*.}" != 0 ]; then
  printf 'ok %s is a patch: the window tells its news, and the article belongs to %s\n' "$version" "${version%.*}.0"
  exit 0
fi

if ! command -v python3 > /dev/null; then
  amiss "python3 is not here, so nobody looked at the article for $version in $folder"
  [ -n "${GITHUB_ACTIONS:-}" ] && exit 1
  exit 0
fi

reads='
import os
import re
import sys

folder = sys.argv[1]
MIN_WORDS = 150
MIN_SECTIONS = 2
PICTURES = (".png", ".jpg", ".jpeg", ".webp", ".gif")
SPANISH = {"el", "la", "los", "las", "que", "de", "del", "y", "en", "para", "con", "una", "un", "tu", "tus", "es", "se", "por", "lo", "al", "sin", "más"}
ENGLISH = {"the", "and", "of", "to", "your", "is", "with", "for", "you", "it", "in", "on", "that", "its", "without", "more"}
PICTURE = re.compile(r"!\[([^\]]*)\]\(([^)\s]+)")
NAMES = {"en": "English", "es": "Spanish"}

said = []


def look(tongue):
    path = f"{folder}/{tongue}.md"
    if not os.path.isfile(path):
        said.append(("fail", f"{path} is not there: tisty.app tells each minor version as an article, in Spanish and in English, before its tag goes out"))
        return None
    text = open(path, encoding="utf-8").read().lstrip("﻿")
    lines = [one.rstrip() for one in text.splitlines()]
    first = next((one for one in lines if one.strip()), "")
    if not re.match(r"^# \S", first):
        said.append(("fail", f"{path} opens with its headline as «# Headline», which is the title the site shows"))
    sections = [one for one in lines if one.startswith("## ")]
    if len(sections) < MIN_SECTIONS:
        said.append(("fail", f"{path} has {len(sections)} «## » sections and an article needs at least {MIN_SECTIONS}: one for each thing the version brings"))
    prose = " ".join(one for one in lines if one.strip() and not one.startswith("#"))
    words = [one.lower() for one in re.findall(r"[^\W\d_]+", PICTURE.sub(" ", prose))]
    spanish = sum(one in SPANISH for one in words)
    english = sum(one in ENGLISH for one in words)
    own, other = (spanish, english) if tongue == "es" else (english, spanish)
    if len(words) < MIN_WORDS:
        said.append(("fail", f"{path} says {len(words)} words and an article needs at least {MIN_WORDS}: tell what changes for a person and why it matters, not a list of changes"))
    elif own <= 2 * other:
        said.append(("fail", f"{path} does not read as {NAMES[tongue]}: each language has its own file, written in that language"))
    pictures = PICTURE.findall(text)
    for alt, source in pictures:
        target = os.path.join(folder, source)
        if re.search(r"[/\\:]", source):
            said.append(("fail", f"{path} shows {source} from elsewhere: copy the picture into {folder} and name it alone, so moving or recapturing a screen never changes the news"))
        elif not os.path.isfile(target):
            said.append(("fail", f"{path} shows {source}, and there is no such file"))
        elif not target.lower().endswith(PICTURES):
            said.append(("fail", f"{path} shows {source}, which is not a picture the site can size"))
        if not alt.strip():
            said.append(("fail", f"{path} shows {source} without saying what it shows, which is all someone who cannot see it gets"))
    return {"sections": len(sections), "pictures": len(pictures)}


english, spanish = look("en"), look("es")
if english and spanish:
    sections = (english["sections"], spanish["sections"])
    pictures = (english["pictures"], spanish["pictures"])
    if sections[0] != sections[1]:
        said.append(("fail", f"the English article has {sections[0]} sections and the Spanish {sections[1]}: the two tell the same news"))
    if pictures[0] != pictures[1]:
        said.append(("fail", f"the English article shows {pictures[0]} pictures and the Spanish {pictures[1]}: each shows the same screens in its own language"))
    if not any(pictures):
        said.append(("warn", f"the article in {folder} shows no picture: a screen of what is new is what a reader and a news site pick up first"))

for kind, text in said:
    print(f"{kind}\t{text}")
'

found=$(python3 -c "$reads" "$folder" 2>&1)
if [ $? != 0 ]; then
  amiss "${found:-the article in $folder could not be looked through}"
  exit 1
fi

status=0
while IFS=$'\t' read -r kind text; do
  case $kind in
    fail)
      amiss "$text"
      status=1
      ;;
    warn) suggest "$text" ;;
  esac
done <<< "$found"

[ "$status" = 0 ] && printf 'ok %s has its article in %s, in both languages\n' "$version" "$folder"
exit $status
