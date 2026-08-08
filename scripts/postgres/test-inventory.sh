#!/usr/bin/env bash
set -Eeuo pipefail

readonly root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/wyrd-pg-inventory.XXXXXX")"
trap 'rm -rf "$temp_dir"' EXIT

# Negative test: inject a removed lifecycle task name and confirm the check
# rejects it.
if [[ -f "$root/mise.toml" ]]; then
  cp "$root/mise.toml" "$temp_dir/mise.toml"
else
  # Synthesise a minimal mise.toml so the negative test can run without a real
  # mise.toml present (mise.toml is authored in T8; this guard lets T3 run the
  # negative test in isolation).
  cat >"$temp_dir/mise.toml" <<'TOML'
[tasks."test:pg"]
run = "scripts/postgres/with-test-postgres.sh -- mise run test:pg:inner"

[tasks."test:pg:inner"]
hide = true
run = "cargo test --locked -p wyrd-dev-fixtures --features pg -- --test-threads=1"

[tasks."test:unit"]
run = "cargo test --locked"

[tasks."pre-pr"]
depends = ["test:unit", "test:pg"]
TOML
fi

python3 - "$temp_dir/mise.toml" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
text = path.read_text()
# Inject a removed task name to trigger the check.
injection = '\n[tasks."setup:db-roles"]\nrun = "echo injected"\n'
path.write_text(text + injection)
PY

if python3 "$root/scripts/postgres/check-inventory.py" "$temp_dir/mise.toml"; then
    echo "inventory negative test unexpectedly passed" >&2
    exit 1
fi

echo "postgres inventory negative test: PASS"

# Positive test: the real mise.toml (if present) passes the check.
if [[ -f "$root/mise.toml" ]]; then
  python3 "$root/scripts/postgres/check-inventory.py"
  echo "postgres inventory positive test: PASS"
else
  echo "postgres inventory positive test: SKIPPED (mise.toml not yet authored by T8)"
fi
