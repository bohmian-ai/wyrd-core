-- Migration fixture for wyrd-sql-core PG behavior tests.
--
-- Creates a table in the `sql_core_test` schema that proves the
-- caller-supplied migrator was applied. The schema itself is created by
-- `run_migrations` via `ensure_schemas` before this migration runs.
--
-- Used by: T2-authored, T3-executed PG tests.
CREATE TABLE IF NOT EXISTS sql_core_test.migrations_ran (
    id          BIGSERIAL PRIMARY KEY,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
