#!/usr/bin/env bash
# WHY THIS FILE EXISTS: SQLx links against libpq (or the pure-Rust driver) at
# compile time and pulls in async runtimes and TLS stacks. If any crate outside
# the two approved roots (wyrd-sql-core and wyrd-dev-fixtures) directly
# depends on SQLx, every consumer of that crate pays the compile and link cost
# even in environments that never touch a database. The check uses cargo
# metadata to walk the real dependency graph rather than grepping Cargo.toml
# names, so it catches transitive leaks too.
#
# WHAT IT CHECKS: the set of workspace-member crates that directly depend on
# sqlx (any kind: normal, dev, build) must be exactly {wyrd-sql-core,
# wyrd-dev-fixtures}. Any member outside that set is a violation.
set -euo pipefail

# WYRD_CHECK_ROOT: override the workspace root.  Set by the negative self-test
# harness to point at a temp fixture workspace with an injected violation.
REPO_ROOT="${WYRD_CHECK_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
cd "$REPO_ROOT"

# WYRD_METADATA_FIXTURE: absolute path to a pre-built cargo metadata JSON file.
# When set, reads metadata from this file instead of running `cargo metadata`.
# Used by the negative self-test harness to inject a sqlx-in-wrong-crate violation.
echo "sqlx-scope: parsing workspace metadata..."
if [ -n "${WYRD_METADATA_FIXTURE:-}" ]; then
    METADATA_FILE="$WYRD_METADATA_FIXTURE"
    _meta_tmp=false
else
    METADATA_FILE=$(mktemp)
    _meta_tmp=true
    cargo metadata --locked --all-features --format-version 1 > "$METADATA_FILE" 2>/dev/null
fi

VIOLATIONS=$(python3 - "$METADATA_FILE" << 'PYEOF'
import json, sys

ALLOWED = {"wyrd-sql-core", "wyrd-dev-fixtures"}

meta_path = sys.argv[1]
with open(meta_path) as f:
    md = json.load(f)

member_ids = set(md["workspace_members"])

violations = []
for pkg in md["packages"]:
    if pkg["id"] not in member_ids:
        continue
    for dep in pkg["dependencies"]:
        if dep["name"] == "sqlx" and pkg["name"] not in ALLOWED:
            violations.append(
                "  {} directly depends on sqlx (kind: {})".format(pkg["name"], dep["kind"])
            )

if violations:
    print("\n".join(violations))
PYEOF
)
[ "$_meta_tmp" = "true" ] && rm -f "$METADATA_FILE"

if [ -n "$VIOLATIONS" ]; then
    echo "FAIL sqlx-scope: sqlx dependency found outside approved roots (wyrd-sql-core, wyrd-dev-fixtures):"
    echo "$VIOLATIONS"
    exit 1
fi

echo "sqlx-scope: passed -- sqlx is confined to wyrd-sql-core and wyrd-dev-fixtures"
