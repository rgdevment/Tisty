#!/usr/bin/env python3
import hashlib
import io
import json
import os
import sys

VERSION = os.environ["VERSION"]
BUNDLE = os.environ["BUNDLE"]
AT = os.environ.get("LISTING", "server.json")
REPO = os.environ.get("REPO", "rgdevment/Tisty")
URL = f"https://github.com/{REPO}/releases/download/v{VERSION}/tisty-mcp-{VERSION}.mcpb"

listing = json.load(io.open(AT, encoding="utf-8"))
listing["version"] = VERSION
listing["packages"][0]["identifier"] = URL
listing["packages"][0]["fileSha256"] = hashlib.sha256(io.open(BUNDLE, "rb").read()).hexdigest()

if len(listing["description"]) > 100:
    sys.exit(f"the registry takes 100 characters, and this is {len(listing['description'])}")

# The file in the repository is a template on purpose; publishing it unstamped would list a
# release that does not exist, at a version nobody can ever correct.
if listing["version"] == "0.0.0" or set(listing["packages"][0]["fileSha256"]) == {"0"}:
    sys.exit("the listing was not stamped: it still carries the template's version or hash")

io.open(AT, "w", encoding="utf-8", newline="\n").write(json.dumps(listing, indent=2) + "\n")
print(json.dumps(listing, indent=2))
