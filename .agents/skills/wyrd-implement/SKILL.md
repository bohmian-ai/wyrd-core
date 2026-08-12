---
name: wyrd-implement
description: Execute or resume exactly one approved, decision-complete wyrd-core foundation task through verified completion or a genuine material-authority blocker. Use for bounded Rust contracts, shared primitives, transport, auth, telemetry, crypto, runtime, queue, versioning, SQL migration-core, or fixture changes. Do not use to plan, close a whole plan, or change server, UI, SDK, Skald-runtime, or Vala/Bifrost repositories.
---

# Wyrd Core Implement

Execute one `Ready` task through `COMPLETE` or a genuine material `BLOCKED`
condition. Use `orient -> localize -> implement -> validate -> refine`; command,
compiler, test, lint, fixture, and local-environment failures are refinement
inputs while an in-scope recovery exists.

Before editing, read the task and approved plan, `AGENTS.md`,
`architecture/agent-rules.md`, `architecture/wyrd-design.md`, and—when a
contract or behavior changes—`architecture/wyrd-doctrine.mdx`. Read
`architecture/references/languages/implementation-execution.md`, then load
only relevant files from `architecture/references/README.md`. Inspect git
status, `mise.toml`, manifests, lockfiles, current owners, callers, and tests.
Use CodeGraph first when `.codegraph/` exists.

Keep all edits inside this foundation workspace. `wyrd-spec` remains IO-,
async-, and PyO3-free; only `wyrd-sql-core` and `wyrd-dev-fixtures` may use
SQLx. Follow the required struct-centered Rust style, rustdoc, stable-error,
and test-integrity rules in `AGENTS.md`.

Implement private mechanics and bounded corrections, recording them in the
living task. Stop before a public/wire/generated/persisted contract, migration,
dependency or feature, auth/tenancy/policy/audit semantics, ownership direction,
or acceptance outcome must change. Run focused checks sequentially using the
task’s exact feature set; use repository `mise` tasks when they own setup.
Finish with `git diff --check`, a complete diff audit, and recorded evidence.
