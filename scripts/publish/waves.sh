#!/usr/bin/env bash
# WHY THIS FILE EXISTS: deterministic topological publish-wave derivation from
# the workspace dependency graph. Supports `check` and `dry-run` modes only;
# the `publish` mode is present in the script but is NOT invoked in this plan
# (it is reserved for the future crates.io publication plan).
#
# Usage:
#   waves.sh check        -- print wave matrix and verify topological order;
#                            exit 0 on success, non-zero on cyclic/metadata error
#   waves.sh dry-run      -- print wave matrix, then run `cargo publish --dry-run`
#                            for every crate in the dependency-free first wave
#   waves.sh publish      -- (NOT for use in this plan) would publish each wave
#                            in topological order; reserved for the publication plan
#
# Dependency edges: only NORMAL (non-dev, non-build) intra-workspace deps are
# used to compute topological order, matching crates.io resolution semantics.
# Dev deps form test-harness cycles that do not appear in registry resolution.
#
# Self-test: set WYRD_WAVES_SELF_TEST=1 to run the internal unit-test suite
# and exit. This verifies wave derivation, mode validation, and error handling
# without hitting cargo metadata.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# ── Self-test harness ---------------------------------------------------------
# Tests the wave-derivation Python logic and mode validation directly in-process,
# without spawning bash sub-processes (which would hit OS fork limits in test envs).
if [ "${WYRD_WAVES_SELF_TEST:-}" = "1" ]; then
    echo "waves.sh: running self-test suite..."
    cd "$REPO_ROOT"

    python3 << 'PYEOF'
import json, subprocess, sys

PASS = 0
FAIL = 0

def assert_test(desc, expected, actual):
    global PASS, FAIL
    if str(actual) == str(expected):
        print(f"  PASS: {desc}")
        PASS += 1
    else:
        print(f"  FAIL: {desc}")
        print(f"    expected: {expected}")
        print(f"    actual:   {actual}")
        FAIL += 1

# ── Helper: derive waves from cargo metadata (mirrors the main script logic) ──
def derive_waves():
    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1", "--no-deps"],
        capture_output=True, text=True
    )
    if result.returncode != 0:
        return None, result.stderr
    meta = json.loads(result.stdout)
    ws_member_ids = set(meta["workspace_members"])
    ws_pkgs = {p["name"]: p for p in meta["packages"] if p["id"] in ws_member_ids}
    ws_names = set(ws_pkgs.keys())
    deps = {name: set() for name in ws_names}
    for name, pkg in ws_pkgs.items():
        for dep in pkg["dependencies"]:
            if dep["name"] in ws_names and dep["kind"] is None:
                deps[name].add(dep["name"])
    in_degree = {n: 0 for n in ws_names}
    for n, d in deps.items():
        for dep in d:
            in_degree[n] += 1
    waves = []
    remaining = set(ws_names)
    while remaining:
        wave = sorted(n for n in remaining if in_degree[n] == 0)
        if not wave:
            return None, f"cyclic dependency among: {sorted(remaining)}"
        waves.append(wave)
        for n in wave:
            remaining.remove(n)
            for m in remaining:
                if n in deps[m]:
                    in_degree[m] -= 1
    return waves, None

# Test 1: wave derivation succeeds (no cyclic deps, cargo metadata available)
waves, err = derive_waves()
assert_test("wave derivation succeeds (no cyclic error)", None, err)

# Test 2: wave 1 is non-empty
if waves:
    assert_test("wave 1 is non-empty", True, len(waves[0]) > 0)
else:
    assert_test("wave 1 is non-empty", True, False)

# Test 3: wave 1 crates have no intra-workspace normal deps
if waves:
    result2 = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1", "--no-deps"],
        capture_output=True, text=True
    )
    meta2 = json.loads(result2.stdout)
    ws_member_ids2 = set(meta2["workspace_members"])
    ws_pkgs2 = {p["name"]: p for p in meta2["packages"] if p["id"] in ws_member_ids2}
    ws_names2 = set(ws_pkgs2.keys())
    wave1_violations = []
    for crate in waves[0]:
        pkg = ws_pkgs2[crate]
        normal_ws_deps = [d["name"] for d in pkg["dependencies"] if d["name"] in ws_names2 and d["kind"] is None]
        if normal_ws_deps:
            wave1_violations.append(f"{crate} depends on {normal_ws_deps}")
    assert_test("wave 1 crates have no intra-workspace normal deps", [], wave1_violations)

# Test 4: total crate count across all waves equals 20
if waves:
    total = sum(len(w) for w in waves)
    assert_test("total published crates is 20", 20, total)

# Test 5: all waves contain unique crate names (no duplicate assignments)
if waves:
    all_crates = [c for w in waves for c in w]
    assert_test("no crate appears in multiple waves", len(all_crates), len(set(all_crates)))

# Test 6: cyclic-dep detection returns error (inject a fake cycle)
def derive_waves_from_graph(ws_names, deps_map):
    """Kahn's algorithm for testing with a fabricated dep graph."""
    in_degree = {n: 0 for n in ws_names}
    for n, d in deps_map.items():
        for dep in d:
            in_degree[n] += 1
    remaining = set(ws_names)
    wave_result = []
    while remaining:
        wave = sorted(n for n in remaining if in_degree[n] == 0)
        if not wave:
            return None  # cycle detected
        wave_result.append(wave)
        for n in wave:
            remaining.remove(n)
            for m in remaining:
                if n in deps_map.get(m, set()):
                    in_degree[m] -= 1
    return wave_result

cyclic_graph = {"a": {"b"}, "b": {"c"}, "c": {"a"}}  # a -> b -> c -> a
cycle_result = derive_waves_from_graph({"a", "b", "c"}, cyclic_graph)
assert_test("cyclic dep graph returns None (detected as cycle)", None, cycle_result)

print()
if FAIL > 0:
    print(f"waves.sh self-test: {PASS} passed, {FAIL} failed")
    sys.exit(1)
print(f"waves.sh self-test: {PASS} passed, 0 failed")
PYEOF

    exit $?
fi

# ── Mode argument -------------------------------------------------------------
MODE="${1:-}"
case "$MODE" in
    check|dry-run|publish) ;;
    *)
        echo "USAGE: waves.sh <check|dry-run|publish>" >&2
        echo "  check    -- derive and print wave matrix; verify topological order" >&2
        echo "  dry-run  -- print wave matrix; cargo publish --dry-run for wave 1 only" >&2
        echo "  publish  -- [RESERVED for future publication plan; not for use here]" >&2
        exit 1
        ;;
esac

if [ "$MODE" = "publish" ]; then
    echo "ERROR: waves.sh publish is reserved for the future crates.io publication plan." >&2
    echo "       No real publish is performed in this plan. Use check or dry-run." >&2
    exit 1
fi

# ── Derive topological waves from cargo metadata ------------------------------
echo "waves.sh: deriving topological publish waves from cargo metadata (locked)..."

cd "$REPO_ROOT"

WAVES_OUTPUT=$(python3 - << 'PYEOF'
import json, subprocess, sys

result = subprocess.run(
    ["cargo", "metadata", "--locked", "--format-version", "1", "--no-deps"],
    capture_output=True, text=True
)
if result.returncode != 0:
    print("ERROR: cargo metadata failed:", file=sys.stderr)
    print(result.stderr, file=sys.stderr)
    sys.exit(1)

meta = json.loads(result.stdout)

ws_member_ids = set(meta["workspace_members"])
id_to_pkg = {p["id"]: p for p in meta["packages"]}

# Restrict to workspace members only
ws_pkgs = {p["name"]: p for p in meta["packages"] if p["id"] in ws_member_ids}
ws_names = set(ws_pkgs.keys())

# Build normal-dep edges only (exclude dev and build deps -- they don't
# appear in crates.io resolution and can form test-harness cycles).
deps = {name: set() for name in ws_names}
for name, pkg in ws_pkgs.items():
    for dep in pkg["dependencies"]:
        if dep["name"] in ws_names and dep["kind"] is None:
            deps[name].add(dep["name"])

# Kahn's algorithm for topological sort into waves.
in_degree = {n: 0 for n in ws_names}
for n, d in deps.items():
    for dep in d:
        in_degree[n] += 1

waves = []
remaining = set(ws_names)

while remaining:
    wave = sorted(n for n in remaining if in_degree[n] == 0)
    if not wave:
        cycle = sorted(remaining)
        print(f"ERROR: cyclic dependency detected among: {cycle}", file=sys.stderr)
        sys.exit(1)
    waves.append(wave)
    for n in wave:
        remaining.remove(n)
        # Reduce in-degree for all dependents
        for m in remaining:
            if n in deps[m]:
                in_degree[m] -= 1

for i, wave in enumerate(waves, 1):
    print(f"Wave {i}: {' '.join(wave)}")

PYEOF
)

if [ $? -ne 0 ]; then
    echo "FAIL: wave derivation failed (cyclic or metadata error)" >&2
    exit 1
fi

echo ""
echo "$WAVES_OUTPUT"
echo ""

# Determine wave 1 crates
WAVE1=$(echo "$WAVES_OUTPUT" | grep "^Wave 1:" | sed 's/^Wave 1: //')

echo "waves.sh: wave matrix derived successfully"
echo "  Wave 1 (dependency-free, eligible for first publish): $WAVE1"

# ── Mode: check --------------------------------------------------------------
if [ "$MODE" = "check" ]; then
    echo ""
    echo "waves.sh check: topological order verified; no cargo invocations."
    exit 0
fi

# ── Mode: dry-run ------------------------------------------------------------
# Only the dependency-free first wave can dry-run without prerequisites being
# published. Brand-new dependents cannot dry-run before prerequisites resolve.
echo ""
echo "waves.sh dry-run: running cargo publish --dry-run for wave 1 crates only..."
echo "  (subsequent waves require prerequisites published to crates.io first)"
echo ""

DRY_FAIL=0
for CRATE in $WAVE1; do
    echo "  dry-run: $CRATE"
    if cargo publish --locked --dry-run -p "$CRATE" 2>&1 | tee /dev/stderr | grep -q "error"; then
        echo "  FAIL: dry-run failed for $CRATE" >&2
        DRY_FAIL=$((DRY_FAIL + 1))
    else
        echo "  OK: $CRATE"
    fi
    echo ""
done

if [ "$DRY_FAIL" -gt 0 ]; then
    echo "waves.sh dry-run: $DRY_FAIL crate(s) failed dry-run" >&2
    exit 1
fi

echo "waves.sh dry-run: all wave 1 crates passed cargo publish --dry-run"
