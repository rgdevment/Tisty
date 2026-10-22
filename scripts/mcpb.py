#!/usr/bin/env python3
import hashlib
import io
import json
import os
import stat
import sys
import tomllib
import zipfile

VERSION = os.environ["VERSION"]
DIST = os.environ.get("DIST", "dist")
LISTING = os.environ.get("LISTING", "server.json")
OUT = os.path.join(DIST, f"tisty-mcp-{VERSION}.mcpb")
REPO = os.environ.get("REPO", "rgdevment/Tisty")
SAID = json.load(io.open(LISTING, encoding="utf-8"))["description"]
LICENSE = tomllib.load(io.open("Cargo.toml", "rb"))["workspace"]["package"]["license"]
ICON = "assets/icons/tisty-512.png"
SCREENSHOTS = ["en-07-mcp.png", "en-01-record.png", "en-02-tasks.png", "en-03-notes.png"]

LONG = f"""Tisty is a free, open source desktop app for Windows and macOS for notes, documents and tasks, kept as plain Markdown on your own disk. No account, no subscription, no telemetry, no server.

This server comes with the app and runs on the same machine over stdio. Your assistant can file tasks with steps and dates, set reminders, write documents and correct them passage by passage, file them into folders, attach files, and search open and archived work alike. It cannot close or delete anything: it says a task is done and you confirm it.

**Requires the Tisty app.** Install it from the [Microsoft Store](https://apps.microsoft.com/detail/9PGVWXD8X93N), with `brew install --cask rgdevment/tap/tisty` on macOS, or with the installers on [GitHub Releases](https://github.com/{REPO}/releases/latest). Settings › Assistants connects yours and lists every assistant that speaks to Tisty as a device you can shut out at any time.

Long documents are read by outline and by section, so a whole document is never sent to a model for one paragraph.

**Learn more:** [documentation](https://www.tisty.app/docs/) · [assistants and MCP](https://www.tisty.app/docs/assistants-and-mcp/) · [Tisty Markdown guide](https://markdown.tisty.app) · [wiki](https://github.com/{REPO}/wiki)"""

MANIFEST = {
    "manifest_version": "0.3",
    "name": "tisty",
    "display_name": "Tisty",
    "version": VERSION,
    "description": SAID,
    "long_description": LONG,
    "author": {"name": REPO.split("/")[0], "url": f"https://github.com/{REPO.split('/')[0]}"},
    "homepage": "https://www.tisty.app/",
    "documentation": "https://www.tisty.app/docs/assistants-and-mcp/",
    "support": f"https://github.com/{REPO}/issues",
    "repository": {"type": "git", "url": f"https://github.com/{REPO}"},
    "icon": "icon.png",
    "screenshots": [f"screenshots/{named}" for named in SCREENSHOTS],
    "keywords": ["notes", "tasks", "documents", "markdown", "local-first", "productivity", "todo", "knowledge-base"],
    "license": LICENSE,
    "server": {
        "type": "binary",
        "entry_point": "server/tisty",
        "mcp_config": {
            # The reference host only substitutes ${...}; nothing there resolves a bare
            # relative path against the extension directory, so a plain one finds nothing.
            "command": "${__dirname}/server/tisty",
            "args": ["mcp"],
            "platform_overrides": {"win32": {"command": "${__dirname}/server/tisty.exe"}},
        },
    },
    "compatibility": {"platforms": ["darwin", "win32"]},
}


def taken(at, by):
    if not os.path.isfile(at):
        sys.exit(f"the bundle needs {at}, and {by} did not leave it")
    return io.open(at, "rb").read()


def main():
    staged = os.environ["STAGED"]
    mac = taken(os.path.join(staged, "server-macos-universal", "tisty"), "the job that builds the server")
    win = taken(os.path.join(staged, "server-windows-x86_64", "tisty.exe"), "the job that builds the server")

    os.makedirs(DIST, exist_ok=True)
    # The Unix bit does not survive a zip written on Windows, and a server nobody can
    # execute installs and then fails in the client with nothing to read.
    items = [
        ("manifest.json", json.dumps(MANIFEST, indent=2, ensure_ascii=False).encode() + b"\n", 0o644),
        ("icon.png", taken(ICON, "the checkout"), 0o644),
        *[(f"screenshots/{named}", taken(f"assets/store/{named}", "the checkout"), 0o644) for named in SCREENSHOTS],
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
