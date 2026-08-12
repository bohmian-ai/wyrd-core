---
name: wyrd-advise
description: "Give doctrine-grounded, opinionated advice for the wyrd-core foundation workspace: contracts, shared Rust primitives, transport, auth, telemetry, crypto, runtime, queues, versioning, SQL migration core, and test fixtures. Use for Wyrd product or architecture trade-offs without editing code, creating plan artifacts, or handing work to another workflow."
---

# Wyrd Core Advise

Give advice, not implementation. Lead with one recommendation and its strongest
repository- or doctrine-grounded reason. Challenge boundary violations directly.

Before advice that depends on repository truth, read `AGENTS.md`,
`architecture/wyrd-design.md`, and the narrowest relevant owner, manifest, and
test. Treat `architecture/wyrd-design.md` as protocol authority and
`architecture/wyrd-doctrine.mdx` as rationale. Start progressive knowledge
loading at `architecture/references/README.md`; load only its existing routes.

`wyrd-core` owns foundation crates only. Server, UI, SDK, Skald runtime, and
Vala/Bifrost implementation paths are out of scope here. Use them only as
doctrine context; recommend work in the owning consumer repository when a
choice requires changing one of those surfaces.

For a material decision, state the owner, contract and tenant/audit impact, the
strongest trade-off, and what evidence would change the recommendation. Ask at
most one question only when it changes that recommendation. Do not invent
legacy nouns, compatibility aliases, or a new architectural layer.
