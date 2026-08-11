#!/usr/bin/env bash
# WHY THIS FILE EXISTS: PyO3 links against libpython at compile time. If it
# leaks into any foundation crate other than wyrd-utils (behind its optional
# python feature), every consumer — including the Rust-only server, CLI, and
# bifrost transport stack — transitively links libpython, making
# cross-compilation harder and adding a runtime dependency on the Python
# interpreter in environments that should be Python-free.
#
# WHAT IT CHECKS:
#   1. Zero PyO3 references in any crate except wyrd-utils.
#   2. wyrd-utils: python feature defined, PyO3 declared optional.
#   3. wyrd-utils: Python helpers are cfg-gated behind #[cfg(feature = "python")].
#   4. wyrd-utils default build: PyO3 does not appear in the dep tree.
set -euo pipefail

# WYRD_CHECK_ROOT: override the workspace root used for file scans and cargo
# tree calls. Set by the negative self-test harness to point at a temp fixture
# workspace containing the injected violation.
REPO_ROOT="${WYRD_CHECK_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
cd "$REPO_ROOT"

fail=0

# ── Gate 1: no PyO3 source references outside wyrd-utils ────────────────────
echo "pyo3-scope: gate 1 — scanning for PyO3 references outside wyrd-utils…"
# Use rg if available, otherwise fall back to grep.
if command -v rg &>/dev/null; then
    pyo3_hits=$(rg -n 'pyo3|pymodule|pyclass|pymethods|#\[pyfunction\]|Python<|Bound<|Py<|PyErr' \
        crates \
        --glob '!crates/wyrd-utils/**' \
        2>/dev/null || true)
else
    pyo3_hits=$(grep -rn --include='*.rs' --include='Cargo.toml' \
        -E 'pyo3|pymodule|pyclass|pymethods|#\[pyfunction\]|Python<|Bound<|Py<|PyErr' \
        crates \
        --exclude-dir=wyrd-utils \
        2>/dev/null || true)
fi
if [ -n "$pyo3_hits" ]; then
    echo "FAIL pyo3-scope gate 1: PyO3 reference found outside wyrd-utils:"
    echo "$pyo3_hits"
    fail=1
else
    echo "  gate 1 passed"
fi

# ── Gate 2: wyrd-utils declares python feature and optional PyO3 ─────────────
echo "pyo3-scope: gate 2 — checking wyrd-utils Cargo.toml…"
UTILS_TOML="crates/wyrd-utils/Cargo.toml"
if ! grep -qF 'python = ["dep:pyo3"]' "$UTILS_TOML"; then
    echo "FAIL pyo3-scope gate 2: wyrd-utils must gate PyO3 behind python = [\"dep:pyo3\"]"
    fail=1
else
    echo "  gate 2a passed (python feature defined)"
fi
if ! grep -qE 'pyo3 = \{ workspace = true, optional = true \}' "$UTILS_TOML"; then
    echo "FAIL pyo3-scope gate 2: wyrd-utils PyO3 dependency must be optional"
    fail=1
else
    echo "  gate 2b passed (PyO3 optional)"
fi

# ── Gate 3: wyrd-utils Python helpers are cfg-gated ──────────────────────────
echo "pyo3-scope: gate 3 — checking wyrd-utils cfg gate…"
if ! grep -q '#\[cfg(feature = "python")\]' crates/wyrd-utils/src/lib.rs; then
    echo "FAIL pyo3-scope gate 3: wyrd-utils Python helpers must be #[cfg(feature = \"python\")] gated"
    fail=1
else
    echo "  gate 3 passed"
fi

# ── Gate 4: default wyrd-utils build tree is PyO3-free ───────────────────────
# WYRD_PYO3_UTILS_TREE: pre-built cargo tree text for the wyrd-utils default
# dep tree. When set, skips the real cargo tree call. Used by negative tests.
echo "pyo3-scope: gate 4 — checking wyrd-utils dep tree at default features…"
if [ -n "${WYRD_PYO3_UTILS_TREE:-}" ]; then
    utils_default_tree="$WYRD_PYO3_UTILS_TREE"
else
    utils_default_tree=$(cargo tree -p wyrd-utils --no-default-features -e normal 2>/dev/null)
fi
pyo3_in_tree=$(echo "$utils_default_tree" | grep -E '(^|[ ─└├])pyo3' || true)
if [ -n "$pyo3_in_tree" ]; then
    echo "FAIL pyo3-scope gate 4: wyrd-utils pulls PyO3 without its python feature:"
    echo "$pyo3_in_tree"
    fail=1
else
    echo "  gate 4 passed"
fi

if [ "$fail" -eq 0 ]; then
    echo "pyo3-scope: all gates passed"
else
    exit 1
fi
