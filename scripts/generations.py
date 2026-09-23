#!/usr/bin/env python3
import json
import re
import subprocess
import sys

REF = "refs/heads/main"
# A rust-cache key ends in the hash of the lockfiles, and a setup-node one in the hash of
# the lockfile itself. Everything before that names the same cache across versions.
FAMILY = [
    re.compile(r"^(.*)-[0-9a-f]{8}$"),
    re.compile(r"^(node-cache-.*-npm)-[0-9a-f]{64}$"),
]


def gh(*args):
    done = subprocess.run(["gh", *args], capture_output=True, text=True)
    if done.returncode != 0:
        sys.exit(f"gh {' '.join(args)} failed: {done.stderr.strip()}")
    return done.stdout


def family(key):
    for shape in FAMILY:
        found = shape.match(key)
        if found:
            return found.group(1)
    return key


def main():
    kept = json.loads(
        gh("cache", "list", "--ref", REF, "--limit", "100", "--json", "key,sizeInBytes,createdAt")
    )
    if not kept:
        print("main holds no caches")
        return

    households = {}
    for one in kept:
        households.setdefault(family(one["key"]), []).append(one)

    stale = []
    for household in households.values():
        household.sort(key=lambda one: one["createdAt"], reverse=True)
        stale.extend(household[1:])

    if not stale:
        print(f"{len(kept)} cache(s), one generation each: nothing to sweep")
        return

    freed = 0
    for one in stale:
        print(f"  {one['sizeInBytes'] // 1048576:>5} MB  {one['key']}")
        done = subprocess.run(
            ["gh", "cache", "delete", one["key"], "--ref", REF],
            capture_output=True,
            text=True,
        )
        if done.returncode == 0:
            freed += one["sizeInBytes"]
        else:
            print(f"         gone already: {one['key']}")

    print(f"::notice::swept {len(stale)} superseded cache(s), {freed // 1048576} MB")


if __name__ == "__main__":
    main()
