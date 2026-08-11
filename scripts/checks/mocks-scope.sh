#!/usr/bin/env bash
# WHY THIS FILE EXISTS: mock libraries (mockall, wiremock, mockito) are
# test-only tools. If they appear in production source — even behind a
# #[cfg(test)] block — they become non-optional transitive deps for every
# downstream consumer, inflating compile times and risking test-only behavior
# being activated in production builds by a misconfigured feature flag.
#
# The foundation allowlist is narrower than the monorepo: wyrd-auth-oidc and
# wyrd-auth-verify legitimately instantiate a wiremock server for their
# integration tests; those files are explicitly allowlisted.
#
# WHAT IT CHECKS: mockall, wiremock, and mockito references do not appear
# outside the explicitly allowlisted auth test-support files.
set -euo pipefail

# WYRD_CHECK_ROOT: override the workspace root used for source file scans.
# Set by the negative self-test harness to point at a temp fixture directory.
REPO_ROOT="${WYRD_CHECK_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
cd "$REPO_ROOT"

echo "mocks-scope: scanning for mock library references…"

if command -v rg &>/dev/null; then
    hits=$(rg -n 'mockall|wiremock|mockito' crates \
        --glob '!crates/wyrd-auth-oidc/Cargo.toml' \
        --glob '!crates/wyrd-auth-oidc/src/jwks.rs' \
        --glob '!crates/wyrd-auth-oidc/src/provider.rs' \
        --glob '!crates/wyrd-auth-verify/Cargo.toml' \
        --glob '!crates/wyrd-auth-verify/src/lib.rs' \
        2>/dev/null || true)
else
    hits=$(grep -rn --include='*.rs' --include='Cargo.toml' \
        -E 'mockall|wiremock|mockito' \
        crates \
        2>/dev/null \
        | grep -v 'crates/wyrd-auth-oidc/Cargo.toml' \
        | grep -v 'crates/wyrd-auth-oidc/src/jwks.rs' \
        | grep -v 'crates/wyrd-auth-oidc/src/provider.rs' \
        | grep -v 'crates/wyrd-auth-verify/Cargo.toml' \
        | grep -v 'crates/wyrd-auth-verify/src/lib.rs' \
        || true)
fi

if [ -n "$hits" ]; then
    echo "FAIL mocks-scope: mock dependency leaked outside allowlisted auth files:"
    echo "$hits"
    exit 1
fi

echo "mocks-scope: passed — mock libraries confined to allowlisted auth test files"
