#!/usr/bin/env bash
# WHY THIS FILE EXISTS: Wyrd uses a single TLS provider (AWS-LC via rustls).
# If a dependency introduces the ring provider -- even transitively -- the
# production binary ends up with two competing crypto implementations at link
# time. On some platforms this causes a linker error; on others it silently
# picks one provider, making TLS behavior non-deterministic. The check also
# verifies the workspace has completed the reqwest 0.12 to 0.13 migration so
# no crate pins the old bundled-TLS variant.
#
# WHAT IT CHECKS:
#   - The all-features production dep graph does NOT enable rustls feature "ring".
#   - The all-features production dep graph DOES enable rustls feature "aws_lc_rs".
#   - No workspace member directly depends on reqwest 0.12.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

# WYRD_RUSTLS_GRAPH_FIXTURE: absolute path to a pre-built cargo-tree output
# file. When set, skips running cargo tree and uses the fixture content
# directly. Set by the negative self-test harness to inject a violation.
# WYRD_RUSTLS_META_FIXTURE: same for cargo metadata --no-deps output.
echo "rustls-provider: building production dependency graph..."
if [ -n "${WYRD_RUSTLS_GRAPH_FIXTURE:-}" ]; then
    GRAPH_FILE="$WYRD_RUSTLS_GRAPH_FIXTURE"
    _graph_tmp=false
else
    GRAPH_FILE=$(mktemp)
    _graph_tmp=true
    cargo tree --workspace --all-features -e features,no-dev > "$GRAPH_FILE" 2>/dev/null
fi

if grep -q 'rustls feature "ring"' "$GRAPH_FILE"; then
    echo "FAIL rustls-provider: production graph enables a Ring-backed Rustls provider"
    grep 'rustls feature "ring"' "$GRAPH_FILE" || true
    [ "$_graph_tmp" = "true" ] && rm -f "$GRAPH_FILE"
    exit 1
fi

if ! grep -qE 'rustls feature "aws[-_]lc[-_]rs"' "$GRAPH_FILE"; then
    echo "FAIL rustls-provider: production graph does not enable the AWS-LC Rustls provider"
    [ "$_graph_tmp" = "true" ] && rm -f "$GRAPH_FILE"
    exit 1
fi
[ "$_graph_tmp" = "true" ] && rm -f "$GRAPH_FILE"

echo "rustls-provider: checking for reqwest 0.12 direct dependencies..."
if [ -n "${WYRD_RUSTLS_META_FIXTURE:-}" ]; then
    METADATA_FILE="$WYRD_RUSTLS_META_FIXTURE"
    _meta_tmp=false
else
    METADATA_FILE=$(mktemp)
    _meta_tmp=true
    cargo metadata --format-version=1 --no-deps > "$METADATA_FILE" 2>/dev/null
fi
REQWEST12_CRATES=$(python3 - "$METADATA_FILE" << 'PYEOF'
import json, sys

meta_path = sys.argv[1]
with open(meta_path) as f:
    md = json.load(f)

offenders = [
    p["name"]
    for p in md["packages"]
    for d in p.get("dependencies", [])
    if d["name"] == "reqwest" and d["req"].startswith("^0.12")
]
if offenders:
    print("  Offending crates: " + ", ".join(offenders))
    sys.exit(1)
PYEOF
)
RET=$?
[ "$_meta_tmp" = "true" ] && rm -f "$METADATA_FILE"
if [ "$RET" -ne 0 ]; then
    echo "FAIL rustls-provider: a workspace package directly depends on reqwest 0.12"
    echo "$REQWEST12_CRATES"
    exit 1
fi

echo "rustls-provider: passed -- AWS-LC is the only production Rustls provider and workspace reqwest is 0.13"
