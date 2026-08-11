#!/usr/bin/env bash
# WHY THIS FILE EXISTS (D9): the CI job (lints-test.yml) enumerates each
# gate as an explicit mise run <task> step. pre-pr is the local aggregate
# that depends on the same set. If they drift, a developer can pass pre-pr
# locally and still fail CI (or vice versa). This script enforces parity
# mechanically: it resolves pre-pr's dependency list from mise tasks --json
# and diffs it against the ordered mise run <task> steps parsed from the
# workflow file.
#
# Pre-T9 behavior: if the workflow file does not yet exist, the check passes
# with an explicit "workflow not yet authored (pre-T9)" notice. Once T9
# authors .github/workflows/lints-test.yml, any drift fails the gate.
#
# NEGATIVE SELF-TEST: the negative self-test harness sets WYRD_WORKFLOW_OVERRIDE
# to a temp fixture workflow path and invokes this real script, asserting non-zero
# exit on the drift. The clean-tree test runs the script without any override and
# asserts zero exit.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

# WYRD_WORKFLOW_OVERRIDE: absolute path to an alternative workflow YAML.
# When set, the check compares against that file instead of the canonical
# .github/workflows/lints-test.yml. Used by the negative self-test harness.
WORKFLOW="${WYRD_WORKFLOW_OVERRIDE:-$REPO_ROOT/.github/workflows/lints-test.yml}"

# ── Resolve pre-pr dependency list from mise ----------------------------------
echo "ci-parity: resolving pre-pr dependency list via mise tasks --json..."
MISE="${MISE_BIN:-}"
if [ -z "$MISE" ]; then
    MISE=$(command -v mise 2>/dev/null || echo "")
fi
if [ -z "$MISE" ] && [ -x "$HOME/.local/bin/mise" ]; then
    MISE="$HOME/.local/bin/mise"
fi
if [ -z "$MISE" ]; then
    echo "FAIL ci-parity: mise not found on PATH or at ~/.local/bin/mise"
    exit 1
fi

TASKS_FILE=$(mktemp)
"$MISE" tasks --json > "$TASKS_FILE" 2>/dev/null

PRE_PR_DEPS=$(python3 - "$TASKS_FILE" << 'PYEOF'
import json, sys

tasks_path = sys.argv[1]
with open(tasks_path) as f:
    tasks = json.load(f)

for task in tasks:
    name = task.get("name", "") or task.get("task", "")
    if name == "pre-pr":
        deps = task.get("depends", []) or task.get("depends_on", [])
        for d in deps:
            print(d)
        sys.exit(0)

print("ERROR: pre-pr task not found in mise tasks --json output", file=sys.stderr)
sys.exit(1)
PYEOF
)
RET=$?
rm -f "$TASKS_FILE"

if [ "$RET" -ne 0 ] || [ -z "$PRE_PR_DEPS" ]; then
    echo "FAIL ci-parity: could not extract pre-pr dependency list"
    exit 1
fi

echo "ci-parity: pre-pr depends on:"
echo "$PRE_PR_DEPS" | sed 's/^/  /'

# ── Check for workflow file ---------------------------------------------------
if [ ! -f "$WORKFLOW" ]; then
    echo ""
    echo "ci-parity: workflow not yet authored (pre-T9)"
    echo "  Expected: $WORKFLOW"
    echo "  Once T9 authors the workflow file, this check will validate parity."
    echo "ci-parity: passed (pre-T9 -- no workflow to compare against)"
    exit 0
fi

# ── Parse the ci job's mise run steps from the workflow ----------------------
echo "ci-parity: parsing mise run steps from $WORKFLOW..."
CI_STEPS=$(python3 - "$WORKFLOW" << 'PYEOF'
import re, sys

workflow_path = sys.argv[1]
with open(workflow_path) as f:
    content = f.read()

# Match lines like: run: mise run <task-name>
# or:              run: mise -C ... run <task-name>
# or multi-line:   run: |
#                    mise run <task-name>
# Task names may contain word chars, colons, and hyphens (e.g. check:no-plane-deps).
# Only match non-comment lines (lines not starting with optional whitespace + '#').
pattern = re.compile(r'^(?!\s*#).*mise(?:\s+-C\s+\S+)?\s+run\s+([\w:][a-zA-Z0-9_:.-]*)', re.MULTILINE)
matches = pattern.findall(content)

seen = set()
for m in matches:
    if m not in seen:
        seen.add(m)
        print(m)
PYEOF
)

if [ -z "$CI_STEPS" ]; then
    echo "FAIL ci-parity: no 'mise run <task>' steps found in $WORKFLOW"
    echo "  The CI workflow must enumerate each gate as an explicit step."
    exit 1
fi

echo "ci-parity: CI job steps:"
echo "$CI_STEPS" | sed 's/^/  /'

# ── Compare the two ordered lists --------------------------------------------
echo "ci-parity: comparing pre-pr deps vs CI steps..."
DIFF_OUT=$(diff \
    <(echo "$PRE_PR_DEPS") \
    <(echo "$CI_STEPS") \
    || true)

if [ -n "$DIFF_OUT" ]; then
    echo "FAIL ci-parity: pre-pr dependency list and CI step list differ:"
    echo "$DIFF_OUT"
    echo ""
    echo "  Fix: update mise.toml pre-pr depends= or .github/workflows/lints-test.yml"
    echo "  so the two lists are identical in content and order."
    exit 1
fi

echo "ci-parity: passed -- pre-pr dependency list matches CI job steps exactly"
