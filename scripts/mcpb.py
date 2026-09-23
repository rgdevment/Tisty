#!/usr/bin/env python3
import hashlib
import io
import json
import os
import stat
import sys
import zipfile

VERSION = os.environ["VERSION"]
DIST = os.environ.get("DIST", "dist")
OUT = os.path.join(DIST, f"tisty-mcp-{VERSION}.mcpb")

MANIFEST = {
    "manifest_version": "0.3",
    "name": "tisty",
    "display_name": "Tisty",
    "version": VERSION,
    "description": "Notes, documents and tasks as plain Markdown on your own disk.",
    "author": {"name": "rgdevment", "url": "https://github.com/rgdevment"},
    "homepage": "https://github.com/rgdevment/Tisty",
    "server": {
        "type": "binary",
        "entry_point": "server/tisty",
        "mcp_config": {
            "command": "server/tisty",
            "args": ["mcp"],
            "platform_overrides": {"win32": {"command": "server/tisty.exe"}},
        },
    },
    "compatibility": {"platforms": ["darwin", "win32"]},
}


def taken(where, named):
    at = os.path.join(where, named)
    if not os.path.isfile(at):
        sys.exit(f"the bundle needs {at}, and the command line job did not leave it")
    return io.open(at, "rb").read()


def main():
    staged = os.environ["STAGED"]
    mac = taken(os.path.join(staged, f"tisty-cli-{VERSION}-macos-universal"), "tisty")
    win = taken(os.path.join(staged, f"tisty-cli-{VERSION}-windows-x86_64"), "tisty.exe")

    os.makedirs(DIST, exist_ok=True)
    # The Unix bit does not survive a zip written on Windows, and a server nobody can
    # execute installs and then fails in the client with nothing to read.
    items = [
        ("manifest.json", json.dumps(MANIFEST, indent=2).encode() + b"\n", 0o644),
        ("server/tisty", mac, 0o755),
        ("server/tisty.exe", win, 0o644),
    ]
    with zipfile.ZipFile(OUT, "w", zipfile.ZIP_DEFLATED) as bundle:
        for named, body, mode in items:
            entry = zipfile.ZipInfo(named, date_time=(2026, 1, 1, 0, 0, 0))
            entry.external_attr = (stat.S_IFREG | mode) << 16
            entry.compress_type = zipfile.ZIP_DEFLATED
            bundle.writestr(entry, body)

    print(f"{OUT} ({os.path.getsize(OUT) / 1048576:.1f} MB)")
    print(hashlib.sha256(io.open(OUT, "rb").read()).hexdigest())


if __name__ == "__main__":
    main()
