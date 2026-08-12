# Wyrd Foundation Reference Library

`architecture/references/` is the reusable Wyrd knowledge library for this
foundation repository. Skills own conversational or execution process; this
library owns concise, durable product and implementation knowledge. Start here
and load only the slices required by the question or change.

> **Scope note:** `wyrd-core` contains pure contracts, shared primitives,
> auth, telemetry, transport, and SQL migration core. It does not include the
> server, UI, client SDKs, Skald runtime plane, or Vala/Bifrost plane; those are
> separate consuming repositories. The doctrine and language references below
> govern cross-plane design decisions this foundation must honor. References to
> plane-specific concepts (the server-tier registry, the Skald and Vala planes,
> the Python SDK aggregator, etc.) are **explanatory cross-plane doctrine** —
> read them as design context, not as runnable local paths.

## Layout

```text
references/
  doctrine/      product framing and canonical boundary facts
  architecture/  implementation ownership and structural patterns
  languages/     Rust, testing, and execution references
```

## Canonical routes

| Reference | Load when the question or change touches |
|---|---|
| `doctrine/positioning-and-vocabulary.md` | Card vocabulary, envelope, `CardRef`, v1 kinds, or removed concepts |
| `doctrine/architecture-constraints.md` | Wyrd/Vala/Skald boundaries, deployment, client-tier constraints, or observation identity |
| `architecture/patterns.md` | Ownership, contract placement, and structural patterns |
| `languages/implementation-execution.md` | Execution authority, adaptation, verification recovery, or completion evidence |
| `languages/rust-core.md` | Rust ownership, async, traits, allocation, or API shape |
| `languages/errors.md` | Stable errors and Rust/HTTP mapping |
| `languages/testing-workflows.md` | Test tiers, Postgres lane, verification scope, boundary checks, and foundation CI gates |

## Reading rules

- Read a selected file completely, then use its stable Wyrd anchors to inspect
  current code only when the question depends on implementation reality.
- Treat `architecture/wyrd-design.md` as the protocol authority and
  `architecture/wyrd-doctrine.mdx` as the public rationale when references and
  code disagree.
- Cross-plane references (Skald, Vala, PyO3, server, UI) are included as
  explanatory doctrine context. They are not runnable local paths. Resolve any
  doctrine question back to the relevant rule, not to a plane path.
- Keep this library concise and nonredundant. Add or extend a reference only
  when a durable knowledge gap is demonstrated.

## Consumers

- `.agents/skills/wyrd-advise/SKILL.md`
- `.agents/skills/wyrd-implement/SKILL.md`
- `.agents/skills/wyrd-implement-plan/SKILL.md`
- `.agents/skills/wyrd-plan/SKILL.md`
- `.agents/skills/wyrd-review/SKILL.md`
