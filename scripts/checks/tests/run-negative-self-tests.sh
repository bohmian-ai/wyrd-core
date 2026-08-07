#!/usr/bin/env bash
# Isolated negative self-tests for every boundary check script.
#
# ANTI-NO-OP GUARANTEE: every test INVOKES THE REAL check script
# (scripts/checks/*.sh or scripts/check_*.py) against an ISOLATED TEMP
# FIXTURE that contains a single injected violation. The test asserts a
# NON-ZERO exit from the real script. Inline twin logic is not used.
#
# Fixture mechanism per check:
#
#   no-plane-deps, sqlx-scope: WYRD_METADATA_FIXTURE=<temp-json>
#     Supplies a pre-built cargo metadata document containing the violation.
#     The real Python parser inside the script sees the injected dep.
#
#   client-tier: WYRD_CLIENT_TIER_SPEC_TREE=<fabricated cargo tree text>
#     The real check_tree() function in the script runs grep against the
#     injected text and must return non-zero.
#
#   pyo3-scope gate 1 (source scan): WYRD_CHECK_ROOT=<temp dir with crates/>
#     The real rg/grep call in the script scans the temp crates/ structure.
#   pyo3-scope gates 2-3 (Cargo.toml/lib.rs checks): WYRD_CHECK_ROOT=<temp dir>
#     with the correct wyrd-utils file structure containing no python feature.
#   pyo3-scope gate 4 (cargo tree): WYRD_PYO3_UTILS_TREE=<tree with pyo3>
#
#   mocks-scope: WYRD_CHECK_ROOT=<temp dir with crates/>
#     The real rg/grep call scans the temp crates/ structure.
#
#   proto-drift: WYRD_PROTO_SNAPSHOT_OVERRIDE=<path to old .bin copy>
#     The real script captures the current snapshot, regenerates, then compares
#     against the override (which differs), triggering the failure.
#     NOTE: This test DOES run cargo build to regenerate wyrd-tonic.
#
#   rustls-provider: WYRD_RUSTLS_GRAPH_FIXTURE=<temp file with ring in tree>
#                    WYRD_RUSTLS_META_FIXTURE=<temp file with reqwest 0.12>
#     The real grep/python logic in the script checks the injected content.
#
#   check_unwrap_audit.py, check_clippy_allow.py:
#     WYRD_CHECK_ROOT=<temp dir with crates/> containing the violation.
#
#   ci-parity: WYRD_WORKFLOW_OVERRIDE=<temp workflow yaml missing a task>
#     The real script diffs pre-pr deps against the injected workflow and fails.
#
#   sql-core-seam: WYRD_SEAM_TEST=<temp broken seam file name>
#     A seam file referencing a non-existent wyrd-sql-core symbol is placed
#     in crates/wyrd-dev-fixtures/tests/ transiently, the real cargo test
#     command compiles it and fails (compiler enforces the negative case),
#     then the temp file is deleted.
#
# Exit 0 only if every check correctly rejects its negative fixture.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
CHECKS_DIR="$REPO_ROOT/scripts/checks"

pass=0
fail=0

ok()  { echo "  [PASS] $1"; pass=$((pass+1)); }
nok() { echo "  [FAIL] $1"; fail=$((fail+1)); }

run_negative() {
    local label="$1"
    shift
    # Run the REAL command with provided env + args.
    # Assert NON-ZERO exit (violation must be detected).
    if "$@" >/dev/null 2>&1; then
        nok "$label: expected non-zero exit but got zero (check is a no-op)"
    else
        ok "$label"
    fi
}

echo "=== Running negative self-tests for boundary check scripts ==="
echo ""

# ─────────────────────────────────────────────────────────────────────────────
# no-plane-deps: inject a cloud SDK dep and a wyrd-testing dep via metadata JSON
# ─────────────────────────────────────────────────────────────────────────────
echo "[no-plane-deps] negative: cloud SDK dep only (gates 1+2 clean, gate 3 fires)..."
TMPDIR_NDEP=$(mktemp -d)
NDEP_META="$TMPDIR_NDEP/meta.json"
# Build a minimal cargo metadata document with ONLY a cloud SDK dep.
# Gates 1 (no out-of-tree path) and 2 (no wyrd-testing) pass; gate 3 fires.
# This fixture specifically tests the cloud SDK detection regex in gate 3:
# if that regex is broken, the script exits 0 and the self-test FAILS.
python3 - "$NDEP_META" << 'PYEOF'
import json, sys

meta = {
    "workspace_root": "/fake",
    "workspace_members": ["path+file:///fake/crates/wyrd-spec#wyrd-spec@0.1.0"],
    "packages": [
        {
            "id": "path+file:///fake/crates/wyrd-spec#wyrd-spec@0.1.0",
            "name": "wyrd-spec",
            "manifest_path": "/fake/crates/wyrd-spec/Cargo.toml",
            "dependencies": [
                {"name": "aws-sdk-s3", "kind": None, "path": None},
            ],
        }
    ],
    "resolve": {"nodes": []},
}
with open(sys.argv[1], "w") as f:
    json.dump(meta, f)
PYEOF
run_negative "no-plane-deps detects aws-sdk cloud dep via gate 3" \
    env WYRD_METADATA_FIXTURE="$NDEP_META" bash "$CHECKS_DIR/no-plane-deps.sh"
rm -rf "$TMPDIR_NDEP"

# Also test wyrd-testing gate specifically (cloud check passes, wyrd-testing fails)
echo "[no-plane-deps] negative: wyrd-testing-only violation..."
TMPDIR_NDEP2=$(mktemp -d)
NDEP_META2="$TMPDIR_NDEP2/meta.json"
python3 - "$NDEP_META2" << 'PYEOF'
import json, sys

meta = {
    "workspace_root": "/fake",
    "workspace_members": ["path+file:///fake/crates/wyrd-spec#wyrd-spec@0.1.0"],
    "packages": [
        {
            "id": "path+file:///fake/crates/wyrd-spec#wyrd-spec@0.1.0",
            "name": "wyrd-spec",
            "manifest_path": "/fake/crates/wyrd-spec/Cargo.toml",
            "dependencies": [
                {"name": "wyrd-testing", "kind": "dev", "path": None},
            ],
        }
    ],
    "resolve": {"nodes": []},
}
with open(sys.argv[1], "w") as f:
    json.dump(meta, f)
PYEOF
run_negative "no-plane-deps detects wyrd-testing in fixture" \
    env WYRD_METADATA_FIXTURE="$NDEP_META2" bash "$CHECKS_DIR/no-plane-deps.sh"
rm -rf "$TMPDIR_NDEP2"

# ─────────────────────────────────────────────────────────────────────────────
# sqlx-scope: inject sqlx in a non-approved crate via metadata JSON
# ─────────────────────────────────────────────────────────────────────────────
echo "[sqlx-scope] negative: sqlx in non-approved crate..."
TMPDIR_SQLX=$(mktemp -d)
SQLX_META="$TMPDIR_SQLX/meta.json"
python3 - "$SQLX_META" << 'PYEOF'
import json, sys

meta = {
    "workspace_root": "/fake",
    "workspace_members": ["path+file:///fake/crates/wyrd-spec#wyrd-spec@0.1.0"],
    "packages": [
        {
            "id": "path+file:///fake/crates/wyrd-spec#wyrd-spec@0.1.0",
            "name": "wyrd-spec",
            "manifest_path": "/fake/crates/wyrd-spec/Cargo.toml",
            "dependencies": [
                {"name": "sqlx", "kind": None, "path": None},
            ],
        }
    ],
    "resolve": {"nodes": []},
}
with open(sys.argv[1], "w") as f:
    json.dump(meta, f)
PYEOF
run_negative "sqlx-scope detects sqlx in non-approved crate" \
    env WYRD_METADATA_FIXTURE="$SQLX_META" bash "$CHECKS_DIR/sqlx-scope.sh"
rm -rf "$TMPDIR_SQLX"

# ─────────────────────────────────────────────────────────────────────────────
# client-tier: inject axum in wyrd-spec dep tree via tree fixture
# ─────────────────────────────────────────────────────────────────────────────
echo "[client-tier] negative: axum in wyrd-spec dep tree..."
run_negative "client-tier detects axum in wyrd-spec" \
    env WYRD_CLIENT_TIER_SPEC_TREE="wyrd-spec v0.1.0
├── axum v0.8.0
│   └── tokio v1.49.0" bash "$CHECKS_DIR/client-tier.sh"

# ─────────────────────────────────────────────────────────────────────────────
# pyo3-scope gate 1: pyo3 reference in a non-wyrd-utils crate source file
# ─────────────────────────────────────────────────────────────────────────────
echo "[pyo3-scope gate 1] negative: pyo3 import in wyrd-spec source..."
TMPDIR_PYO3=$(mktemp -d)
mkdir -p "$TMPDIR_PYO3/crates/wyrd-spec/src"
mkdir -p "$TMPDIR_PYO3/crates/wyrd-utils/src"
# Add a valid wyrd-utils/Cargo.toml and lib.rs so gates 2-3 pass
mkdir -p "$TMPDIR_PYO3/crates/wyrd-utils"
cat > "$TMPDIR_PYO3/crates/wyrd-utils/Cargo.toml" << 'EOF'
[package]
name = "wyrd-utils"

[features]
python = ["dep:pyo3"]

[dependencies]
pyo3 = { workspace = true, optional = true }
EOF
cat > "$TMPDIR_PYO3/crates/wyrd-utils/src/lib.rs" << 'EOF'
#[cfg(feature = "python")]
pub mod python_helpers {}
EOF
# The violation: pyo3 reference outside wyrd-utils
echo 'use pyo3::prelude::*;' > "$TMPDIR_PYO3/crates/wyrd-spec/src/lib.rs"
# Gate 4 needs a tree; inject a clean one so only gate 1 fires
run_negative "pyo3-scope detects pyo3 ref in non-wyrd-utils crate" \
    env WYRD_CHECK_ROOT="$TMPDIR_PYO3" WYRD_PYO3_UTILS_TREE="wyrd-utils v0.1.0" \
    bash "$CHECKS_DIR/pyo3-scope.sh"
rm -rf "$TMPDIR_PYO3"

# ─────────────────────────────────────────────────────────────────────────────
# pyo3-scope gates 2-3: wyrd-utils missing python feature / optional pyo3
# ─────────────────────────────────────────────────────────────────────────────
echo "[pyo3-scope gates 2-3] negative: wyrd-utils missing python feature in Cargo.toml..."
TMPDIR_PYO3B=$(mktemp -d)
mkdir -p "$TMPDIR_PYO3B/crates/wyrd-utils/src"
# No pyo3 reference in source (gate 1 passes), but Cargo.toml lacks python feature (gate 2 fails)
cat > "$TMPDIR_PYO3B/crates/wyrd-utils/Cargo.toml" << 'EOF'
[package]
name = "wyrd-utils"

[dependencies]
EOF
cat > "$TMPDIR_PYO3B/crates/wyrd-utils/src/lib.rs" << 'EOF'
#[cfg(feature = "python")]
pub mod python_helpers {}
EOF
run_negative "pyo3-scope detects missing python feature in wyrd-utils" \
    env WYRD_CHECK_ROOT="$TMPDIR_PYO3B" WYRD_PYO3_UTILS_TREE="wyrd-utils v0.1.0" \
    bash "$CHECKS_DIR/pyo3-scope.sh"
rm -rf "$TMPDIR_PYO3B"

# ─────────────────────────────────────────────────────────────────────────────
# pyo3-scope gate 4: pyo3 in wyrd-utils default dep tree
# ─────────────────────────────────────────────────────────────────────────────
echo "[pyo3-scope gate 4] negative: pyo3 in wyrd-utils default tree..."
TMPDIR_PYO3C=$(mktemp -d)
mkdir -p "$TMPDIR_PYO3C/crates/wyrd-utils/src"
# Gates 1-3 pass; gate 4 sees pyo3 in the injected tree
cat > "$TMPDIR_PYO3C/crates/wyrd-utils/Cargo.toml" << 'EOF'
[package]
name = "wyrd-utils"

[features]
python = ["dep:pyo3"]

[dependencies]
pyo3 = { workspace = true, optional = true }
EOF
cat > "$TMPDIR_PYO3C/crates/wyrd-utils/src/lib.rs" << 'EOF'
#[cfg(feature = "python")]
pub mod python_helpers {}
EOF
run_negative "pyo3-scope detects pyo3 in wyrd-utils default dep tree" \
    env WYRD_CHECK_ROOT="$TMPDIR_PYO3C" \
    WYRD_PYO3_UTILS_TREE="wyrd-utils v0.1.0
└── pyo3 v0.23.0" \
    bash "$CHECKS_DIR/pyo3-scope.sh"
rm -rf "$TMPDIR_PYO3C"

# ─────────────────────────────────────────────────────────────────────────────
# sql-core-seam: broken seam file referencing a non-existent symbol
# A temp seam file is placed in wyrd-dev-fixtures/tests/, the real script
# compiles it (Rust type system rejects the bad import), then the file is
# deleted. This is the only test that transiently writes to the workspace --
# only to the tests/ directory, not to src/ or manifests.
# ─────────────────────────────────────────────────────────────────────────────
echo "[sql-core-seam] negative: seam file importing a non-existent symbol..."
BROKEN_SEAM="$REPO_ROOT/crates/wyrd-dev-fixtures/tests/sql_core_seam_broken_neg.rs"
cat > "$BROKEN_SEAM" << 'EOF'
//! Negative seam test -- references a symbol that does not exist in
//! wyrd-sql-core to prove the real sql-core-seam.sh fails on compile error.
use wyrd_sql_core::this_symbol_does_not_exist_in_wyrd_sql_core;

#[test]
fn negative_seam_must_fail_to_compile() {
    // This test should never run -- the import above causes E0432.
    let _ = this_symbol_does_not_exist_in_wyrd_sql_core;
}
EOF
run_negative "sql-core-seam fails on broken seam file" \
    env WYRD_SEAM_TEST="sql_core_seam_broken_neg" bash "$CHECKS_DIR/sql-core-seam.sh"
rm -f "$BROKEN_SEAM"

# ─────────────────────────────────────────────────────────────────────────────
# mocks-scope: wiremock reference in a non-allowlisted crate source file
# ─────────────────────────────────────────────────────────────────────────────
echo "[mocks-scope] negative: wiremock in non-allowlisted crate..."
TMPDIR_MOCK=$(mktemp -d)
mkdir -p "$TMPDIR_MOCK/crates/wyrd-spec/src"
echo 'use wiremock::MockServer;' > "$TMPDIR_MOCK/crates/wyrd-spec/src/lib.rs"
run_negative "mocks-scope detects wiremock in non-allowlisted source" \
    env WYRD_CHECK_ROOT="$TMPDIR_MOCK" bash "$CHECKS_DIR/mocks-scope.sh"
rm -rf "$TMPDIR_MOCK"

# ─────────────────────────────────────────────────────────────────────────────
# proto-drift: the "committed" snapshot is mutated so it differs from what
# protoc regenerates. WYRD_PROTO_COMMITTED_SNAPSHOT points at a temp copy of
# the real .bin with one byte appended. The real script regenerates wyrd.v1.bin
# (canonical path, always same output from protoc), then compares against the
# mutated snapshot -- they differ, so the check fails. The real cargo build
# runs inside the real workspace (no temp workspace needed; the regenerated
# .bin is always the same bytes as long as protoc version is pinned).
# ─────────────────────────────────────────────────────────────────────────────
echo "[proto-drift] negative: committed snapshot byte-differs from regenerated descriptor..."
REAL_SNAPSHOT="$REPO_ROOT/crates/wyrd-tonic/proto/wyrd.v1.bin"
TMPDIR_PROTO=$(mktemp -d)
MUTATED="$TMPDIR_PROTO/mutated.bin"
# Copy and append a null byte to create a snapshot that won't match protoc output
cp "$REAL_SNAPSHOT" "$MUTATED"
printf '\x00' >> "$MUTATED"
run_negative "proto-drift detects mutated committed snapshot" \
    env WYRD_PROTO_COMMITTED_SNAPSHOT="$MUTATED" bash "$CHECKS_DIR/proto-drift.sh"
rm -rf "$TMPDIR_PROTO"

# ─────────────────────────────────────────────────────────────────────────────
# rustls-provider: ring in production dep graph
# ─────────────────────────────────────────────────────────────────────────────
echo "[rustls-provider] negative: ring-backed rustls in graph fixture..."
TMPDIR_RUSTLS=$(mktemp -d)
RING_GRAPH="$TMPDIR_RUSTLS/ring-graph.txt"
cat > "$RING_GRAPH" << 'EOF'
wyrd-tls v0.1.0
└── rustls feature "ring"
    └── ring v0.17.0
EOF
run_negative "rustls-provider detects ring in dep graph" \
    env WYRD_RUSTLS_GRAPH_FIXTURE="$RING_GRAPH" bash "$CHECKS_DIR/rustls-provider.sh"

# rustls-provider: AWS-LC absent from production dep graph
echo "[rustls-provider] negative: aws_lc_rs absent from graph fixture..."
NO_LC_GRAPH="$TMPDIR_RUSTLS/no-lc-graph.txt"
cat > "$NO_LC_GRAPH" << 'EOF'
wyrd-tls v0.1.0
└── rustls v0.23.0
EOF
run_negative "rustls-provider detects absent aws_lc_rs provider" \
    env WYRD_RUSTLS_GRAPH_FIXTURE="$NO_LC_GRAPH" bash "$CHECKS_DIR/rustls-provider.sh"
rm -rf "$TMPDIR_RUSTLS"

# ─────────────────────────────────────────────────────────────────────────────
# check_unwrap_audit.py: .unwrap() in production source under WYRD_CHECK_ROOT
# ─────────────────────────────────────────────────────────────────────────────
echo "[unwrap-audit] negative: .unwrap() in production source under fixture root..."
TMPDIR_UNWRAP=$(mktemp -d)
mkdir -p "$TMPDIR_UNWRAP/crates/wyrd-spec/src"
cat > "$TMPDIR_UNWRAP/crates/wyrd-spec/src/lib.rs" << 'EOF'
pub fn bad() {
    let x: Option<i32> = None;
    let _ = x.unwrap();
}
EOF
run_negative "unwrap-audit detects .unwrap() in fixture root" \
    env WYRD_CHECK_ROOT="$TMPDIR_UNWRAP" python3 "$REPO_ROOT/scripts/check_unwrap_audit.py"
rm -rf "$TMPDIR_UNWRAP"

# ─────────────────────────────────────────────────────────────────────────────
# check_clippy_allow.py: unjustified #[allow(clippy::...)] in production source
# ─────────────────────────────────────────────────────────────────────────────
echo "[clippy-allow-audit] negative: unjustified allow in production source..."
TMPDIR_CLIPPY=$(mktemp -d)
mkdir -p "$TMPDIR_CLIPPY/crates/wyrd-spec/src"
cat > "$TMPDIR_CLIPPY/crates/wyrd-spec/src/lib.rs" << 'EOF'
#[allow(clippy::too_many_arguments)]
pub fn foo() {}
EOF
run_negative "clippy-allow-audit detects unjustified allow in fixture root" \
    env WYRD_CHECK_ROOT="$TMPDIR_CLIPPY" python3 "$REPO_ROOT/scripts/check_clippy_allow.py"
rm -rf "$TMPDIR_CLIPPY"

# ─────────────────────────────────────────────────────────────────────────────
# ci-parity: workflow missing a task that pre-pr depends on
# ─────────────────────────────────────────────────────────────────────────────
echo "[ci-parity] negative: workflow file missing a pre-pr dependency..."
TMPDIR_CI=$(mktemp -d)
FAKE_WORKFLOW="$TMPDIR_CI/lints-test.yml"
# Build a workflow that only has 3 steps; pre-pr has 16 deps -> definite drift
cat > "$FAKE_WORKFLOW" << 'EOF'
jobs:
  ci:
    steps:
      - run: mise run fmt:check
      - run: mise run lints
      - run: mise run test:unit
EOF
run_negative "ci-parity detects workflow missing required tasks" \
    env WYRD_WORKFLOW_OVERRIDE="$FAKE_WORKFLOW" bash "$CHECKS_DIR/ci-parity.sh"
rm -rf "$TMPDIR_CI"

# ─────────────────────────────────────────────────────────────────────────────
# Summary
# ─────────────────────────────────────────────────────────────────────────────
echo ""
echo "=== Negative self-test summary: $pass passed, $fail failed ==="
if [ "$fail" -gt 0 ]; then
    exit 1
fi
