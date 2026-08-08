-- Fixture migration for wyrd-sql-core pg_migration tests.
-- Creates a table in the test schema to prove the caller-supplied migrator
-- was applied by run_migrations.
CREATE TABLE IF NOT EXISTS test_schema.sql_core_fixture_table (
    id   BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL
);
