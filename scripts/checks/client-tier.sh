#!/usr/bin/env bash
# WHY THIS FILE EXISTS: foundation crates are consumed by lightweight Python
# wheels, CLI tools, and external clients. If any crate silently pulls in
# server-tier dependencies (sqlx outside the approved roots, axum, datafusion,
# cloud SDKs) the entire consumer pays the compile cost and risks linking
# server infrastructure into environments that should stay minimal.
#
# WHAT IT CHECKS (using cargo metadata, not text grep):
#   - wyrd-spec: no utoipa, pyo3, sqlx, axum, or opentelemetry at default features
#   - wyrd-auth-verify: no pyo3 or sqlx in the all-features dep tree
#   - wyrd-client: no sqlx, postgres, datafusion, axum, kube, cloud SDK,
#     or opendal in the all-features normal dep tree
#   - wyrd-utils (default, no python feature): PyO3 absent from the dep tree
set -euo pipefail

# WYRD_CHECK_ROOT: override the workspace root (cwd for cargo tree).
# Set by the negative self-test harness.
REPO_ROOT="${WYRD_CHECK_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
cd "$REPO_ROOT"

# Fixture overrides: each env var supplies pre-built cargo tree text for one
# check path, bypassing the real cargo tree call.  Used by the negative
# self-test harness to inject a forbidden-dep violation without requiring a
# compilable temp workspace.  Any variable left unset runs the real cargo tree.
#   WYRD_CLIENT_TIER_SPEC_TREE      -- wyrd-spec tree text
#   WYRD_CLIENT_TIER_AV_TREE        -- wyrd-auth-verify tree text
#   WYRD_CLIENT_TIER_CLIENT_TREE    -- wyrd-client tree text
#   WYRD_CLIENT_TIER_UTILS_TREE     -- wyrd-utils tree text

fail=0

# Helper: check cargo tree output for forbidden patterns and print any matches.
check_tree() {
    local label="$1"
    local pattern="$2"
    local tree_output="$3"
    local matches
    matches=$(echo "$tree_output" | grep -E "$pattern" || true)
    if [ -n "$matches" ]; then
        echo "FAIL client-tier [$label]: forbidden dep(s) found:"
        echo "$matches"
        fail=1
    fi
}

echo "client-tier: checking wyrd-spec (no-default-features)…"
if [ -n "${WYRD_CLIENT_TIER_SPEC_TREE:-}" ]; then
    spec_tree="$WYRD_CLIENT_TIER_SPEC_TREE"
else
    spec_tree=$(cargo tree -p wyrd-spec --no-default-features -e normal 2>/dev/null)
fi
check_tree "wyrd-spec" "(^|[ ─└├])(utoipa|pyo3|sqlx|axum|opentelemetry)" "$spec_tree"

echo "client-tier: checking wyrd-auth-verify (all-features)…"
if [ -n "${WYRD_CLIENT_TIER_AV_TREE:-}" ]; then
    av_tree="$WYRD_CLIENT_TIER_AV_TREE"
else
    av_tree=$(cargo tree -p wyrd-auth-verify --all-features -e normal 2>/dev/null)
fi
check_tree "wyrd-auth-verify" "(^|[ ─└├])(pyo3|sqlx)" "$av_tree"

echo "client-tier: checking wyrd-client (all-features)…"
if [ -n "${WYRD_CLIENT_TIER_CLIENT_TREE:-}" ]; then
    client_tree="$WYRD_CLIENT_TIER_CLIENT_TREE"
else
    client_tree=$(cargo tree -p wyrd-client --all-features -e normal 2>/dev/null)
fi
check_tree "wyrd-client" "(^|[ ─└├])(sqlx|tokio-postgres|postgres|deltalake|datafusion|axum|kube|aws-sdk|azure_|google-cloud|opendal|rdkafka|lapin|redis|deadpool-)" "$client_tree"

echo "client-tier: checking wyrd-utils (no python feature)…"
if [ -n "${WYRD_CLIENT_TIER_UTILS_TREE:-}" ]; then
    utils_tree="$WYRD_CLIENT_TIER_UTILS_TREE"
else
    utils_tree=$(cargo tree -p wyrd-utils --no-default-features -e normal 2>/dev/null)
fi
check_tree "wyrd-utils default" "(^|[ ─└├])pyo3" "$utils_tree"

if [ "$fail" -eq 0 ]; then
    echo "client-tier: all checks passed"
else
    exit 1
fi
