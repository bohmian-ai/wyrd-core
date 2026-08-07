# Faithful-port audit — foundation vs surfaces vs oracle

Date: 2026-08-06. Orchestrator: `$wyrd-implement-plan` (Opus).
Method: git blob-SHA comparison (content hashes are identical across repos for
identical bytes, so no checkout/extract needed). Script:
`.dev/extraction/faithful-port-audit.py` (reproducible).

- Foundation: working tree of `wyrd-core` (branch `stack/04-spec-replay`
  @ `a2041f0` + uncommitted T5).
- Surfaces target: `wyrd` monorepo `f2553269` (surfaces-python).
- Oracle: `wyrd` monorepo `6b450184` (== `wyrd-bifrost-oracle`, bifrost SoT).

## Reconciliation policy (D11/D12, user-locked 2026-08-06)

1. **Surfaces (`f2553269`) is the base for the entire foundation; surfaces wins
   for every non-bifrost file.**
2. **Oracle (`6b450184`) is authoritative for bifrost logic**, whose foundation
   footprint is the `wyrd-spec::vala` **bifrost** files (`api`, `audit_detail`,
   `error`, `managed_columns`, `mod`, `trace/mod`, `system_columns` deletion) +
   `wyrd-tonic` query-transport (`query_conversion.rs`, `private_conversion.rs`,
   `frame_codec.rs`, `transport.rs`) + `wyrd-tls`. **Not** oracle-authoritative:
   `wyrd-spec::vala::eval/*` + `observation.rs` are surfaces-owned `Eval`/
   observation Card contracts (D12a) — surfaces advanced them, oracle did not.
3. Same-file bifrost overlaps reconcile case-by-case (oracle authoritative for
   bifrost semantics).
4. Every oracle-only addition in a non-bifrost crate is surfaced to the user,
   never silently retained.
4a. **D13 (plan v6):** "surfaces wins" is the *full* rule "surfaces base **+
   reconcile oracle additions**", applied per file via three-way analysis (merge
   base `f58ec630`). Group A = surfaces-only-advanced → flip surfaces. Group B =
   oracle-only-advanced (surfaces == merge base) → keep oracle, because
   surfaces-base-plus-oracle-additions *equals* oracle's bytes; a blanket flip
   would discard the oracle addition. Group C = both-advanced → union (surfaces
   wins non-bifrost, oracle additions retained). Retained oracle additions FLAGged.
5. Editions match source per crate (`wyrd-tls`, `wyrd-tonic` = 2021).
6. `wyrd-bench` REMOVED (D14 — Bifrost-only, no foundation consumer).

## Per-crate disposition

| Crate | Audit | Disposition | Owner task |
|---|---|---|---|
| skald-spec | code-faithful (only Cargo.toml scaffolding differs) | none | — |
| wyrd-auth-oidc | code-faithful | none | — |
| wyrd-crypt | code-faithful | none | — |
| wyrd-error-derive | code-faithful | none | — |
| wyrd-test-contract-macros | code-faithful | none | — |
| wyrd-version | code-faithful | none | — |
| wyrd-auth-check | 3-way: 4 files surfaces-only-advanced | **Group A** FLIP to surfaces (context, guard, hook, request) | T11 |
| wyrd-auth-issue | 3-way: both-advanced (surfaces +AuditSeal+CardRef, oracle only *deleted* AuditSeal) | **Group C** take surfaces wholesale (surfaces wins non-bifrost audit; no oracle addition to merge) | T11 |
| wyrd-auth-verify | 3-way: both-advanced (surfaces 2 CardRef compat, oracle +verify_access_token/DataTenantId sentinel) | **Group C** oracle base + 2 surfaces CardRef `Some(space)` edits (keep oracle tenant-id security addition) — FLAG | T11 |
| wyrd-queue | 3-way: 3 files oracle-only-advanced (`has_pending`) | **Group B** KEEP oracle (surfaces==MB; flip would discard oracle addition) — FLAG | T11 |
| wyrd-runtime | 3-way: permission oracle-only-advanced (`BifrostOraclePeer`, bifrost); principal+request_context surfaces-only-advanced | **Group B** keep oracle permission (FLAG) + **Group A** flip principal, request_context | T11 |
| wyrd-semver | 3-way: spec surfaces-only-advanced | **Group A** FLIP to surfaces | T11 |
| wyrd-telemetry | 3-way: lib oracle-only-advanced (otel 0.31 + test-support + wyrd_tls) | **Group B** KEEP oracle (foundation dep baseline D12) — FLAG | T11 |
| wyrd-utils | 3-way: lib surfaces-only-advanced + dropped `config_dir.rs` | **Group A** FLIP lib + ADD config_dir.rs (surfaces bytes; stale copy has non-faithful rustdoc) | T11 |
| wyrd-client | 3-way split (merge base `f58ec630`): surfaces-only `config.rs`/`lib.rs`/`transport/config.rs`/`transport/credential.rs` → take surfaces; oracle-only `transport/grpc.rs`/`transport/http.rs`/`tests/transport/http.rs` → KEEP+FLAG (client streaming, D12); both-changed disjoint `auth.rs`/`client.rs`/`error.rs` → union (surfaces priority); dropped `global_config.rs` → restore; `tests/pg_*_against_fixture.rs` use `wyrd_testing` → cut+ledger | T5 |
| wyrd-spec | vala **bifrost** files (`api,audit_detail,error,managed_columns,mod,trace/mod`) + vala schemas =oracle (KEEP); vala **Eval/observation Card contracts** (`eval/{spec,mod,llm_judge,record}.rs`,`observation.rs`) =surfaces (surfaces-only advance; oracle==merge-base — see D12a); 35 non-vala surfaces files replayed; 40+40 goldens regen T6; `system_columns.rs` oracle-deleted (KEEP deleted) | already correct (T4) | T4 (verify) |
| wyrd-tonic | 6 files =oracle + 4 oracle-only (frame_codec, private_conversion, query_conversion, transport) | KEEP oracle bifrost stack (D12); reconcile non-bifrost error/health/lib/server against surfaces case-by-case | T12 |
| wyrd-tls | oracle-only crate (absent in surfaces) | KEEP (bifrost transport dep, D12) | T12 |
| wyrd-bench | oracle-only crate (absent in surfaces) | REMOVED (D14 — Bifrost-only, no foundation consumer) | T13 |
| wyrd-sql-core | new crate (no source in either) | legitimate new extraction (T2) | — |
| wyrd-dev-fixtures | reconciled by T3 (lib/pg); dropped surfaces `src/cards.rs`; added 2 novel test-migration SQLs | verify T3's cards.rs drop is intentional plane-neutral design | T3 (verify) |

Note: nearly every crate's `Cargo.toml` differs from surfaces — this is
legitimate T1 workspace-inheritance normalization (version/edition/license/
repository inherited from `[workspace.package]`), NOT a logic divergence, and is
excluded from the flip except the `edition` field (must match source per crate).

## vala full 3-way verification (all 46 files, `vala-3way-check.py`)

Complete symmetric-miss sweep of `wyrd-spec/src/vala/**` against surfaces,
oracle, and merge base `f58ec630`. Result: **0 defects, 0 reconcile cases.**
- Oracle-advanced bifrost files shipped oracle: `api`, `audit_detail`, `error`,
  `managed_columns`, `mod`, `trace/mod`; `system_columns.rs` oracle-deleted (absent). ✓
- Surfaces-advanced Card contracts shipped surfaces: `eval/{spec,mod,llm_judge,record}.rs`,
  `observation.rs` (oracle == merge base for all five). ✓
- All remaining vala files: neither stream changed them vs merge base, so the
  surfaces and oracle blobs are identical — shipping surfaces == shipping oracle. ✓
No file was surfaces-advanced-but-shipped-oracle, and none was changed by both
streams. The accepted T4 vala tree is fully consistent with D3/D11/D12/D12a.

Full per-file output: see `faithful-port-audit.txt` alongside this file.

## T12 verification confirmation (2026-08-06)

Executed blob-SHA fidelity check for all T12-scope files against oracle `6b450184`.
All 11 wyrd-tonic files and 1 wyrd-tls file are byte-identical to oracle (wyrd-bench verified at T12 time; removed by T13/D14).
Edition pins: `wyrd-tonic` = "2021", `wyrd-tls` = "2021" — all match source.
`cargo check -p wyrd-tonic` (default features, no `server` feature, axum-free): PASS (25 deps compiled).
`cargo check -p wyrd-tls`: PASS.
`cargo test --locked -p wyrd-tonic --lib`: 31 PASSED (bifrost conversion + frame-codec unit tests).
`cargo fmt -p wyrd-tonic -p wyrd-tls -- --check`: CLEAN.
No surfaces content introduced; no bifrost module dropped. D12 disposition confirmed.

## T11 execution confirmation (2026-08-06)

Per-file three-way reconcile executed per D13/packet. All 15 files verified:

**Group A (flip to surfaces f2553269) — 10 files PASS:**
wyrd-auth-check/{context,guard,hook,request}.rs, wyrd-runtime/{principal,request_context}.rs,
wyrd-semver/spec.rs, wyrd-utils/lib.rs, wyrd-auth-issue/lib.rs (Group C #9),
wyrd-utils/src/config_dir.rs (added from surfaces).

**Group B (restored to oracle HEAD b154c52) — 5 files PASS:**
wyrd-queue/{producer,queue,sink}.rs, wyrd-runtime/permission.rs, wyrd-telemetry/lib.rs.
All oracle-only additions confirmed: `has_pending`, `BifrostOraclePeer`/`bifrost_oracle_peer_invoke`,
otel-0.31/`wyrd_tls::install_crypto_provider()` wiring.

**Group C #10 (oracle base + 2 CardRef compat edits) — wyrd-auth-verify/lib.rs PASS:**
Restored to HEAD, applied exactly two `space: Some(...)` substitutions. Oracle additions retained:
`verify_access_token`, `DataTenantId` sentinel, `replace_system_owner_tenant_ids`,
`restore_system_owner_tenant_ids`.

`cargo check -p <crate>` (9 crates, default features): ALL PASS.
`cargo test -p <crate> --lib` (9 crates): ALL PASS (wyrd-auth-verify: 32/39 pass; 7 `verify_external_*`
tests fail on missing rustls crypto provider — pre-existing baseline issue, no crypto-provider
setup in either oracle or surfaces source; confirmed not caused by our diff).
`cargo fmt -p <crate> -- --check` (9 crates): ALL PASS. No Cargo.toml changes. No dependency added.
