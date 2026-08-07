# Agent Rules

The following rules are explicitly defined for agents working in this
repository. These rules govern behavior, interactions, and responsibilities
to ensure consistent and efficient operation.

> **Scope note:** `wyrd-foundation` contains pure contracts, shared primitives,
> auth, telemetry, transport, and SQL migration core. Rules below that reference
> `TenantConn`, `OperatorPool`, Vala, Bifrost, `wyrd-server`, or other plane
> components are included as **cross-plane doctrine** for contributors who work
> across both repositories. Commands reference only paths that exist here.

- Cargo features must be earned. Creating new cargo features can invalidate
  compilation caches. Cargo features are designed around minimizing
  re-compilation across task suites. This is important.
- Client-tier crates do not depend on `sqlx`, cloud SDKs, `datafusion`, or
  `deltalake`. Enforced by `check:client-tier`. The only crates in this
  workspace permitted to use `sqlx` are `wyrd-sql-core` and `wyrd-dev-fixtures`.
- Cross-tier imports go through the owning tier's re-exports, not the
  underlying crate. In the server-plane (outside this repo): Vala and Bifrost
  code imports `use vala_sql::{TenantConn, OperatorPool, SqlError};` — never
  `use wyrd_sql::TenantConn;` from outside `wyrd-sql` itself. The same
  principle applies here: import `wyrd_sql_core` types from `wyrd-sql-core`,
  not from transitive paths.
- Bring types in with `use` and use bare names in signatures. Fully-qualified
  paths in signatures are noise that hides which crate owns the type; the `use`
  block at the top of the file is the correct place to declare that dependency.
  Applies to struct fields, function parameters, return types, trait bounds,
  and `where` clauses.
- Cross-plane SQL boundaries (for contributors working across `wyrd` + this
  repo): `TenantConn` is the load-bearing tenant boundary in the server plane.
  Do not add manual per-query tenant filters on a `TenantConn` path; RLS
  already enforces it. A function accepting `&mut TenantConn<'_>` MUST NOT
  call `conn.commit()` or `conn.rollback()`. The caller opens the transaction
  and owns its lifecycle.
- Audit cardinality follows auditable domain operations and independently
  durable state transitions, not endpoint invocations or requests. Never
  collapse multiple independently committed transitions into one audit row
  merely because one handler initiated them.
- Tests are inlined into the owning `src/` module (`#[cfg(test)] mod ...`) by
  default. An external `tests/` file must earn its place: it wires ≥2
  non-foundational crates, brings up a real gRPC/HTTP surface, or drives a
  real external service (Postgres). Each external file is a separate test
  binary.
- Tests needing Postgres go in `mod pg_tests` (or a `pg_*` file), never the
  fast lane. The fast lane runs without credentials, Docker, or a database.
- Never circumvent a gate to make it pass: no `#[allow]`, `#[ignore]`,
  deleting/weakening a test, or broadening a boundary glob to hide a violation.
  Fix the root cause. Use only a check's own sanctioned mechanism (e.g. a
  documented per-file allowlist) for a legitimately test-only, in-pattern case.
- Clippy lints are diagnostic signals, not paperwork. `#[allow(clippy::...)]`
  in production code is banned unless immediately preceded by a
  `// justification: <one-line reason>` comment naming why the lint is wrong
  for that specific site.
- Never hand-edit generated artifacts: OpenAPI/JSON schemas, golden files under
  each crate's `schemas/`. Change the source or generator, regenerate, and let
  `codegen:check` verify.
- Build scripts (`build.rs`) write generated output to `OUT_DIR`, never into
  the source tree.
- Map columns/fields by name, not positional index, whenever the two schemas
  can diverge. Positional alignment silently mis-maps fields.
- Never mention references to plans, tasks, or other agents in the codebase.
  The code is a permanent artifact; plans and tasks are ephemeral. Do not
  hardcode plan/task names, IDs, or agent references into the codebase.
- Rust structure follows `AGENTS.md` §5 "Required Struct-Centered Rust Style"
  as a hard acceptance criterion. Before adding a workflow, identify its owning
  concrete struct.
- Rust documentation follows `AGENTS.md` §14 as a hard acceptance criterion.
  Every new or materially modified Rust item requires rustdoc that explains
  intent, workflow role, operation, and relevant invariants or side effects.
  Missing or placeholder rustdoc is `BLOCK_BEFORE_MERGE`.
- Synchronous Rust is the default. An `async fn` is allowed only when it
  directly awaits IO or intentionally composes operations that do. Do not make
  validation, parsing, planning, transformations, or other pure computation
  async because an async caller invokes it.
- All `use` statements live at the top of the module (after the `//!` doc
  comment, before `const`/type/`fn` items). No function-scoped imports, no
  imports inside `impl` blocks. Two narrow exceptions: (a) `#[cfg(test)] mod
  tests { use super::*; ... }` — tests are their own scope; (b) `use
  TraitName as _;` to enable trait methods inside a single generic function
  where the trait does not belong in module scope.
- Before the server fetches a user- or tenant-supplied URL, SSRF-screen it:
  resolve DNS, reject if any resolved IP is internal, and pin the connection to
  the screened address. (Cross-plane doctrine for server contributions.)
- Validate the resolved/effective value, not the literal input. Checking a
  string then re-resolving is a check-then-use race (DNS rebinding); resolve
  once, validate, act on that exact result.
