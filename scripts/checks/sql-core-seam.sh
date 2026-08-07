#!/usr/bin/env bash
# WHY THIS FILE EXISTS: `wyrd-sql-core` ships a stable plane-neutral API seam
# (DSN/pool/operator/tenant/error/migration). If a public symbol is removed or
# re-namespaced, existing consumers (wyrd-sql, vala-sql, test harnesses) break
# at link time rather than at a visible compile step. This check compiles the
# dedicated seam target (`crates/wyrd-dev-fixtures/tests/sql_core_seam.rs`)
# which imports the *complete* T2 public export set; a compilation failure
# means API loss.
#
# The seam target has no `pg` feature gate, so it exercises the pure-Rust type
# surface without requiring Docker or a live database.
#
# WHAT IT CHECKS: compiling `wyrd-dev-fixtures --test sql_core_seam` (default
# features) succeeds, proving every wyrd-sql-core re-export is intact.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

# WYRD_SEAM_TEST: name of the test file (without .rs) under
# crates/wyrd-dev-fixtures/tests/ to compile and run.  Defaults to the
# production seam target sql_core_seam.  Set by the negative self-test
# harness to point at a temp broken seam file (placed in the same directory)
# that references a non-existent symbol, proving the script exits nonzero
# on a compilation failure rather than passing silently.
SEAM_TEST="${WYRD_SEAM_TEST:-sql_core_seam}"

echo "sql-core-seam: compiling seam target (no pg feature required)…"
if cargo test --locked -p wyrd-dev-fixtures --test "$SEAM_TEST" -- --nocapture 2>&1; then
    echo "sql-core-seam: seam compile and test passed"
else
    echo "FAIL sql-core-seam: seam target failed to compile or test failed"
    echo "  This means wyrd-sql-core lost or re-namespaced a public export."
    echo "  Locate the removed symbol in crates/wyrd-dev-fixtures/tests/${SEAM_TEST}.rs"
    echo "  and restore or update the public API in wyrd-sql-core."
    exit 1
fi
