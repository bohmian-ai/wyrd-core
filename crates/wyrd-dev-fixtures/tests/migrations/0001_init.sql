-- Fixture migration: create a table in the test schema to prove the
-- caller-supplied migrator was applied by PgFixture::start.
CREATE TABLE IF NOT EXISTS test_schema.fixture_table (
    id   BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL
);
