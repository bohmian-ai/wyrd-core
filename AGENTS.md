# AGENTS.md

Foundation repository-wide standards for agentic and human contributors.
Skills, task packets, and CI gates reference this file as the source of truth.
When a skill or PR doc references "AGENTS.md", it means **this file**.

This document is **Wyrd-native**. Do not import legacy names, package names,
module names, compatibility shims, or migration shorthand into this repository.
Reproduce useful patterns under Wyrd vocabulary and Wyrd paths.

`wyrd-core` is a standalone, independently published repository of 20
foundation crates — pure contracts, shared primitives, auth, telemetry, crypto,
transport, SQL migration core, and test fixtures. It does not contain the
server, UI, client SDKs, or the Skald and Vala/Bifrost planes; those are
separate repositories that consume these crates. Cross-plane doctrine is
included only as explanatory context so the boundary rules here make sense;
every runnable command references paths that exist in this repository.

## 1. First Pass Before Editing

1. Read this file (AGENTS.md).
2. Follow all agent rules listed in `architecture/agent-rules.md`.
3. Read `architecture/wyrd-design.md`; it is the active design authority and
   wins over generated artifacts, older planning files, and implementation
   drift.
4. Read `architecture/wyrd-doctrine.mdx` before changing
   Wyrd contracts, public or internal APIs, SDK surfaces, CLI, MCP, docs,
   generated schemas, or implementation behavior.
5. Identify the owning crate (see §3 Ownership Boundaries).
6. Inspect the nearest existing Wyrd implementation and tests.
7. Check `Cargo.toml`, crate manifests, and lockfiles before relying on
   version-specific behavior.

Do not invent a new architecture until the current Wyrd boundary proves wrong
for the user workflow. When an approved feature intentionally replaces a
design decision, name the superseded decision and update
`architecture/wyrd-design.md` before or in the same cohesive change as the
code that relies on the replacement.

## 2. Current Decisions

Locked cross-cutting decisions that any contributor must honor:

- The protocol doctrine in `architecture/wyrd-design.md` is the first design
  filter for Wyrd nouns, layers, services, and public surfaces. Internal APIs,
  external APIs, generated schemas, and agent-facing contracts must align with it.
- Wyrd is the AI layer for human and agentic workflows, not a general-purpose
  framework or runtime for arbitrary code execution.
- Wyrd follows a language-agnostic client/server model. The Wyrd server owns
  durable behavior and core logic; clients project server contracts and call API
  surfaces.
- Core durable logic is Rust-only server logic. Contracts live on the API wire
  through typed schemas, HTTP/MCP payloads, generated docs, and stable error
  codes so any language can implement a Wyrd client.
- Rust, Python, and TypeScript are first-class client languages. Go is planned
  and becomes first-class only when its SDK and the same contract/journey gates
  ship. SDKs may add local authoring helpers, OTEL integration, and test
  tooling, but must not move server-owned durable behavior out of the server.
- Wyrd must work both self-hosted and as a cloud SaaS product. SaaS and
  enterprise deployments require full tenant separation for identity, authz,
  storage, registry, policy, audit, observability, evaluation, and generated
  artifacts.
- Wyrd is agent-first and headless. MCP, CLI, HTTP, generated schemas, stable
  errors, and machine-readable docs are primary surfaces.
- Every registered AI system component is a `Card` with the shared envelope:
  `apiVersion: wyrd/v1`, top-level `metadata`, `kind`, `spec`,
  server-derived `relationships`, and server-managed `status`. There is no
  outer `kind: Card` wrapper. The target v1 doctrine is 16 native kinds plus
  `External`: `Data`, `Model`, `Artifact`, `Experiment`, `Prompt`, `Agent`,
  `Workflow`, `Mcp`, `Service`, `Policy`, `Audit`, `Drift`, `Eval`, `Source`,
  `Trigger`, and `Operator`.
- `Tool` is a Skald/runtime registry concept, not a Card kind.
- Sub-agency is an Agent-to-Agent relationship, not a `SubAgent` Card kind.
- `Skill` is not a v1 Card kind unless a future architecture decision adds it.
- Current `wyrd-spec` code still exposes stale `Tool`, `Skill`, and
  `SubAgent` specs and lacks `SourceSpec`. Treat that as implementation drift
  to remove, not as contract precedent.
- `CardRef` carries `kind`, `name`, one `version` field, optional `space`, and
  optional `uid`. Do not introduce a separate version requirement field.
- `wyrd-spec` is IO-free, async-free, and foundational. It is strictly
  PyO3-free. Specs, schemas, validators, the error catalog, and identity
  newtypes stay PyO3-free.
- Client-tier crates do not depend on `sqlx`, cloud SDKs, `datafusion`, or
  `deltalake`. Enforced by `check:client-tier` in CI.
- Skald owns reusable agent primitives. Vala may depend on Skald for evaluation.
  Skald does not depend on Vala. (Skald and Vala planes live outside this repo;
  their doctrine is included as explanatory context, not runnable local paths.)
- Crate ownership includes dependency cost. Do not move a specialized dependency
  into a foundational or broadly consumed crate merely to centralize
  configuration. Keep it in the narrowest crate that owns the behavior.
- MCP is first-class; read tools are always available, write tools require
  explicit scopes.
- Audit is foundational across CLI, MCP, and SDK surfaces.

## 3. Ownership Boundaries

Within `wyrd-core`:

- `crates/wyrd-spec`: pure contracts, ids, cards/specs, schema generation,
  request/response shapes, validation, stable error catalog.
- `crates/skald-spec`: Skald provider/capability specs (contracts only; no
  runtime).
- `crates/wyrd-tonic`: gRPC/protobuf transport layer.
- `crates/wyrd-client`: HTTP + gRPC client for the Wyrd server.
- `crates/wyrd-utils`: shared primitives and helpers.
- `crates/wyrd-runtime`: async runtime boundary helpers.
- `crates/wyrd-queue`: task queue primitives.
- `crates/wyrd-semver`, `crates/wyrd-version`: versioning types and identity.
- `crates/wyrd-telemetry`: OpenTelemetry integration.
- `crates/wyrd-crypt`: cryptographic primitives.
- `crates/wyrd-auth-issue`, `crates/wyrd-auth-verify`, `crates/wyrd-auth-check`,
  `crates/wyrd-auth-oidc`: auth token issuance, verification, authorization, OIDC.
- `crates/wyrd-tls`: TLS configuration.
- `crates/wyrd-sql-core`: Postgres migration runner and pool primitives. The
  only crate in this repo permitted to use `sqlx` directly.
- `crates/wyrd-dev-fixtures`: Postgres test fixtures (dev/test only; `sqlx`
  permitted here). Never enabled on production builds.
- `crates/wyrd-error-derive`: `WyrdError` derive macro.
- `crates/wyrd-test-contract-macros`: contract test macros.

Out of scope for this repository (owned by separate consuming repositories):

- the server-tier registry handle and engine
- the server, CLI, MCP, and UI host
- the Skald runtime plane
- the Vala/Bifrost plane
- the language SDK surfaces (Rust, Python, TypeScript, and planned Go)

When behavior crosses boundaries, put the durable contract in `wyrd-spec`,
expose the API through language-agnostic wire contracts, and implement client
surfaces in the appropriate SDK repository.

## 4. Rust Core Rules

- Keep core behavior in Rust. Python should be typed and ergonomic, not a
  duplicate implementation.
- Use domain types instead of raw strings for durable identifiers
  (`TenantId`, `RunId`, `CardUid`, etc.).
- Prefer `&str`, `&Path`, `&[T]`, and typed references when ownership is not
  needed.
- Treat `.clone()` as a design question. Allowed only for concrete ownership
  needs, small boundary values, or `Arc::clone`/`Bytes::clone` for real shared
  state. Per-crate `CLONES.md` may whitelist additional cases.
- Use `thiserror` for crate-local library error enums. Use `anyhow` only in
  binaries.
- Use the derive-backed `wyrd_spec::error::WyrdError` catalog for public errors
  that cross HTTP, Python, MCP, CLI, or generated-documentation boundaries.
- Register public error metadata with
  `#[wyrd_error(code = "...", status = N, title = "...", remediation = "...")]`.
  Never hand-write parallel `code()`, `status()`, `remediation()`, or
  problem-json logic.
- Use `tracing` with structured fields for diagnostics.
- Use `secrecy::SecretString` for secrets and redacted custom `Debug` impls for
  secret-bearing structs.
- Do not use `unwrap()` for environment, filesystem, network, parsing, user
  input, database, storage, or external-service behavior in non-test code.
- Use `expect()` only for true invariants, with a message naming the invariant.
- Do not add wildcard dependency versions or per-crate profile blocks.
- Task and milestone checks use default features or the exact optional features
  exercised by the change. Whole-plan closeout and explicitly requested aggregate
  checks use `--all-features` so every code path is verified once after
  integration. Test and build tasks declare only the minimal feature set they
  need — `--all-features` in a test task forces the heavy cone to recompile at a
  different feature-union and defeats artifact reuse.

## 5. Abstraction Rules

- Concrete types when there is one implementation.
- Enums for closed sets where exhaustiveness matters.
- Traits when multiple real implementations share stable behavior.
- Generics for hot-path static dispatch.
- `Box<dyn Trait>` only when runtime extensibility is intentional.
- Keep traits small and capability-focused.
- Avoid broad platform traits created for one caller.
- Avoid `Arc<Mutex<T>>` by default; first check whether ownership, immutable
  state, a narrower lock, or message passing fits.

### Required Struct-Centered Rust Style

Wyrd uses a hybrid Rust style centered on cohesive, composable structs. This is
a repository requirement, not a preference. Code that produces the correct
behavior with the wrong structural shape is incomplete.

- All new and materially modified Rust code MUST follow this style. Existing
  functional code is implementation drift, not precedent. A localized edit
  does not require an unrelated crate-wide rewrite, but every new or materially
  changed symbol and its immediate module structure must comply.
- Stateful capabilities, multi-step workflows, dependency-backed behavior,
  configuration-backed behavior, and invariant-bearing domain behavior MUST
  have one clear owning concrete struct.
- Public operations and internal orchestration that use an owner's state or
  dependencies MUST be inherent methods on that owner.
- Compose dependencies through explicit struct fields and constructors. When
  multiple functions repeatedly accept the same clients, stores, configuration,
  or context, consolidate that state into the owning struct instead of
  threading it through a functional call graph.
- Free functions are permitted only for genuinely stateless, deterministic
  helpers, narrow conversions, and algorithms with no natural owner. A
  workflow function is not made stateless merely because all of its
  dependencies are parameters.
- Do not create zero-sized utility structs solely to turn unrelated functions
  into methods. The struct must own meaningful state, dependencies, identity,
  or invariants.
- Keep domain values and service handles distinct. Domain structs own
  construction, validation, invariants, and pure transformations. Service or
  handle structs own IO dependencies and orchestration.
- Struct-centered design does not permit god objects. Split a struct when its
  methods do not share a cohesive responsibility, dependencies, or invariants.
- Traits remain reserved for multiple real implementations sharing stable
  behavior. Do not create inheritance-shaped traits around a single struct.
- Wyrd Card envelopes and specs remain declarative. They MUST NOT acquire
  registry clients, storage clients, server behavior, or hidden IO merely to
  satisfy this style.

## 6. Async And Runtime Rules

- Synchronous Rust is the default. Every `async fn` MUST earn its state-machine,
  lifetime, cancellation, and `Send` complexity by directly awaiting IO or
  intentionally composing operations that do.
- Use async only at real IO boundaries: HTTP, database, storage, queues,
  network calls, and server handlers that await them.
- Keep validation, parsing, planning, transformations, and other pure
  computation synchronous. An async caller does not justify making a
  synchronous callee async.
- Keep the async boundary as narrow as practical. Do not propagate async
  through a module merely for uniform signatures or possible future IO.
- Do not create ad hoc Tokio runtimes in library code.
- Do not block inside async request paths without an explicit blocking strategy.
- Use bounded concurrency and timeouts for external calls when available.

## 7. PyO3 Boundary Rules

PyO3 is **not present** in `wyrd-core`. All foundation crates are
strictly PyO3-free. `wyrd-spec` is IO-free, async-free, and foundational.

Do not add a `python` feature or PyO3 imports to any crate in this repository.
Python-visible behavior belongs to the Python SDK repository behind optional
`python` features in approved owner crates, with a thin aggregator module at its
root.

The rules below are included as cross-plane doctrine for contributors who also
work in the Python SDK repository:

- PyO3 belongs in crates that own Python-visible behavior, behind an optional
  `python` feature.
- The Python aggregator module stays thin: no duplicated validation, lifecycle,
  registry, storage, or runtime logic.
- Keep `Python<'py>`, `Bound<'py, T>`, `Py<T>`, and `PyErr` out of crates that
  did not opt into a `python` feature.
- Name the `#[new]` method `fn __new__` (not `fn new`) and give it an explicit
  `#[pyo3(signature = (...))]`.
- Convert Python inputs at the boundary, then call Rust-native APIs.
- Use `Bound<'py, T>` for new PyO3 code; convert to `Py<T>` before storing
  across awaits, threads, or long-lived state.
- Never hold a `Bound<'py, T>` across `.await`.
- Release the GIL for blocking disk or network work not already routed through
  async infrastructure.

## 8. Server And Contract Rules

The server, CLI, MCP, and audit surfaces are owned by separate consuming
repositories. The rules below govern contract and API work done in this
repository:

- Public request/response bodies are typed structs.
- Wire types derive schema support where required by the feature gate.
- Public errors use the `WyrdError` derive.
- Versioned API contracts are explicit.
- Do not add compatibility routes or aliases for old surfaces.
- Preserve tenant isolation across every public and internal path.
- `wyrd-spec` is the durable contract layer for this foundation.

## 9. Testing Workflow

### Test Taxonomy (priority order)

Wyrd has three test tiers. Higher tiers prove the product works; lower tiers
prove a part works. A lower tier never substitutes for a missing higher one.

1. **Integration tests — supporting.** Exercise one subsystem against its real
   dependency (a migration runner against Postgres, a transport client against a
   gRPC mock). Use them to pin a seam contract precisely.
2. **Unit tests — supporting.** A single function or type in isolation, IO-free,
   credential-free, in the fast lane. Use them for pure logic, error/`WyrdError`
   mapping, and negative branches that are cleaner to force in-process.

User-journey tests (full client → server → client paths) are not present in
this repo; they live in the consuming repository where the server runs.

### Verification Scope

Run verification for the code you changed:

```bash
# Format and lint
cargo fmt --check --all
cargo clippy --workspace --all-features -- -D warnings

# Fast unit tests (no Postgres, no Docker)
cargo test --workspace

# Postgres-gated integration tests (requires docker compose up -d)
cargo test -p wyrd-dev-fixtures --features pg
cargo test -p wyrd-sql-core --features pg
```

Run `--all-features` for format/lint so every code path is covered. Use
targeted feature sets for test tasks to preserve artifact reuse.

### Client-Tier Boundary Check

The client-tier constraint is enforced in CI. To verify locally:

```bash
cargo check --workspace
```

Any dependency violation (`sqlx`, cloud SDKs, `datafusion`, `deltalake`) in a
client-tier crate is a build failure, not a warning.

## 10. Completion Standard

A change is not done until:

- The implementation matches the owning crate's local patterns.
- New core behavior has Rust tests when practical.
- Public contracts regenerate cleanly when touched.
- No legacy names, compatibility aliases, or plane-specific imports were added.
- Format, lint, and targeted tests for the touched surface pass.
- Do not circumvent a gate to make it pass: never weaken or disable a check,
  add `#[allow]`, delete or `#[ignore]` a failing test, or broaden a boundary
  glob to hide a real violation. Fix the underlying cause.

## 11. Git Identity Rules

- Use your locally configured Git identity.
- Never sign commits as anyone else.
- Never add AI co-author trailers.
- Never run `git config` to alter identity.
- Contributor identity: name=`Thorrester`, email=`sjforrester32@gmail.com`.

## 12. Planning

Planning artifacts for this repository are kept under this repository's own
planning workspace and are not tracked in version control (`/.dev/` is
git-ignored).

### Agent skill bindings

- Skills are referenced by name; each harness resolves a named skill from its
  own skill directory.
- Foundation Rust implementors use the `wyrd-implement` skill.
- The repository-local `wyrd-implement-plan` orchestrates approved foundation
  plans and loads this repository's architecture authorities before execution.
- The integration review uses `wyrd-review`.

## 13. Implementation Rules

- `wyrd-spec` is foundational but not a dumping ground for all contracts. If it
  is not spec-related, find another place.
- `wyrd-sql-core` owns Postgres migration runner and pool primitives only. It is
  the sole SQLx entry point in this workspace.
- `wyrd-dev-fixtures` is test/dev only; it is never enabled on production builds
  or published wheels.
- Wyrd is open source and independently publishable. This repository contains no
  private enterprise licensing keys, feature gates, startup hooks, or product
  contracts.
- KEEP IT SIMPLE STUPID: avoid over-engineering and adding unnecessary
  complexity. YAGNI. Follow a modular design that solves the problem at hand
  without adding extra layers, abstractions, or future-proofing not justified by
  current needs.
- Follow industry and Rust community best practices.

## 14. General Code Rules

- Do not add comments, docstrings, or type annotations to code you did not touch.
- Every new or materially modified Rust item MUST have rustdoc. This includes
  modules, structs, fields, enums, variants, traits, associated types,
  constants, type aliases, functions, methods, test helpers, and test functions,
  regardless of visibility.
- Rustdoc MUST explain intent, how the item participates in the surrounding
  workflow, and relevant invariants or side effects. Function and method docs
  MUST describe how the operation works at the level a maintainer needs to
  modify it safely.
- Every fallible Rust function or method MUST include a `# Errors` section
  naming the error conditions. Add `# Panics` whenever a panic remains
  possible, and document cancellation, partial progress, or retry behavior for
  async and durable operations when relevant.
- Documentation is part of implementation correctness. Missing or placeholder
  rustdoc on any touched Rust item is a hard blocker even when the code
  compiles and tests pass.
- All code must be directly testable.
- Functions and classes follow the single responsibility principle. If a
  function does two things, split it.
- Follow existing code style and patterns. Do not introduce new paradigms unless
  there is a compelling reason. Consistency over cleverness.

## 15. CodeGraph

In repositories indexed by CodeGraph (a `.codegraph/` directory exists at the
repo root), use `codegraph_explore` BEFORE grep/find or reading files.

If there is no `.codegraph/` directory, skip CodeGraph entirely.

## 16. Collaborator Context

**Who you are working with:** Steven Forrester — AI Platform engineer and TPM
at Shipt. Builds developer tooling, ML infrastructure, and agentic systems.
Deep Rust/Python/SvelteKit expertise.

Primary stack: Rust (tokio, axum, tonic, DataFusion, Delta Lake, PyO3), Python
(pytest, Pydantic, uv, maturin), SvelteKit 2 / Svelte 5 / Tailwind CSS v4.

**Working style:** Direct and concise. Lead with the answer, then the reasoning.
Be a senior technical architect. Volunteer opinions and blindspots. Push back on
over-engineering.
