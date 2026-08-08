#!/usr/bin/env python3
"""Reject fixed-port and setup-only Postgres task regressions in the foundation."""
from pathlib import Path
import sys
import tomllib

root = Path(__file__).resolve().parents[2]
mise_path = (
    Path(sys.argv[1])
    if len(sys.argv) == 2
    else root / "mise.toml"
)
tasks = tomllib.loads(mise_path.read_text())["tasks"]

# These lifecycle tasks were removed in D10; they must not reappear.
removed = {"build:postgres", "setup:db-roles", "setup:postgres", "impl:setup", "pg:teardown"}
if removed & tasks.keys():
    raise SystemExit(f"removed lifecycle tasks remain: {sorted(removed & tasks.keys())}")

# No fixed Postgres host port in compose or mise.
compose = (root / "docker-compose.yml").read_text()
if "55432" in compose or '"55432:5432"' in compose:
    raise SystemExit("fixed Postgres host port remains")
if "docker ps" in (root / "mise.toml").read_text() or "docker rm" in (root / "mise.toml").read_text():
    raise SystemExit("global Docker enumeration/deletion remains")

# test:pg must route through the wrapper and have a hidden inner task.
pg_tasks = {"test:pg"}
for name in pg_tasks:
    task = tasks.get(name)
    if task is None or "with-test-postgres.sh" not in str(task.get("run", "")):
        raise SystemExit(f"{name} is not routed through the lifecycle wrapper")
    if f"{name}:inner" not in tasks:
        raise SystemExit(f"{name} has no hidden inner task")
    deps = set(task.get("depends", []))
    if deps & {"setup:postgres", "db:migrate"}:
        raise SystemExit(f"{name} still has setup-only Postgres dependencies")

# Aggregate tasks must not own a lifecycle.
aggregates = {"test:unit", "pre-pr"}
for name in aggregates:
    if name in tasks and "with-test-postgres.sh" in str(tasks[name].get("run", "")):
        raise SystemExit(f"aggregate {name} owns a nested lifecycle")

# No fixed port in any task.
for name, task in tasks.items():
    if ":inner" not in name and "55432" in str(task):
        raise SystemExit(f"fixed database endpoint remains in task {name}")

print("postgres task inventory: PASS")
