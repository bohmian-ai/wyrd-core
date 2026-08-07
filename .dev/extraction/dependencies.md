# T1 Extraction Dependency Ledger

Oracle SHA: `6b450184a7ca2a106f2d9bba5b813764dbdbf91e`
Source monorepo branch: `surfaces-python` @ `f25532692b29b2ef2e718ff89067923da6df1740`
Target workspace: `/Users/stevenforrester/Documents/GitHub/wyrd-core`
Workspace version: `0.1.0`, edition `2024`, MSRV `1.94.0`, toolchain `1.95.0`, Apache-2.0
Repository URL: `https://github.com/mitari-ai/wyrd-core`

## Crate Source Paths → Target Paths

| Crate | Oracle Source Path | Target Path |
|---|---|---|
| `wyrd-spec` | `crates/wyrd-spec` | `crates/wyrd-spec` |
| `skald-spec` | `crates/skald/skald-spec` | `crates/skald-spec` |
| `wyrd-tonic` | `crates/wyrd/wyrd-tonic` | `crates/wyrd-tonic` |
| `wyrd-client` | `crates/shared/wyrd-client` | `crates/wyrd-client` |
| `wyrd-utils` | `crates/shared/wyrd-utils` | `crates/wyrd-utils` |
| `wyrd-runtime` | `crates/shared/wyrd-runtime` | `crates/wyrd-runtime` |
| `wyrd-queue` | `crates/shared/wyrd-queue` | `crates/wyrd-queue` |
| `wyrd-semver` | `crates/shared/wyrd-semver` | `crates/wyrd-semver` |
| `wyrd-version` | `crates/shared/wyrd-version` | `crates/wyrd-version` |
| `wyrd-telemetry` | `crates/shared/wyrd-telemetry` | `crates/wyrd-telemetry` |
| `wyrd-crypt` | `crates/shared/wyrd-crypt` | `crates/wyrd-crypt` |
| `wyrd-error-derive` | `crates/shared/wyrd-error-derive` | `crates/wyrd-error-derive` |
| `wyrd-test-contract-macros` | `crates/shared/wyrd-test-contract-macros` | `crates/wyrd-test-contract-macros` |
| `wyrd-dev-fixtures` | `crates/shared/wyrd-dev-fixtures` | `crates/wyrd-dev-fixtures` |
| `wyrd-auth-verify` | `crates/shared/wyrd-auth-verify` | `crates/wyrd-auth-verify` |
| `wyrd-auth-issue` | `crates/shared/wyrd-auth-issue` | `crates/wyrd-auth-issue` |
| `wyrd-auth-check` | `crates/shared/wyrd-auth-check` | `crates/wyrd-auth-check` |
| `wyrd-auth-oidc` | `crates/shared/wyrd-auth-oidc` | `crates/wyrd-auth-oidc` |
| `wyrd-tls` | `crates/shared/wyrd-tls` | `crates/wyrd-tls` |

**Note:** `wyrd-sql-core` (the 21st D1 member) is reserved for T2 and not present in this snapshot. `wyrd-bench` was removed by T13 (D14 — Bifrost-only, no foundation consumer).

## Crate Descriptions Added

The following 15 crates lacked a description in the oracle source. Descriptions
are factually derived from each crate's `src/lib.rs` module-level rustdoc and/or
README content at the oracle SHA.

| Crate | Added Description |
|---|---|
| `skald-spec` | "Native provider type set for Skald: request, response, message, and wire contracts shared end to end." |
| `wyrd-tonic` | "Wyrd-owned tonic facade: single pin point for all tonic-family versions and generated gRPC stubs." |
| `wyrd-utils` | "Small shared helpers for Wyrd with no server dependencies: codec, filesystem, JSON utilities, and optional Python boundary helpers." |
| `wyrd-runtime` | "Runtime singleton and cross-cutting behavior shells for Wyrd: request context, principal, RBAC, audit, and OTEL." |
| `wyrd-version` | "Shared version primitives for Wyrd: WyrdVersion newtype over semver::Version with serde, schemars, and bump helpers." |
| `wyrd-telemetry` | "Telemetry setup shell for Wyrd: tracing-subscriber configuration, OTLP export, and test-support subscriber." |
| `wyrd-crypt` | "AES-256-GCM encryption helpers for Wyrd: sealing key derivation, encrypt/decrypt, and zero-on-drop secret material." |
| `wyrd-error-derive` | "Proc-macro derive for Wyrd stable error-code enums: WyrdError derive with code, status, title, and remediation metadata." |
| `wyrd-test-contract-macros` | "Attribute macros for declaring test-critical Wyrd contracts that CI checks against pytest coverage markers." |
| `wyrd-dev-fixtures` | "Plane-neutral development fixture primitives for Wyrd SQL integration tests." |
| `wyrd-auth-verify` | "Verify-only authentication helpers for Wyrd: JWT decoding, JWKS caching, bearer-token verification, and principal extraction." |
| `wyrd-auth-issue` | "Server-tier authentication issuance helpers for Wyrd: JWT signing, key management, password hashing, and token generation." |
| `wyrd-auth-check` | "Shared authz-check mechanism primitives for Wyrd: policy hook trait, guard, and request/response types." |
| `wyrd-auth-oidc` | "Generic OIDC verification toolkit for Wyrd: provider discovery, JWKS caching, trusted-issuer registry, and claim mapping." |
| `wyrd-tls` | "Process-wide TLS crypto-provider ownership for Wyrd transports." |

## Internal Dependency Normalization

The following hardcoded path+version entries in oracle source were converted to
workspace inheritance (`{ workspace = true }`) with workspace path+version entries
at `0.1.0`:

| Crate | Field Changed | Old Value | New Value |
|---|---|---|---|
| `wyrd-runtime` | `wyrd-spec` dep | `{ path = "../../wyrd-spec", version = "0.0.1" }` | `{ workspace = true }` |
| `wyrd-runtime` | `wyrd-semver` dev-dep | `{ path = "../wyrd-semver", version = "0.0.1" }` | `{ workspace = true }` |
| `wyrd-telemetry` | `wyrd-spec` dep | `{ path = "../../wyrd-spec", version = "0.0.1" }` | `{ workspace = true }` |
| `wyrd-auth-oidc` | `wyrd-spec` dep | `{ path = "../../wyrd-spec", version = "0.0.1" }` | `{ workspace = true }` |
| `wyrd-auth-verify` | `wyrd-auth-oidc` dep | `{ path = "../wyrd-auth-oidc", version = "0.0.1" }` | `{ workspace = true }` |
| `wyrd-auth-verify` | `wyrd-spec` dep | `{ path = "../../wyrd-spec", version = "0.0.1" }` | `{ workspace = true }` |
| `wyrd-auth-verify` | `wyrd-semver` dev-dep | `{ path = "../wyrd-semver", version = "0.0.1" }` | `{ workspace = true }` |
| `wyrd-auth-issue` | `wyrd-spec` dep | `{ path = "../../wyrd-spec", version = "0.0.1" }` | `{ workspace = true }` |
| `wyrd-auth-issue` | `wyrd-semver` dev-dep | `{ path = "../wyrd-semver", version = "0.0.1" }` | `{ workspace = true }` |
| `wyrd-auth-check` | `wyrd-spec` dep | `{ path = "../../wyrd-spec", version = "0.0.1" }` | `{ workspace = true }` |
| `wyrd-auth-check` | `wyrd-semver` dev-dep | `{ path = "../wyrd-semver", version = "0.0.1" }` | `{ workspace = true }` |

## Plane Dependencies Removed

The following out-of-cone (plane) dependencies were removed during T1 normalization.
These will be replaced in T2 (`wyrd-sql-core`) and T3 (`wyrd-dev-fixtures` rebuild):

| Crate | Removed Dep | Reason |
|---|---|---|
| `wyrd-dev-fixtures` | `vala-sql` (plane) | Outside D1; crate rebuilt in T3 with `wyrd-sql-core` |
| `wyrd-dev-fixtures` | `wyrd-sql` (plane) | Outside D1; crate rebuilt in T3 with `wyrd-sql-core` |
| `wyrd-client` | `wyrd-testing` (plane, dev-dep) | Removed per D4; PG journeys removed |

## Source Files Removed

| Crate | File | Reason |
|---|---|---|
| `wyrd-dev-fixtures` | `src/pg.rs` | Imports `vala_sql`, `wyrd_sql`; rebuilt in T3 |
| `wyrd-dev-fixtures` | `src/cards.rs` | Imports `wyrd_sql::TenantConn`; rebuilt in T3 |
| `wyrd-client` | `tests/pg_auth_e2e_against_fixture.rs` | Uses `wyrd_testing::WyrdTestServer`; removed per D4 |
| `wyrd-client` | `tests/pg_discovery_against_fixture.rs` | Uses `wyrd_testing::WyrdTestServer`; removed per D4 |

## Package Field Normalizations

| Crate | Field | Old Value | New Value |
|---|---|---|---|
| `skald-spec` | `version` | `"0.1.0"` (hardcoded) | `{ workspace = true }` |
| `skald-spec` | `edition/license/etc.` | `field.workspace = true` | `field = { workspace = true }` |
| `skald-spec` | `publish = false` | present | removed |
| `wyrd-tonic` | `version` | `"0.0.1"` (hardcoded) | `{ workspace = true }` |
| `wyrd-tonic` | `edition` | `"2021"` (hardcoded) | `{ workspace = true }` |
| `wyrd-tonic` | `license/rust-version/repository` | absent | `{ workspace = true }` |
| `wyrd-tls` | `version` | `"0.0.1"` (hardcoded) | `{ workspace = true }` |
| `wyrd-tls` | `edition` | `"2021"` (hardcoded) | `{ workspace = true }` |
| `wyrd-tls` | `license/rust-version/repository` | absent | `{ workspace = true }` |
| `wyrd-client` | `publish = false` | present | removed |
| `wyrd-queue` | `publish = false` | present | removed |
