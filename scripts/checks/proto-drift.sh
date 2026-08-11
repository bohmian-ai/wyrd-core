#!/usr/bin/env bash
# WHY THIS FILE EXISTS: the checked-in crates/wyrd-tonic/proto/wyrd.v1.bin
# FileDescriptorSet is used for gRPC server reflection and tooling. If the
# .proto changes without regenerating the snapshot, the reflection API and
# the actual service diverge: clients see stale field names/types and the
# mismatch is invisible until a client call fails at runtime. Catching drift
# at CI time costs one build; catching it in production costs an incident.
#
# WHAT IT CHECKS: captures the committed descriptor bytes BEFORE regeneration,
# forces a fresh build.rs run with WYRD_PROTO_SNAPSHOT=1 (which overwrites
# wyrd.v1.bin in place), then compares the pre-rebuild committed bytes against
# the freshly generated output. On a clean tree they are identical; if the
# .proto has changed without a corresponding committed .bin update, they differ.
#
# WYRD_PROTO_COMMITTED_SNAPSHOT: absolute path to an alternative pre-rebuild
# snapshot file. When set, the check uses that file as the committed baseline
# instead of capturing crates/wyrd-tonic/proto/wyrd.v1.bin before regeneration.
# Used by the negative self-test harness: provide a mutated .bin so the check
# sees committed≠regenerated and exits non-zero, proving the gate is live.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

CANONICAL_GENERATED="crates/wyrd-tonic/proto/wyrd.v1.bin"

echo "proto-drift: capturing current descriptor snapshot..."
if [ -n "${WYRD_PROTO_COMMITTED_SNAPSHOT:-}" ]; then
    # Negative self-test path: caller supplies a pre-mutated snapshot.
    BEFORE_COPY="$WYRD_PROTO_COMMITTED_SNAPSHOT"
    _before_tmp=false
else
    # Production path: copy the on-disk (committed) bytes BEFORE regeneration
    # overwrites them. On a clean tree the on-disk file is the committed snapshot.
    BEFORE_COPY="$(mktemp)"
    _before_tmp=true
    cp "$CANONICAL_GENERATED" "$BEFORE_COPY"
fi

echo "proto-drift: rebuilding wyrd-tonic to regenerate descriptor snapshot..."
# Touching build.rs forces cargo to re-run it even with a warm target cache.
touch crates/wyrd-tonic/build.rs
WYRD_PROTO_SNAPSHOT=1 cargo build --locked -p wyrd-tonic --all-features

echo "proto-drift: checking for snapshot drift..."
if ! cmp -s "$BEFORE_COPY" "$CANONICAL_GENERATED"; then
    echo "FAIL proto-drift: wyrd.v1 descriptor drift detected"
    echo "  The .proto source produced a different descriptor from the committed .bin."
    echo "  Run:"
    echo "    WYRD_PROTO_SNAPSHOT=1 cargo build --locked -p wyrd-tonic --all-features"
    echo "  and commit crates/wyrd-tonic/proto/wyrd.v1.bin"
    [ "$_before_tmp" = "true" ] && rm -f "$BEFORE_COPY"
    exit 1
fi

[ "$_before_tmp" = "true" ] && rm -f "$BEFORE_COPY"
echo "proto-drift: passed -- descriptor snapshot is current with .proto source"
