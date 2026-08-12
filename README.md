# wyrd-core

`wyrd-core` is the Rust foundation workspace for [Wyrd](https://github.com/mitari-ai/wyrd). It defines contracts and shared infrastructure used by Wyrd services and clients.

It contains 20 crates for cards and schemas, transport, authentication, telemetry, cryptography, versioning, queues, TLS, and Postgres migrations.

This repository does not contain the Wyrd server, UI, CLI or MCP host, Skald runtime, Vala/Bifrost plane, or language SDKs.

## Crates

| Area | Crates |
|---|---|
| Contracts | `wyrd-spec`, `skald-spec` |
| Transport | `wyrd-tonic`, `wyrd-client` |
| Authentication | `wyrd-auth-issue`, `wyrd-auth-verify`, `wyrd-auth-check`, `wyrd-auth-oidc` |
| Data and runtime primitives | `wyrd-sql-core`, `wyrd-dev-fixtures`, `wyrd-runtime`, `wyrd-queue`, `wyrd-utils` |
| Versioning | `wyrd-semver`, `wyrd-version` |
| Security and observability | `wyrd-crypt`, `wyrd-tls`, `wyrd-telemetry` |
| Test and error support | `wyrd-error-derive`, `wyrd-test-contract-macros` |

`wyrd-spec` is the durable contract layer. It contains typed Card envelopes, specs, identifiers, validation, schemas, and stable public errors. It is IO-free, async-free, and PyO3-free.

## Requirements

- Rust, installed through `rustup`; the repository selects its pinned toolchain from [`rust-toolchain.toml`](rust-toolchain.toml)
- Docker and the PostgreSQL client tools for Postgres-backed integration tests

## Build and test

Run the fast workspace build and test suite:

```console
cargo build --workspace
cargo test --workspace
```

Run the Postgres-backed integration tests:

```console
bash scripts/postgres/with-test-postgres.sh -- \
  cargo test --locked -p wyrd-sql-core -p wyrd-dev-fixtures \
  --features pg -- --test-threads=1
```

The wrapper starts an isolated Postgres container, configures test roles and connection variables, runs the command, and removes the container afterward.

For formatting, linting, generated-schema checks, and the complete local validation workflow, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Design

Wyrd models registered AI-system components as Cards with a shared envelope:

```yaml
apiVersion: wyrd/v1
metadata:
  name: churn-model
  version: "1.2.0"
kind: Model
spec:
  # kind-specific declaration
relationships: []
status: null
```

Cards declare intent. Specs provide kind-specific meaning. `CardRef` values connect cards, while the server derives relationships and manages status.

The active protocol authority is [architecture/wyrd-design.md](architecture/wyrd-design.md). Read [architecture/wyrd-doctrine.mdx](architecture/wyrd-doctrine.mdx) for the broader design rationale.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for local setup and validation requirements. Repository-wide contributor rules are in [AGENTS.md](AGENTS.md).

## License

This project is licensed under the [Apache License, Version 2.0](LICENSE).
