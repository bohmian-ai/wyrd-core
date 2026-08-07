# Testing Workflows

Use this reference with
`architecture/references/languages/implementation-execution.md`. A bounded
task runs task-defined focused verification with default or exact features. A
milestone or phase may add the default-feature workspace gate. Treat
all-feature workspace checks as whole-plan closeout gates, not per-task
defaults.

> **Scope note:** `wyrd-core` contains pure contracts, shared primitives,
> auth, telemetry, transport, and SQL migration core. It has no server, UI,
> Python package, TypeScript SDK, `WyrdTestServer`, Skald runtime, or
> Vala/Bifrost plane. The Three Tiers below are narrowed accordingly. Cross-plane
> boundary checks are included as **annotated context** for contributors who also
> work in the `wyrd` monorepo; the foundation-applicable gates are called out
> explicitly.

## Two Tiers (Priority Order For This Repository)

Wyrd ranks proof:

1. **Integration tests — primary for IO-touching surfaces.** Exercise one
   subsystem against its real dependency without standing up a full
   client→server journey. The key foundation integration surface is the
   Postgres lane: `wyrd-dev-fixtures` brings up a real `PgPool` (via
   `PgFixture`) to exercise the `wyrd-sql-core` migration runner and any
   crate that declares `--features wyrd-dev-fixtures/pg`. Use them to pin
   seam contracts precisely — migration success, rollback behavior, schema
   fingerprint conflict, negative SQL paths.
2. **Unit tests — supporting.** A single function or type in isolation,
   IO-free, credential-free, in the fast lane. Use for pure logic,
   error/`WyrdError` mapping, spec validation, and negative branches that are
   cleaner to force in-process.

Full user-journey tests (real SDK → real server → real SDK) are not present
in this repository. They live in the `wyrd` monorepo where `WyrdTestServer`
and the full server run. A capability contributed here may still require a
journey test upstream — coordinate via the plan that lands the server-side
consumer.

Rule: every new user- or agent-facing contract change in `wyrd-spec` should
have both a unit test (pure validation) and, where the behavior has a
downstream Postgres path in `wyrd-sql-core` or `wyrd-dev-fixtures`, an
integration test in the Postgres lane.

## Verification Scope

Run verification for the code you changed. The narrowest correct surface is
the right choice.

### Format and lint

```bash
# For any Rust change:
cargo fmt --check --all
cargo clippy --workspace --all-features -- -D warnings

# Or equivalently via mise (once installed by T8):
# mise run fmt
# mise run lints
```

Do not use `--all-features` in test tasks — it forces a heavy feature union
to recompile and defeats artifact reuse. Use `--all-features` only for
`clippy` and format checks so every code path is covered statically.

### Rust crate changes — fast lane (no database)

```bash
# All workspace unit tests (credential-free, Docker-free):
cargo test --workspace

# Single crate focused iteration:
cargo test --locked -p <crate> <test_name> -- --nocapture --test-threads=1
```

### Postgres-gated integration tests

Requires `docker compose up -d` (see `docker-compose.yml`):

```bash
# wyrd-sql-core migration runner tests:
cargo test --locked -p wyrd-sql-core --features pg -- --nocapture --test-threads=1

# wyrd-dev-fixtures harness tests:
cargo test --locked -p wyrd-dev-fixtures --features wyrd-dev-fixtures/pg -- --nocapture --test-threads=1

# Or via mise (once installed by T8):
# mise run test:sql
# mise run test:pg
```

Use `--test-threads=1` for Postgres-gated tests to avoid port and schema
collisions.

### Schema and contract drift checks

`wyrd-spec` generates JSON schemas. After any spec change:

```bash
# Verify schema goldens match source (via mise once installed):
# mise run codegen:check

# Or manually: regenerate and diff
cargo test --locked -p wyrd-spec -- --nocapture
```

Any schema golden drift that is not regenerated and committed is a CI
failure.

### Transport and auth crates (no database required)

```bash
cargo test --locked -p wyrd-tonic -- --nocapture
cargo test --locked -p wyrd-auth-verify -- --nocapture
cargo test --locked -p wyrd-client -- --nocapture
```

Note: `wyrd-auth-verify` unit tests that build a `reqwest`/rustls client
require `wyrd_tls::install_crypto_provider()` in test setup (installed by
T6). Run them through `cargo test --locked -p wyrd-auth-verify` after T6.

## Foundation Boundary Checks

The following gates apply directly to this repository and are enforced in CI:

| Gate | What it enforces | Foundation applicability |
|---|---|---|
| `check:client-tier` | Client-tier crates do not depend on `sqlx`, cloud SDKs, `datafusion`, or `iceberg` | APPLIES — enforced for all foundation crates except `wyrd-sql-core` and `wyrd-dev-fixtures` |
| `check:unwrap-audit` | Audits `unwrap()`/`expect()` outside tests | APPLIES |
| `check:fixtures-no-server` | `wyrd-dev-fixtures` does not import `wyrd-server` | APPLIES |
| `check:no-tonic-outside-wyrd-tonic` | Reject tonic-family deps outside `wyrd-tonic` + workspace pins | APPLIES |
| `check:proto-drift` | `wyrd.v1` FileDescriptorSet matches `.proto` | APPLIES — `wyrd-tonic` owns proto |

The following gates apply to the `wyrd` monorepo server/plane crates only —
included as **cross-plane doctrine** for contributors who work across both
repositories, not as runnable local checks:

| Gate | Cross-plane context only |
|---|---|
| `check:pyo3-scope` | PyO3 boundary (no PyO3 in foundation) |
| `check:tenant-isolation` | SQL tenant isolation (server-tier, not present here) |
| `check:registry-tx-coupling` | TenantConn tx coupling (server-tier) |
| `check:from-pools-allowlist` | Pool construction sites (server-tier) |
| `check:object-store-pin` | Single object-store/DataFusion/arrow versions (Vala plane) |
| `check:design-sync` | ValaQueryService permissions (Vala plane) |
| `check:tokens` | Theme CSS (UI plane) |
| `check:py-wheel-no-testing` | Production Python wheel (Python plane) |

## Test Design

- Exercise public or crate-visible behavior.
- Cover success, stable failures, and edge cases; assert Wyrd error codes.
- Use local fixtures. For Postgres, use `PgFixture` from `wyrd-dev-fixtures`
  (Postgres-gated tests only; the fast lane must remain credential- and
  Docker-free).
- Do not require credentials for unit tests.
- Keep generated output (schema goldens) drift-free.
- Do not broaden tests into slow integration gates unless the touched
  behavior requires it.

## Never Circumvent A Gate

Do not weaken or disable a check, add `#[allow]`, delete or `#[ignore]` a
failing test, or broaden a boundary glob to hide a real violation. Fix the
underlying cause. Only use a check's own sanctioned mechanism (e.g. the
documented per-file allowlist) when the usage is legitimately test-only
and matches existing in-pattern precedent.
