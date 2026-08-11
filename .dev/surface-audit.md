# wyrd-core public-API surface audit

Generated: 2026-08-06
Task: T6 — Regenerate and baseline public surface
Branch: stack/06-generated-baseline
Diff base: c9a282d (T5 accepted)

## Purpose

This audit records the initial public-API baseline for every foundation crate
at the time T6 regenerated schema goldens and stabilized the workspace. It
does **not** redesign any API. All public items recorded here are preserved
unchanged from the reconciled source (D2).

## Method

Each crate's public surface was captured via:

```sh
cargo public-api -sss -p <crate> > .dev/public-api/<crate>.txt
```

`cargo-public-api 0.51.0` was used. The `-sss` flag silences build output.
Output files are checked in alongside this document.

## Crate surfaces

| Crate | Public items (approx) | Baseline file |
|---|---|---|
| `skald-spec` | ~1 747 lines | [skald-spec.txt](public-api/skald-spec.txt) |
| `wyrd-auth-check` | ~304 lines | [wyrd-auth-check.txt](public-api/wyrd-auth-check.txt) |
| `wyrd-auth-issue` | ~119 lines | [wyrd-auth-issue.txt](public-api/wyrd-auth-issue.txt) |
| `wyrd-auth-oidc` | ~228 lines | [wyrd-auth-oidc.txt](public-api/wyrd-auth-oidc.txt) |
| `wyrd-auth-verify` | ~113 lines | [wyrd-auth-verify.txt](public-api/wyrd-auth-verify.txt) |
| `wyrd-client` | ~397 lines | [wyrd-client.txt](public-api/wyrd-client.txt) |
| `wyrd-crypt` | ~42 lines | [wyrd-crypt.txt](public-api/wyrd-crypt.txt) |
| `wyrd-dev-fixtures` | ~19 lines | [wyrd-dev-fixtures.txt](public-api/wyrd-dev-fixtures.txt) |
| `wyrd-error-derive` | ~8 lines | [wyrd-error-derive.txt](public-api/wyrd-error-derive.txt) |
| `wyrd-queue` | ~237 lines | [wyrd-queue.txt](public-api/wyrd-queue.txt) |
| `wyrd-runtime` | ~400 lines | [wyrd-runtime.txt](public-api/wyrd-runtime.txt) |
| `wyrd-semver` | ~112 lines | [wyrd-semver.txt](public-api/wyrd-semver.txt) |
| `wyrd-spec` | ~7 007 lines | [wyrd-spec.txt](public-api/wyrd-spec.txt) |
| `wyrd-sql-core` | ~372 lines | [wyrd-sql-core.txt](public-api/wyrd-sql-core.txt) |
| `wyrd-telemetry` | ~57 lines | [wyrd-telemetry.txt](public-api/wyrd-telemetry.txt) |
| `wyrd-test-contract-macros` | ~7 lines | [wyrd-test-contract-macros.txt](public-api/wyrd-test-contract-macros.txt) |
| `wyrd-tls` | ~14 lines | [wyrd-tls.txt](public-api/wyrd-tls.txt) |
| `wyrd-tonic` | ~1 109 lines | [wyrd-tonic.txt](public-api/wyrd-tonic.txt) |
| `wyrd-utils` | ~40 lines | [wyrd-utils.txt](public-api/wyrd-utils.txt) |
| `wyrd-version` | ~69 lines | [wyrd-version.txt](public-api/wyrd-version.txt) |

## Intentional preservation

All public items in the above baseline are preserved from the reconciled
source streams (surfaces-python `f2553269` base with oracle `6b450184`
authoritative for bifrost, per D11/D12). No `pub` visibility was demoted
during T6 or any prior T1–T5/T11/T12 task. The audit baseline is a record,
not a redesign.

## Stale kinds absent (AC2 — Source/stale-kind)

The following stale v1 kinds are absent from the foundation:

- `ToolSpec` — not a Card kind (Tool is a Skald runtime concept)
- `SkillSpec` — not a v1 Card kind
- `SubAgentSpec` — sub-agency is an Agent-to-Agent relationship, not a Card kind

`SourceSpec` is present in `wyrd_spec::card::source::SourceSpec` as a v1 Card
kind, confirmed by the `wyrd-spec.txt` baseline and the 834-test suite passing.

## Contract stream notes

Both contract streams (surfaces-python `f2553269` and oracle `6b450184`) are
reconciled per D3/D4/D11/D12/D13:

- `wyrd-spec`: non-bifrost from surfaces, bifrost vala files from oracle (D3/D12a)
- `wyrd-client`: surfaces-priority 3-way reconcile (D4/D11)
- `wyrd-tonic` + `wyrd-tls`: oracle bifrost transport stack (D12)
- 9 non-bifrost crates: per-file Group A/B/C disposition (D13/T11)

Both streams compile and all drift/security tests pass (834 wyrd-spec + 119
wyrd-client + 39 wyrd-auth-verify = 992 tests confirmed green at T6).
