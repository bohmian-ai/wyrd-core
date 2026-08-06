//! Generic, caller-supplied Postgres migration runner.
//!
//! This module owns the plane-neutral advisory-lock migration sequence that both
//! the Wyrd and Vala planes share. Each plane keeps a thin `migrate(&ResolvedDsns)`
//! wrapper that supplies its own compile-time-embedded [`sqlx::migrate::Migrator`]
//! and plane-specific constants, then delegates the full sequence to
//! [`run_migrations`].
//!
//! The sequence is: validate options (pure, no IO) → build a migrator pool from
//! the caller-supplied DSNs → acquire the advisory lock → ensure schemas → set
//! search path → run the migrator → release the advisory lock → close the
//! connection and pool → return the original migration outcome, surfacing unlock
//! errors only when migration succeeded.

use secrecy::ExposeSecret as _;
use sqlx::postgres::PgConnection;

use crate::dsn::ResolvedDsns;
use crate::error::SqlError;
use crate::pool::{PoolConfig, build_pool};

/// Options for a single [`run_migrations`] invocation.
///
/// All identifier slices are validated before any IO. See [`run_migrations`] for
/// the full control-flow contract.
pub struct MigrateOptions<'a> {
    /// Session-level advisory lock key acquired before migrations run and
    /// released unconditionally on all reachable outcomes.
    pub advisory_lock_key: i64,
    /// Schema names to create (if absent) before running the migrator. Every
    /// element must satisfy the identifier grammar `[a-z_][a-z0-9_]*` and the
    /// list must not contain duplicates.
    pub ensure_schemas: &'a [&'a str],
    /// Postgres `search_path` set on the migrator connection before running.
    /// Must be non-empty; every element must satisfy `[a-z_][a-z0-9_]*`.
    pub search_path: &'a [&'a str],
}

/// A migrator and its associated run options, used by callers that pass
/// multiple ordered migration sets to a fixture or bootstrap function.
pub struct MigrationSet<'a> {
    /// The compile-time-embedded migrator supplied by the calling plane.
    pub migrator: &'a sqlx::migrate::Migrator,
    /// Options governing the advisory lock, schemas, and search path for this
    /// migrator's run.
    pub options: MigrateOptions<'a>,
}

/// Apply `migrator` against a migrator-role pool built from `dsns`.
///
/// The complete control flow:
/// 1. [`validate_options`] — pure, fails with [`SqlError::InvariantViolation`]
///    before any IO if identifiers are invalid or the search path is empty.
/// 2. Build a 2-connection migrator pool from `dsns.migrator` via
///    [`build_pool`] and [`PoolConfig::migrator_defaults`].
/// 3. Acquire one connection and call `pg_advisory_lock` with
///    `options.advisory_lock_key`.
/// 4. For each schema in `options.ensure_schemas`: execute
///    `CREATE SCHEMA IF NOT EXISTS "<schema>"` using quoted, validated names.
/// 5. Execute `SET search_path TO "<a>", "<b>", ...` from validated names.
/// 6. Run `migrator` on the connection.
/// 7. Call `pg_advisory_unlock` with the same key, unconditionally.
/// 8. Close the physical connection and the pool.
/// 9. Return the migration result; surface an unlock error only when migration
///    succeeded (primary error wins via [`combine_migration_outcome`]).
///
/// Raw `PgPool` never crosses a library signature or field. Each call builds
/// and destroys its own boot-time migrator pool, which is safe because advisory
/// lock keys are distinct across planes and both planes run sequentially at boot
/// on 2-connection pools.
///
/// # Errors
/// - [`SqlError::InvariantViolation`] for invalid options before any IO.
/// - [`SqlError::Connect`] for pool/connection/lock failure.
/// - [`SqlError::Migrate`] or [`SqlError::MigrateChecksum`] for migration failure.
pub async fn run_migrations(
    dsns: &ResolvedDsns,
    migrator: &sqlx::migrate::Migrator,
    options: &MigrateOptions<'_>,
) -> Result<(), SqlError> {
    validate_options(options)?;

    let pool = build_pool(
        dsns.migrator.expose_secret(),
        PoolConfig::migrator_defaults(),
    )
    .await
    .map_err(SqlError::Connect)?;

    let mut conn = pool.acquire().await.map_err(SqlError::Connect)?;

    acquire_advisory_lock(&mut conn, options.advisory_lock_key).await?;

    let primary: Result<(), SqlError> = async {
        for schema in options.ensure_schemas {
            // Identifiers are fully validated before this point; double-quoting
            // provides injection safety. `AssertSqlSafe` is required because
            // SQLx 0.9 rejects dynamic strings by default to prevent accidental
            // injection — this usage is intentional and safe.
            let sql = format!(r#"CREATE SCHEMA IF NOT EXISTS "{schema}""#);
            sqlx::raw_sql(sqlx::AssertSqlSafe(sql.as_str()))
                .execute(&mut *conn)
                .await
                .map_err(classify_ddl_error)?;
        }

        let path_list = options
            .search_path
            .iter()
            .map(|s| format!(r#""{s}""#))
            .collect::<Vec<_>>()
            .join(", ");
        let set_sql = format!("SET search_path TO {path_list}");
        // SAFETY: identifiers are validated; `AssertSqlSafe` acknowledges the
        // intentional use of a dynamic SQL string.
        sqlx::raw_sql(sqlx::AssertSqlSafe(set_sql.as_str()))
            .execute(&mut *conn)
            .await
            .map_err(classify_ddl_error)?;

        migrator.run(&mut *conn).await.map_err(SqlError::from)
    }
    .await;

    let unlock = release_advisory_lock(&mut conn, options.advisory_lock_key).await;

    if let Err(error) = conn.close().await {
        tracing::warn!(
            error = %error,
            "failed to close migration connection cleanly"
        );
    }

    pool.close().await;

    combine_migration_outcome(primary, unlock)
}

/// Validate that all identifier and structural constraints on `options` are met.
///
/// This is a pure, IO-free check that runs before any pool construction. Every
/// failure maps to [`SqlError::InvariantViolation`] so the caller can
/// distinguish invalid configuration from connection or migration errors.
///
/// Validates:
/// - Every element of `ensure_schemas` passes [`validate_identifier`].
/// - `ensure_schemas` contains no duplicate entries.
/// - `search_path` is non-empty.
/// - Every element of `search_path` passes [`validate_identifier`].
///
/// # Errors
/// Returns [`SqlError::InvariantViolation`] for any invalid option.
fn validate_options(options: &MigrateOptions<'_>) -> Result<(), SqlError> {
    for schema in options.ensure_schemas {
        validate_identifier(schema).map_err(|_| SqlError::InvariantViolation {
            detail: format!("ensure_schemas contains invalid identifier: {schema:?}"),
        })?;
    }

    // Check for duplicates by comparing sorted pairs.
    let mut sorted = options.ensure_schemas.to_vec();
    sorted.sort_unstable();
    for window in sorted.windows(2) {
        if window[0] == window[1] {
            return Err(SqlError::InvariantViolation {
                detail: format!("ensure_schemas contains duplicate schema: {:?}", window[0]),
            });
        }
    }

    if options.search_path.is_empty() {
        return Err(SqlError::InvariantViolation {
            detail: "search_path must not be empty".to_owned(),
        });
    }

    for entry in options.search_path {
        validate_identifier(entry).map_err(|_| SqlError::InvariantViolation {
            detail: format!("search_path contains invalid identifier: {entry:?}"),
        })?;
    }

    Ok(())
}

/// Validate a Postgres identifier against the grammar `[a-z_][a-z0-9_]*`.
///
/// Only lowercase ASCII letters, digits, and underscores are accepted; the
/// identifier must start with a letter or underscore and must be non-empty.
/// This grammar is intentionally stricter than Postgres's general identifier
/// rules so that quoted DDL built from validated identifiers is injection-safe
/// and portable across locales.
///
/// # Errors
/// Returns [`SqlError::InvariantViolation`] when `ident` does not satisfy the
/// grammar.
fn validate_identifier(ident: &str) -> Result<(), SqlError> {
    let mut chars = ident.chars();
    let first = chars.next().ok_or_else(|| SqlError::InvariantViolation {
        detail: "identifier must not be empty".to_owned(),
    })?;
    if !matches!(first, 'a'..='z' | '_') {
        return Err(SqlError::InvariantViolation {
            detail: format!(
                "identifier {ident:?} must start with a lowercase letter or underscore"
            ),
        });
    }
    for ch in chars {
        if !matches!(ch, 'a'..='z' | '0'..='9' | '_') {
            return Err(SqlError::InvariantViolation {
                detail: format!("identifier {ident:?} contains invalid character: {ch:?}"),
            });
        }
    }
    Ok(())
}

/// Combine the primary migration result with the advisory-lock unlock result.
///
/// Oracle precedence (from `wyrd-sql/src/lib.rs`): the primary (migration)
/// error always wins; the unlock error surfaces only when the primary
/// operation succeeded. When both fail, a warning is logged and the primary
/// error is returned so the caller sees the root cause.
fn combine_migration_outcome(
    primary: Result<(), SqlError>,
    unlock: Result<(), SqlError>,
) -> Result<(), SqlError> {
    match (primary, unlock) {
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), Err(unlock_err)) => Err(unlock_err),
        (Err(primary_err), Ok(())) => Err(primary_err),
        (Err(primary_err), Err(unlock_err)) => {
            tracing::warn!(
                error = %unlock_err,
                "failed to release migration advisory lock after migration error"
            );
            Err(primary_err)
        }
    }
}

/// Acquire the advisory lock on `conn` using `pg_advisory_lock`.
///
/// The lock is session-scoped and must be released via [`release_advisory_lock`]
/// on the same connection before it is closed.
///
/// # Errors
/// Returns [`SqlError::Connect`] when Postgres cannot acquire the lock.
async fn acquire_advisory_lock(conn: &mut PgConnection, key: i64) -> Result<(), SqlError> {
    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(key)
        .execute(&mut *conn)
        .await
        .map_err(SqlError::Connect)?;
    Ok(())
}

/// Release the advisory lock on `conn` using `pg_advisory_unlock`.
///
/// Must be called with the same `key` passed to [`acquire_advisory_lock`].
/// Called unconditionally after the migration attempt so that even a failed
/// migration does not strand the lock.
///
/// # Errors
/// Returns [`SqlError::Connect`] when Postgres cannot release the lock.
async fn release_advisory_lock(conn: &mut PgConnection, key: i64) -> Result<(), SqlError> {
    sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(key)
        .execute(&mut *conn)
        .await
        .map_err(SqlError::Connect)?;
    Ok(())
}

/// Classify a DDL error, promoting privilege denials to [`SqlError::InsufficientPrivilege`].
///
/// Schema creation and search-path setting require DDL privileges on the
/// `wyrd_migrator` role. A `42501` error from either step indicates that the
/// pool was opened with an under-privileged role; surfacing it as
/// `InsufficientPrivilege` gives the operator a clearer remediation path than a
/// generic connection error.
fn classify_ddl_error(error: sqlx::Error) -> SqlError {
    if let sqlx::Error::Database(ref dbe) = error
        && dbe.code().as_deref() == Some("42501")
    {
        return SqlError::InsufficientPrivilege {
            detail: dbe.message().to_owned(),
        };
    }
    SqlError::Connect(error)
}

#[cfg(test)]
mod tests {
    use super::{MigrateOptions, combine_migration_outcome, validate_options};
    use crate::error::SqlError;

    /// Helper to make a valid `MigrateOptions` for use in multiple tests.
    fn valid_opts<'a>() -> MigrateOptions<'a> {
        MigrateOptions {
            advisory_lock_key: 1,
            ensure_schemas: &["wyrd", "platform"],
            search_path: &["wyrd", "platform", "public"],
        }
    }

    /// Helper to create an `InvariantViolation` error for precedence tests.
    fn invariant(detail: &str) -> SqlError {
        SqlError::InvariantViolation {
            detail: detail.to_owned(),
        }
    }

    // --- validate_options unit tests (AC3) ---

    /// `validate_options` returns `Ok(())` for a well-formed options struct with
    /// valid lowercase identifiers, no duplicates, and a non-empty search path.
    #[test]
    fn validate_options_accepts_valid_lowercase() {
        assert!(validate_options(&valid_opts()).is_ok());
    }

    /// `validate_options` rejects schema names that contain characters outside
    /// `[a-z_][a-z0-9_]*`, mapping the failure to `SqlError::InvariantViolation`.
    #[test]
    fn validate_options_rejects_non_identifier_schema() {
        let opts = MigrateOptions {
            advisory_lock_key: 1,
            ensure_schemas: &["bad-name"],
            search_path: &["public"],
        };
        assert!(matches!(
            validate_options(&opts),
            Err(SqlError::InvariantViolation { .. })
        ));
    }

    /// `validate_options` rejects `ensure_schemas` slices that contain the same
    /// schema name more than once, mapping the failure to `InvariantViolation`.
    #[test]
    fn validate_options_rejects_duplicate_schema() {
        let opts = MigrateOptions {
            advisory_lock_key: 1,
            ensure_schemas: &["wyrd", "wyrd"],
            search_path: &["wyrd"],
        };
        assert!(matches!(
            validate_options(&opts),
            Err(SqlError::InvariantViolation { .. })
        ));
    }

    /// `validate_options` rejects an empty `search_path` slice, mapping the
    /// failure to `InvariantViolation` before any IO.
    #[test]
    fn validate_options_rejects_empty_search_path() {
        let opts = MigrateOptions {
            advisory_lock_key: 1,
            ensure_schemas: &["wyrd"],
            search_path: &[],
        };
        assert!(matches!(
            validate_options(&opts),
            Err(SqlError::InvariantViolation { .. })
        ));
    }

    /// `validate_options` rejects `search_path` entries that contain whitespace
    /// or other invalid characters, mapping the failure to `InvariantViolation`.
    #[test]
    fn validate_options_rejects_non_identifier_search_path() {
        let opts = MigrateOptions {
            advisory_lock_key: 1,
            ensure_schemas: &["wyrd"],
            search_path: &["a b"],
        };
        assert!(matches!(
            validate_options(&opts),
            Err(SqlError::InvariantViolation { .. })
        ));
    }

    // --- combine_migration_outcome unit tests (AC3) ---

    /// When both the primary migration and the advisory-lock unlock fail,
    /// `combine_migration_outcome` returns the **primary** error so the caller
    /// sees the root cause rather than the secondary unlock failure.
    #[test]
    fn combine_migration_outcome_prefers_primary_error() {
        let primary = invariant("migration failed");
        let unlock = invariant("unlock failed");
        let result = combine_migration_outcome(Err(primary), Err(unlock));
        assert!(matches!(
            result,
            Err(SqlError::InvariantViolation { ref detail }) if detail == "migration failed"
        ));
    }

    /// When the primary migration succeeds but the advisory-lock unlock fails,
    /// `combine_migration_outcome` returns the unlock error so the operator can
    /// investigate the stranded lock.
    #[test]
    fn combine_migration_outcome_returns_unlock_error_when_primary_ok() {
        let unlock = invariant("unlock failed");
        let result = combine_migration_outcome(Ok(()), Err(unlock));
        assert!(matches!(
            result,
            Err(SqlError::InvariantViolation { ref detail }) if detail == "unlock failed"
        ));
    }

    /// When both the primary migration and the advisory-lock unlock succeed,
    /// `combine_migration_outcome` returns `Ok(())`.
    #[test]
    fn combine_migration_outcome_ok_when_both_ok() {
        let result = combine_migration_outcome(Ok(()), Ok(()));
        assert!(result.is_ok());
    }
}

// --- PG behavior tests (AC4, deferred to T3) ---
//
// These integration tests require a live Postgres instance provisioned by the
// T3 test harness (`mise run test:pg` via `scripts/postgres/with-test-postgres.sh`).
// They are authored here but gated behind the `pg` feature so they are never
// compiled or executed without an explicit Postgres lane. T2 acceptance records
// this deferral; T3's focused verification executes them.
//
// Test fixture: `crates/wyrd-sql-core/tests/migrations/0001_init.sql`
// Creates `sql_core_test` schema with a `migrations_ran` table used to prove
// that the caller-supplied migrator was applied.

#[cfg(all(test, feature = "pg"))]
mod pg_tests {
    //! Postgres behavior tests for [`run_migrations`].
    //!
    //! These tests require the DSN environment variables set by the T3 test
    //! harness. They are compiled only when the `pg` feature is active and are
    //! not invoked in T2's focused verification.

    use std::borrow::Cow;
    use std::path::PathBuf;
    use std::pin::Pin;

    use secrecy::SecretString;
    use sqlx::SqlSafeStr as _;
    use sqlx::error::BoxDynError;
    use sqlx::migrate::{Migration, MigrationSource, MigrationType, Migrator};

    use crate::dsn::ResolvedDsns;
    use crate::error::SqlError;
    use crate::migration::{MigrateOptions, run_migrations};

    /// Read DSNs from the test harness environment variables.
    fn test_dsns() -> ResolvedDsns {
        let migrator_url = std::env::var("WYRD_TEST_MIGRATOR_URL")
            .expect("WYRD_TEST_MIGRATOR_URL must be set by the T3 test harness");
        let app_url = std::env::var("WYRD_TEST_APP_URL").unwrap_or_else(|_| migrator_url.clone());
        ResolvedDsns {
            app: SecretString::from(app_url),
            migrator: SecretString::from(migrator_url),
            platform_admin: None,
            catalog_app: SecretString::from(
                std::env::var("WYRD_TEST_CATALOG_URL")
                    .unwrap_or_else(|_| "postgres://unused@localhost/unused".to_owned()),
            ),
        }
    }

    /// Build the test migrator from the checked-in fixture directory by awaiting
    /// `Migrator::new` inside an async context — no external `FutureExt` needed.
    async fn test_migrator() -> Migrator {
        let dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/migrations");
        Migrator::new(dir.as_path())
            .await
            .expect("test migrations directory is readable and valid")
    }

    const TEST_LOCK_KEY: i64 = 0x5754_4553_5453_514c; // "WTESTSQL" in hex

    /// A failing [`MigrationSource`] that produces one migration with invalid SQL
    /// (version 9999). Used to verify that `run_migrations` returns a migration
    /// error and releases the advisory lock even when the migrator fails.
    ///
    /// The return type is spelled out as `Pin<Box<dyn Future<...> + Send>>` —
    /// the expansion of `BoxFuture<'static, ...>` — so no `futures_core`
    /// dependency is required.
    #[derive(Debug)]
    struct FailingSource;

    impl MigrationSource<'static> for FailingSource {
        fn resolve(
            self,
        ) -> Pin<
            Box<
                dyn std::future::Future<Output = Result<Vec<Migration>, BoxDynError>>
                    + Send
                    + 'static,
            >,
        > {
            Box::pin(async {
                Ok(vec![Migration::new(
                    9999,
                    Cow::Borrowed("fail"),
                    MigrationType::Simple,
                    // `AssertSqlSafe` converts a string literal to the `SqlStr`
                    // type that `Migration::new` requires in sqlx 0.9.
                    sqlx::AssertSqlSafe("SELECT INVALID GARBAGE THAT FAILS;").into_sql_str(),
                    false,
                )])
            })
        }
    }

    /// `run_migrations` applies the caller-supplied migrator: the test schema
    /// and table created by `0001_init.sql` must exist after the call.
    #[tokio::test]
    async fn run_migrations_applies_supplied_migrator() {
        let dsns = test_dsns();
        let migrator = test_migrator().await;
        let options = MigrateOptions {
            advisory_lock_key: TEST_LOCK_KEY,
            ensure_schemas: &["sql_core_test"],
            search_path: &["sql_core_test", "public"],
        };

        run_migrations(&dsns, &migrator, &options)
            .await
            .expect("migrations should succeed on a clean database");

        // Verify the table created by the migrator exists.
        use secrecy::ExposeSecret as _;
        use sqlx::postgres::PgPoolOptions;
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(dsns.migrator.expose_secret())
            .await
            .expect("connect for verification");
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = 'sql_core_test'
                  AND table_name   = 'migrations_ran'
            )",
        )
        .fetch_one(&pool)
        .await
        .expect("existence query");
        assert!(row.0, "migrations_ran table must exist after migration");
        pool.close().await;
    }

    /// Running `run_migrations` a second time with the same migrator is
    /// idempotent: SQLx skips already-applied migrations and the call returns
    /// `Ok(())`.
    #[tokio::test]
    async fn run_migrations_is_idempotent_on_repeat() {
        let dsns = test_dsns();
        let migrator = test_migrator().await;
        let options = MigrateOptions {
            advisory_lock_key: TEST_LOCK_KEY,
            ensure_schemas: &["sql_core_test"],
            search_path: &["sql_core_test", "public"],
        };

        // First run.
        run_migrations(&dsns, &migrator, &options)
            .await
            .expect("first migration must succeed");

        // Second run must also succeed.
        run_migrations(&dsns, &migrator, &options)
            .await
            .expect("repeat migration must succeed (idempotent)");
    }

    /// When the supplied migrator fails (e.g. a bad SQL statement), `run_migrations`
    /// returns a migration error **and** releases the advisory lock so that a
    /// subsequent call with a corrected migrator can succeed.
    #[tokio::test]
    async fn run_migrations_returns_migration_error_and_releases_lock() {
        let dsns = test_dsns();

        let failing_migrator = Migrator::new(FailingSource)
            .await
            .expect("failing migrator builds");

        let options = MigrateOptions {
            advisory_lock_key: TEST_LOCK_KEY,
            ensure_schemas: &["sql_core_test"],
            search_path: &["sql_core_test", "public"],
        };

        let first = run_migrations(&dsns, &failing_migrator, &options).await;
        assert!(
            matches!(
                first,
                Err(SqlError::Migrate(_) | SqlError::MigrateChecksum { .. })
            ),
            "failing migrator must return a migration error"
        );

        // The advisory lock must have been released: a subsequent call with the
        // good migrator must not deadlock or fail on lock acquisition.
        let good_migrator = test_migrator().await;
        run_migrations(&dsns, &good_migrator, &options)
            .await
            .expect("good migrator succeeds after failed attempt (lock was released)");
    }
}
