#!/usr/bin/env bash
# WHY THIS FILE EXISTS: The foundation workspace may only contain the 21
# decided crates (D1). Any path or workspace dependency pointing outside that
# set smuggles plane-specific, server-side, or excluded code into the foundation
# build graph. This check uses cargo metadata -- not name-prefix greps -- to
# inspect the real dependency graph, so renames or path changes cannot mask
# violations.
#
# WHAT IT CHECKS (three independent gates):
#   1. No path dependency whose canonical path falls outside the crates/ members.
#   2. wyrd-testing does not appear in normal, dev, or build dependency sets.
#   3. No cloud-SDK crate (aws-sdk-*, azure_*, google-cloud-*, kube) anywhere
#      in the workspace manifest dependencies.
set -euo pipefail

# WYRD_CHECK_ROOT: override the workspace root used for metadata and file scans.
# Set by the negative self-test harness to point at a temp fixture workspace.
# Defaults to the repository root derived from this script's location.
REPO_ROOT="${WYRD_CHECK_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
cd "$REPO_ROOT"

# WYRD_METADATA_FIXTURE: absolute path to a pre-built cargo metadata JSON file.
# When set, the script reads metadata from this file instead of running
# `cargo metadata`. Used by the negative self-test harness to inject violations
# without requiring the fixture to be a real compilable workspace.
echo "no-plane-deps: parsing workspace metadata..."
if [ -n "${WYRD_METADATA_FIXTURE:-}" ]; then
    METADATA_FILE="$WYRD_METADATA_FIXTURE"
    _meta_tmp=false
else
    METADATA_FILE=$(mktemp)
    _meta_tmp=true
    cargo metadata --locked --all-features --format-version 1 > "$METADATA_FILE" 2>/dev/null
fi

# ── Gate 1: no path dep outside the 21-crate member set ─────────────────────
echo "no-plane-deps: gate 1 -- checking path dependencies..."
VIOLATIONS=$(python3 - "$METADATA_FILE" << 'PYEOF'
import json, sys, os

meta_path = sys.argv[1]
with open(meta_path) as f:
    md = json.load(f)

workspace_root = md["workspace_root"]
member_ids = set(md["workspace_members"])

# crates/ canonical prefix
crates_prefix = os.path.join(workspace_root, "crates") + os.sep

violations = []
for pkg in md["packages"]:
    if pkg["id"] not in member_ids:
        continue
    for dep in pkg["dependencies"]:
        if dep.get("path") is not None:
            dep_path = dep["path"]
            # Resolve relative paths against the package manifest directory
            if not os.path.isabs(dep_path):
                pkg_dir = os.path.dirname(pkg["manifest_path"])
                dep_path = os.path.normpath(os.path.join(pkg_dir, dep_path))
            if not dep_path.startswith(crates_prefix):
                violations.append(
                    "  {} -> {} (path: {})".format(pkg["name"], dep["name"], dep_path)
                )

if violations:
    print("\n".join(violations))
PYEOF
)

if [ -n "$VIOLATIONS" ]; then
    [ "$_meta_tmp" = "true" ] && rm -f "$METADATA_FILE"
    echo "FAIL no-plane-deps gate 1: path dep(s) outside crates/ member set:"
    echo "$VIOLATIONS"
    exit 1
fi
echo "  gate 1 passed"

# ── Gate 2: wyrd-testing not in any dep kind ─────────────────────────────────
echo "no-plane-deps: gate 2 -- checking for wyrd-testing..."
WT_VIOLATIONS=$(python3 - "$METADATA_FILE" << 'PYEOF'
import json, sys

meta_path = sys.argv[1]
with open(meta_path) as f:
    md = json.load(f)

member_ids = set(md["workspace_members"])

violations = []
for pkg in md["packages"]:
    if pkg["id"] not in member_ids:
        continue
    for dep in pkg["dependencies"]:
        if dep["name"] == "wyrd-testing":
            violations.append(
                "  {} depends on wyrd-testing (kind: {})".format(pkg["name"], dep["kind"])
            )

if violations:
    print("\n".join(violations))
PYEOF
)

if [ -n "$WT_VIOLATIONS" ]; then
    [ "$_meta_tmp" = "true" ] && rm -f "$METADATA_FILE"
    echo "FAIL no-plane-deps gate 2: wyrd-testing found in workspace deps:"
    echo "$WT_VIOLATIONS"
    exit 1
fi
echo "  gate 2 passed"

# ── Gate 3: no cloud SDK ──────────────────────────────────────────────────────
echo "no-plane-deps: gate 3 -- checking for cloud SDK deps..."
CLOUD_VIOLATIONS=$(python3 - "$METADATA_FILE" << 'PYEOF'
import json, sys, re

CLOUD_PAT = re.compile(r'^(aws-sdk-|azure_|google-cloud-|kube$|kube-)')

meta_path = sys.argv[1]
with open(meta_path) as f:
    md = json.load(f)

member_ids = set(md["workspace_members"])

violations = []
for pkg in md["packages"]:
    if pkg["id"] not in member_ids:
        continue
    for dep in pkg["dependencies"]:
        if CLOUD_PAT.match(dep["name"]):
            violations.append(
                "  {} -> {} (kind: {})".format(pkg["name"], dep["name"], dep["kind"])
            )

if violations:
    print("\n".join(violations))
PYEOF
)
[ "$_meta_tmp" = "true" ] && rm -f "$METADATA_FILE"

if [ -n "$CLOUD_VIOLATIONS" ]; then
    echo "FAIL no-plane-deps gate 3: cloud SDK dep(s) found:"
    echo "$CLOUD_VIOLATIONS"
    exit 1
fi
echo "  gate 3 passed"

echo "no-plane-deps: all gates passed"
