#!/usr/bin/env python3
import io
import json
import os
import sys
import urllib.request

VERSION = os.environ["VERSION"]
AT = os.environ.get("LISTING", "server.json")
REPO = os.environ.get("REPO", "rgdevment/Tisty")
NPM = "https://registry.npmjs.org"

listing = json.load(io.open(AT, encoding="utf-8"))
listing["version"] = VERSION

if len(listing["description"]) > 100:
    sys.exit(f"the registry takes 100 characters, and this is {len(listing['description'])}")

# The launcher on npm has its own versions; the listing names the one people get today.
package = listing["packages"][0]
try:
    with urllib.request.urlopen(f"{NPM}/{package['identifier']}/latest", timeout=30) as answer:
        published = json.load(answer)
except OSError as failed:
    sys.exit(f"npm does not have {package['identifier']} yet: {failed}")
# The registry refuses a package whose mcpName is not this server's, but only after the version is spent.
if published.get("mcpName") != listing["name"]:
    sys.exit(f"{package['identifier']}@{published['version']} says mcpName {published.get('mcpName')!r}, not {listing['name']!r}")
package["version"] = published["version"]

# The file in the repository is a template on purpose; publishing it unstamped would list a
# release that does not exist, at a version nobody can ever correct.
if listing["version"] == "0.0.0" or package["version"] == "0.0.0":
    sys.exit("the listing was not stamped: it still carries the template's version")

# Pinned to the tag: a listed version is never revisited, so an icon on main could change under it.
for icon in listing.get("icons", []):
    if "/v0.0.0/" not in icon["src"]:
        sys.exit(f"the template's icon has to sit under /v0.0.0/ to be stamped, and it is {icon['src']}")
    icon["src"] = f"https://raw.githubusercontent.com/{REPO}/v{VERSION}/{icon['src'].split('/v0.0.0/', 1)[1]}"
    try:
        with urllib.request.urlopen(urllib.request.Request(icon["src"], method="HEAD"), timeout=30):
            pass
    except OSError as failed:
        sys.exit(f"the icon the listing names is not there: {icon['src']} ({failed})")

io.open(AT, "w", encoding="utf-8", newline="\n").write(json.dumps(listing, indent=2) + "\n")
print(json.dumps(listing, indent=2))
