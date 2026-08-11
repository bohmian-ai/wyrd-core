# Contributing to wyrd-core

## Welcome

Thanks for contributing. This guide covers local setup, validation commands, and
review expectations for this repository.

## Table of Contents

- [Contributing to wyrd-core](#contributing-to-wyrd-core)
  - [Welcome](#welcome)
  - [Table of Contents](#table-of-contents)
  - [Environment Setup](#environment-setup)
  - [Contributing Changes](#contributing-changes)
  - [Coding Conventions](#coding-conventions)
  - [Git](#git)

## Environment Setup

This repository is a standalone Rust workspace. No SvelteKit frontend, Python
SDK, or server plane is included — those are separate consuming repositories.

Ensure Rust is installed at the toolchain version pinned in `rust-toolchain.toml`
(the toolchain file installs it automatically via `rustup`).

For Postgres-backed tests, start the local database:

```console
docker compose up -d
```

Run the workspace build and fast tests:

```console
cargo build --workspace
cargo test --workspace
```

Run Postgres-gated integration tests (requires `docker compose up -d`):

```console
cargo test -p wyrd-dev-fixtures --features pg
```

## Contributing Changes

1. Create a branch for your change.
2. Make the smallest coherent change that satisfies the issue.
3. Run format and lint checks from the repository root:

```console
cargo fmt --check --all
cargo clippy --workspace --all-features -- -D warnings
```

4. Run the workspace tests:

```console
cargo test --workspace
```

5. Open a pull request after local validation passes.

Both CI workflows (`lints-test`) must be green before merge.

## Coding Conventions

- Rust edition `2024`. Workspace deps pin exact major.minor minimum; pre-1.0
  deps pin exact `x.y.z`. No wildcards.
- Library errors use `thiserror`. Binary errors use `anyhow`.
- Public Spec enums and structs are `#[non_exhaustive]`.
- Every wire type derives `schemars::JsonSchema`.
- Client-tier crates may not depend on `sqlx`, `datafusion`, `deltalake`, or
  cloud SDKs. Enforced by `check:client-tier` in CI.
- PyO3 is not present in this repository. `wyrd-spec` and all foundation crates
  are strictly PyO3-free.

### SQLx Allowlist

`sqlx` is permitted only in:

- `crates/wyrd-sql-core` — migration runner and pool construction
- `crates/wyrd-dev-fixtures` — test fixture helpers

No other crate in this workspace may depend on `sqlx`. This is enforced in CI.

### Configuration Secrets

Any `Config` type that carries secrets uses `secrecy::SecretString` with a
custom `Debug` impl that redacts the secret. Plain `String` for secrets is
forbidden.

### Error Code Format

All structured errors carry a stable code in the form
`WYRD_<DOMAIN>_<STATUS>_<SLUG>` (for example `WYRD_REGISTRY_404_CARD_NOT_FOUND`).

## Git

- Identity: `Thorrester <sjforrester32@gmail.com>`.
- No `Co-Authored-By` trailers.
- Branch names: `<short-slug>` for feature branches.
