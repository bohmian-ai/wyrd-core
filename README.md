# wyrd-foundation

Foundation crates for the [Wyrd](https://github.com/mitari-ai/wyrd) AI platform.
Includes pure contracts (`wyrd-spec`, `skald-spec`), shared primitives
(auth, telemetry, crypto, queue, semver, runtime), SQL migration core, and gRPC
transport (`wyrd-tonic`). These crates publish as standalone Apache-2.0 libraries.

## Crates

| Crate | Purpose |
|---|---|
| `wyrd-spec` | Card envelope, specs, schemas, identifiers, stable error catalog |
| `skald-spec` | Skald provider/capability specs |
| `wyrd-tonic` | gRPC/protobuf transport layer |
| `wyrd-client` | HTTP + gRPC client for the Wyrd server |
| `wyrd-utils` | Shared primitives and helpers |
| `wyrd-runtime` | Async runtime boundary helpers |
| `wyrd-queue` | Task queue primitives |
| `wyrd-semver` | Semantic versioning types |
| `wyrd-version` | Version identity |
| `wyrd-telemetry` | OpenTelemetry integration |
| `wyrd-crypt` | Cryptographic primitives |
| `wyrd-auth-issue` | Token issuance |
| `wyrd-auth-verify` | Token verification |
| `wyrd-auth-check` | Authorization checks |
| `wyrd-auth-oidc` | OIDC integration |
| `wyrd-tls` | TLS configuration |
| `wyrd-sql-core` | Postgres migration runner and pool primitives |
| `wyrd-dev-fixtures` | Test fixture helpers (dev/test only) |
| `wyrd-error-derive` | `WyrdError` derive macro |
| `wyrd-test-contract-macros` | Contract test macros |

## License

Apache-2.0. See [LICENSE](LICENSE) for details.

## Requirements

- Rust stable (see `rust-toolchain.toml` for the pinned version)
- Docker (for Postgres-backed tests only)

## Development

```console
cargo build --workspace
cargo test --workspace
```

Postgres-backed integration tests require a running Postgres instance. See
`docker-compose.yml` to start one locally:

```console
docker compose up -d
```

Then run the Postgres-gated test suite:

```console
cargo test -p wyrd-dev-fixtures --features pg
```

See `CONTRIBUTING.md` for detailed setup and contribution guidelines.
