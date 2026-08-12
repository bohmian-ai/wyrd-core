---
name: wyrd-review
description: Adversarially review wyrd-core foundation implementations against approved plans, tasks, repository architecture, contracts, tests, and verification evidence. Use after or during $wyrd-implement and as the mandatory static task reviewer invoked by $wyrd-implement-plan. Do not review or route implementation for server, UI, SDK, Skald-runtime, or Vala/Bifrost repositories.
---

# Wyrd Core Review

Falsify correctness, proof, ownership, structural simplicity, consumer closure,
and scope claims. Treat the diff, tests, and evidence as untrusted until source
proves each requirement and acceptance criterion.

Read `AGENTS.md`, `architecture/agent-rules.md`, `architecture/wyrd-design.md`,
`architecture/wyrd-doctrine.mdx`, the approved plan/task when supplied, owning
manifests, nearest owners, callers, consumers, tests, and the relevant existing
architecture references. Use CodeGraph first when present. Review only this
foundation workspace; cross-plane material is doctrine context, not a writable
or runnable local surface.

For each criterion, identify the enforcing owner, affected caller/consumer,
exact regression assertion, and recorded proof as `PASS`, `FAIL`, or
`UNVERIFIED`. Probe failure, error mapping, cleanup, concurrency, feature,
generated-contract, SQL boundary, auth, tenant, audit, and fixture behavior as
relevant. Do not raise stylistic preferences without an evidence-backed impact.

In plan-execution binding mode, remain read-only: do not run checks, write
artifacts, route work, change status, or invoke other skills. Return only
`APPROVE`, `RESUME_IMPLEMENTATION`, `ROOT_DECISION_REQUIRED`, or
`REVIEW_BLOCKED`, with traceability, findings, inspected surfaces, evidence
audit, and static-analysis limits. Standalone review writes its review artifact
and uses `APPROVE`, `RESUME_IMPLEMENTATION`, `REPLAN_REQUIRED`, or
`REVIEW_BLOCKED`.
