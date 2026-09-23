#!/usr/bin/env python3
import json
import os
import re
import subprocess
import sys
from datetime import datetime

REF = "refs/heads/main"
DRY = os.environ.get("DRY_RUN", "").lower() == "true"
# Only rust: a setup-node key carries the hash of its lockfile and nothing else, so two live
# caches for one platform look alike and the sweep would take one of them for a leftover.
RUST = re.compile(r"^(v0-rust-.+)-[0-9a-f]{8}-[0-9a-f]{8}$")
GONE = "Could not find a cache matching"


def gh(*args):
    done = subprocess.run(["gh", *args], capture_output=True, text=True)
    if done.returncode != 0:
        sys.exit(f"gh {' '.join(args)} failed: {done.stderr.strip()}")
    return done.stdout


def main():
    kept = json.loads(
        gh("cache", "list", "--ref", REF, "--limit", "100", "--json", "key,sizeInBytes,createdAt")
    )

    households = {}
    for one in kept:
        found = RUST.match(one["key"])
        if found:
            households.setdefault(found.group(1), []).append(one)

    stale = []
    for household in households.values():
        household.sort(key=lambda one: datetime.fromisoformat(one["createdAt"]), reverse=True)
        stale.extend(household[1:])

    if not stale:
        print(f"{len(kept)} cache(s) on main, one generation each: nothing to sweep")
        return

    freed = 0
    failed = []
    for one in stale:
        print(f"  {one['sizeInBytes'] // 1048576:>5} MB  {one['key']}")
        if DRY:
            freed += one["sizeInBytes"]
            continue
        done = subprocess.run(
            ["gh", "cache", "delete", one["key"], "--ref", REF],
            capture_output=True,
            text=True,
        )
        if done.returncode == 0:
            freed += one["sizeInBytes"]
        elif GONE in done.stderr:
            print(f"         evicted before we got to it: {one['key']}")
        else:
            failed.append(f"{one['key']}: {done.stderr.strip()}")

    said = "would sweep" if DRY else "swept"
    print(f"::notice::{said} {len(stale) - len(failed)} superseded cache(s), {freed // 1048576} MB")
    if failed:
        sys.exit("::error::" + "; ".join(failed))


if __name__ == "__main__":
    main()
