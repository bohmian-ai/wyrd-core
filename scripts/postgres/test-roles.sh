#!/usr/bin/env bash
set -Eeuo pipefail

readonly root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly wrapper="$root/scripts/postgres/with-test-postgres.sh"
command -v docker >/dev/null 2>&1 || { echo "docker is required for the role audit" >&2; exit 1; }
docker info >/dev/null 2>&1 || { echo "Docker daemon is unavailable" >&2; exit 1; }

"$wrapper" -- bash -c '
  set -Eeuo pipefail
  root="$1"
  # Verify the three foundation roles have the expected attributes.
  test "$(psql "$DATABASE_URL" -Atqc "SELECT rolcanlogin, rolbypassrls, rolsuper, rolcreatedb FROM pg_roles WHERE rolname = '\''wyrd_migrator'\''")" = "t|t|f|f"
  test "$(psql "$DATABASE_URL" -Atqc "SELECT rolcanlogin, rolbypassrls, rolsuper, rolcreatedb FROM pg_roles WHERE rolname = '\''wyrd_app'\''")" = "t|f|f|f"
  test "$(psql "$DATABASE_URL" -Atqc "SELECT rolcanlogin, rolbypassrls, rolsuper, rolcreatedb FROM pg_roles WHERE rolname = '\''wyrd_platform_admin'\''")" = "t|t|f|f"
  test "$(psql "$DATABASE_URL" -Atqc "SELECT has_database_privilege('\''wyrd_app'\'', '\''wyrd'\'', '\''CONNECT'\''), has_database_privilege('\''wyrd_migrator'\'', '\''wyrd'\'', '\''CREATE'\'' )")" = "t|t"

  admin_password="${WYRD_TEST_POSTGRES_ADMIN_PASSWORD:-wyrd_test_admin_pw}"
  role_sql="$root/scripts/postgres/roles.sql"
  # Drift-repair test: mutate roles then re-apply roles.sql and verify idempotent convergence.
  PGPASSWORD="$admin_password" psql "$WYRD_TEST_DATABASE_ADMIN_URL" -v ON_ERROR_STOP=1 -c "ALTER ROLE wyrd_app CREATEDB BYPASSRLS; REVOKE CONNECT ON DATABASE wyrd FROM wyrd_app;"
  PGPASSWORD="$admin_password" psql "$WYRD_TEST_DATABASE_ADMIN_URL" \
    --set=migrator_password="$WYRD_DATABASE_MIGRATOR_PASSWORD" \
    --set=app_password="${WYRD_TEST_POSTGRES_APP_PASSWORD:-wyrd_app_pw}" \
    --set=platform_admin_password="$WYRD_DATABASE_PLATFORM_ADMIN_PASSWORD" \
    --file="$role_sql"
  test "$(psql "$DATABASE_URL" -Atqc "SELECT rolcanlogin, rolbypassrls, rolsuper, rolcreatedb FROM pg_roles WHERE rolname = '\''wyrd_app'\''")" = "t|f|f|f"

  app_url="$WYRD_DATABASE_URL"
  platform_admin_url="postgres://wyrd_platform_admin:${WYRD_DATABASE_PLATFORM_ADMIN_PASSWORD}@${DATABASE_URL#*@}"
  PGPASSWORD="${WYRD_TEST_POSTGRES_APP_PASSWORD:-wyrd_app_pw}" psql "$app_url" -Atqc "SELECT 1" >/dev/null
  PGPASSWORD="$WYRD_DATABASE_PLATFORM_ADMIN_PASSWORD" psql "$platform_admin_url" -Atqc "SELECT 1" >/dev/null

  database_acl="$(psql "$DATABASE_URL" -Atqc "SELECT string_agg(role.rolname || '\'':'\'' || acl.privilege_type, '\'','\'' ORDER BY role.rolname, acl.privilege_type) FROM pg_database database CROSS JOIN LATERAL aclexplode(database.datacl) acl JOIN pg_roles role ON role.oid=acl.grantee WHERE database.datname='\''wyrd'\'' AND role.rolname IN ('\''wyrd_migrator'\'','\''wyrd_app'\'','\''wyrd_platform_admin'\'')")"
  test "$database_acl" = "wyrd_app:CONNECT,wyrd_migrator:CONNECT,wyrd_migrator:CREATE,wyrd_platform_admin:CONNECT"
' _ "$root"
echo "postgres role audit: PASS"
