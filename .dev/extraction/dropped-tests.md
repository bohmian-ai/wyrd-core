# Dropped Tests Ledger

Tests removed from `wyrd-core` that require excluded dependencies and
must be re-homed as user-journey tests in the source monorepo or its plane
repositories.

## `wyrd-client` PG Journey Tests

### `tests/pg_auth_e2e_against_fixture.rs`

**Path (Oracle source):** `crates/shared/wyrd-client/tests/pg_auth_e2e_against_fixture.rs`

**What it covered:** End-to-end authentication flow against a live Postgres-backed
Wyrd server fixture. The test exercised the full `wyrd-client` auth path — API
key credential resolution, token exchange against `/auth/token`, and bearer
injection on a protected endpoint — using `wyrd-testing`'s `WyrdTestServer`
harness which spins up an embedded Postgres instance with migrations applied.

**Why dropped:** The test depends on `wyrd-testing` (a `dev-dependency` in the
Oracle `wyrd-client` manifest). `wyrd-testing` is a plane crate that depends on
`wyrd-server`, `sqlx`, and the embedded Postgres machinery; it is not a
foundation crate and is explicitly excluded from `wyrd-core` (see T5
prohibited changes and D4). Adding `wyrd-testing` to the foundation would
import the full server plane, violating R1 (no plane-owned dependency).

**Re-homing requirement:** This test must be re-authored as a user-journey test
in the source monorepo (`crates/shared/wyrd-client/tests/`) or in whichever
plane repository owns server-level integration testing. It should use the
existing `WyrdTestServer` harness from `wyrd-testing` and continue to drive
the real `wyrd-client` against a real server. This is a T1 carry-forward item.

---

### `tests/pg_discovery_against_fixture.rs`

**Path (Oracle source):** `crates/shared/wyrd-client/tests/pg_discovery_against_fixture.rs`

**What it covered:** Card discovery (list/get) across the gRPC and HTTP transports
using a live Postgres-backed Wyrd server fixture. The test populated the
registry via the server and then exercised `wyrd-client` discovery methods,
verifying that results are consistent across both transport planes.

**Why dropped:** Same as above — depends on `wyrd-testing` and its
`WyrdTestServer` harness, which requires `wyrd-server`, `sqlx`, and embedded
Postgres. These dependencies are plane-owned and excluded from the foundation.

**Re-homing requirement:** This test must be re-authored as a user-journey test
in the source monorepo or the owning plane repository. The discovery contract
(client → server → client, both transports) is exactly the kind of
client→server→client coverage that belongs in the Tier 1 user-journey lane
(see AGENTS.md §11 Testing Workflow). This is a T1 carry-forward item.

---

## T5 Confirmation

T5 (wyrd-client reconcile) confirmed both PG journey tests were absent from
the foundation at `stack/05-client-reconcile` HEAD (T1 already omitted them).
The `wyrd-testing` dev-dependency has been removed from
`crates/wyrd-client/Cargo.toml`; it was present in both source streams but is
excluded from the foundation per D4/R1.
