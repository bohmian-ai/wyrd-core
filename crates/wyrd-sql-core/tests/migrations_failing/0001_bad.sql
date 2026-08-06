-- Deliberately invalid SQL to cause a migration error. Used by
-- run_migrations_returns_migration_error_and_releases_lock to verify that the
-- advisory lock is released after a migration failure so a subsequent good-
-- migrator run succeeds.
THIS IS NOT VALID SQL;
