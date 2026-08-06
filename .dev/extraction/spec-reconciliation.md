# wyrd-spec Reconciliation Ledger (T4)

Generated: 2026-08-06
Merge base: f58ec630
Surfaces SHA (monorepo): f2553269
Oracle SHA (wyrd-bifrost-oracle): 6b450184
Foundation worktree HEAD: fdda5bc (T1 Oracle copy)

## Disposition Key

- `replay`   — surfaces-only source; copy from f2553269
- `retain`   — Oracle-only; already present in T1 copy; do not overwrite
- `union`    — both refs touched; merge inventory (gen_schemas.rs)
- `generate` — golden/schema JSON output; defer to T6; do not touch here
- `delete`   — surfaces deletion to apply (or already absent after T1)

## Source Files

### replay (surfaces-only → copy from f2553269)

crates/wyrd-spec/src/auth/card_scope.rs
crates/wyrd-spec/src/auth/token.rs
crates/wyrd-spec/src/card/agent.rs
crates/wyrd-spec/src/card/artifact.rs
crates/wyrd-spec/src/card/audit.rs
crates/wyrd-spec/src/card/common.rs
crates/wyrd-spec/src/card/data.rs
crates/wyrd-spec/src/card/drift.rs
crates/wyrd-spec/src/card/experiment.rs
crates/wyrd-spec/src/card/mcp.rs
crates/wyrd-spec/src/card/mod.rs
crates/wyrd-spec/src/card/model.rs
crates/wyrd-spec/src/card/operator.rs
crates/wyrd-spec/src/card/prompt/mod.rs
crates/wyrd-spec/src/card/service.rs
crates/wyrd-spec/src/card/trigger.rs
crates/wyrd-spec/src/card/workflow.rs
crates/wyrd-spec/src/envelope.rs
crates/wyrd-spec/src/error.rs
crates/wyrd-spec/src/graph/canonical.rs       [NEW — surfaces-only addition]
crates/wyrd-spec/src/graph/composition.rs     [NEW — surfaces-only addition]
crates/wyrd-spec/src/graph/mod.rs             [NEW — surfaces-only addition]
crates/wyrd-spec/src/graph/root.rs            [NEW — surfaces-only addition]
crates/wyrd-spec/src/graph/topo.rs            [NEW — surfaces-only addition]
crates/wyrd-spec/src/lib.rs
crates/wyrd-spec/src/reference.rs
crates/wyrd-spec/src/refs/mod.rs              [NEW — surfaces-only addition]
crates/wyrd-spec/src/registry/enums.rs
crates/wyrd-spec/src/registry/hydrated.rs     [NEW — surfaces-only addition]
crates/wyrd-spec/src/registry/ids.rs
crates/wyrd-spec/src/registry/mod.rs
crates/wyrd-spec/src/registry/reads.rs
crates/wyrd-spec/src/registry/submission.rs
crates/wyrd-spec/src/registry/upload.rs       [NEW — surfaces-only addition]
crates/wyrd-spec/src/storage/upload.rs
crates/wyrd-spec/src/vala/eval/llm_judge.rs
crates/wyrd-spec/src/vala/eval/mod.rs
crates/wyrd-spec/src/vala/eval/record.rs
crates/wyrd-spec/src/vala/eval/spec.rs
crates/wyrd-spec/src/vala/observation.rs

Count: 41 paths (36 modified, 5 new directories/files + refs/mod.rs + hydrated.rs + upload.rs)

### retain (Oracle-only → already present in T1; do not overwrite)

crates/wyrd-spec/src/vala/api.rs
crates/wyrd-spec/src/vala/audit_detail.rs
crates/wyrd-spec/src/vala/error.rs
crates/wyrd-spec/src/vala/managed_columns.rs  [NEW in Oracle — present via T1]
crates/wyrd-spec/src/vala/mod.rs
crates/wyrd-spec/src/vala/trace/mod.rs

Count: 6 paths

### union (both refs touched → merge inventories in examples/gen_schemas.rs)

crates/wyrd-spec/examples/gen_schemas.rs

Count: 1 path

### delete (Oracle deletion — already absent in T1; verify only)

crates/wyrd-spec/src/vala/system_columns.rs   [absent in T1 ✓]

Count: 1 path (no action needed)

### generate (golden/schema JSON — defer to T6; no action here)

crates/wyrd-spec/schemas/*.json               (all schema goldens)
crates/wyrd-spec/tests/schemas/*.json         (all test schema goldens)
crates/wyrd-spec/tests/fixtures/**/*.json     (eval/observation schema fixtures)

Note: The surfaces diff includes R075/R090 renames of schema JSON files and
new schema JSON files. All classified generate — deferred to T6.

### replay (test input fixtures — parser inputs, not generated goldens)

crates/wyrd-spec/tests/fixtures/operator-remediation.yaml   [replayed from f2553269]
crates/wyrd-spec/tests/fixtures/trigger-on-drift.yaml       [replayed from f2553269]

Count: 2 paths. These YAML files are input fixtures consumed by the surfaces
operator/trigger parse tests; they ship with the source they exercise and are
replayed now (not regenerated at T6, which only owns the *.json goldens).

## Coverage Check

All paths in both sorted diff sets have exactly one disposition.

Surfaces diff (crates/wyrd-spec/src + examples): 41 source paths → replay (40) + union (1)
Surfaces test fixtures (crates/wyrd-spec/tests/fixtures): 2 yaml input fixtures → replay; *.json goldens → generate (T6)
Oracle diff (crates/wyrd-spec/src + examples): 8 paths → retain (6) + delete (1) + union (1)

No path is unclassified. AC1 satisfied.
