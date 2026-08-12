---
name: wyrd-plan
description: Plan wyrd-core foundation features, refactors, migrations, contracts, transport, auth, telemetry, crypto, runtime, queue, versioning, SQL migration-core, fixtures, testing, and architecture as decision-complete executable task packets. Use before $wyrd-implement or $wyrd-implement-plan; do not implement or review completed code.
---

# Wyrd Core Plan

Convert user intent and live repository evidence into the smallest
decision-complete plan an implementation agent can execute without redesign.
This skill plans only `wyrd-core`; server, UI, SDK, Skald runtime, and
Vala/Bifrost changes belong to their consuming repositories.

Before planning, read `AGENTS.md`, `architecture/agent-rules.md`,
`architecture/wyrd-design.md`, `architecture/wyrd-doctrine.mdx`, the local
`wyrd-implement` and `wyrd-implement-plan` skills, and
`architecture/references/README.md`. Load applicable existing references,
inspect manifests, lockfiles, `mise.toml`, owners, callers, tests, and generated
surfaces. Use CodeGraph first when present. Design authority wins over source
drift; name and update any intentionally superseded decision.

Prove the primary workflow, owning crate, contract and dependency effects,
security/tenancy/audit implications, test seams, and executable commands. Build
a concrete Mermaid impact graph. Resolve every material choice: public or
persisted behavior, dependencies/features, ownership, migration, data loss,
auth, tenancy, audit, compatibility, and acceptance proof. Reversible private
mechanics remain implementation authority.

Load the bundled planning references progressively. Validate every proposed
command and cold-rehearse each task before `Ready`. Emit task packets for only
`$wyrd-implement`; use the validator at
`.agents/skills/wyrd-plan/scripts/validate_plan_artifacts.py` for materialized
plans. The root `$wyrd-implement-plan` serializes task implementation and
mandatory review.
