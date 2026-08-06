//! Generic, caller-supplied Postgres migration runner.
//!
//! This module owns the plane-neutral advisory-lock migration sequence that both
//! the Wyrd and Vala planes share. Each plane keeps a thin `migrate(&PgPool)`
//! wrapper that supplies its own compile-time-embedded [`sqlx::migrate::Migrator`]
//! and plane-specific constants, then delegates the full sequence to
//! [`run_migrations`].
//!
//! The sequence is: validate options (pure, no IO) → acquire a connection from
//! the caller-supplied pool → acquire the advisory lock → ensure schemas → set
//! search path → run the migrator → release the advisory lock → close the
//! acquired connection → return the original migration outcome, surfacing unlock
//! errors only when migration succeeded.

use sqlx::postgres::PgConnection;

use crate::error::SqlError;

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

/// Apply `migrator` against a connection acquired from the caller-supplied `migrator_pool`.
///
/// The complete control flow:
/// 1. [`validate_options`] — pure, fails with [`SqlError::InvariantViolation`]
///    before any IO if identifiers are invalid or the search path is empty.
/// 2. Acquire one connection from `migrator_pool` and call `pg_advisory_lock`
///    with `options.advisory_lock_key`.
/// 3. For each schema in `options.ensure_schemas`: execute
///    `CREATE SCHEMA IF NOT EXISTS "<schema>"` using quoted, validated names.
/// 4. Execute `SET search_path TO "<a>", "<b>", ...` from validated names.
/// 5. Run `migrator` on the connection.
/// 6. Call `pg_advisory_unlock` with the same key, unconditionally.
/// 7. Close the acquired connection (the caller retains ownership of the pool).
/// 8. Return the migration result; surface an unlock error only when migration
///    succeeded (primary error wins via [`combine_migration_outcome`]).
///
/// The caller owns `migrator_pool` and is responsible for closing it after all
/// migration sets have run. This matches the source `wyrd-sql`/`vala-sql`
/// `migrate(&pool)` contract exactly.
///
/// # Errors
/// - [`SqlError::InvariantViolation`] for invalid options before any IO.
/// - [`SqlError::Connect`] for connection or lock failure.
/// - [`SqlError::Migrate`] or [`SqlError::MigrateChecksum`] for migration failure.
pub async fn run_migrations(
    migrator_pool: &sqlx::postgres::PgPool,
    migrator: &sqlx::migrate::Migrator,
    options: &MigrateOptions<'_>,
) -> Result<(), SqlError> {
    validate_options(options)?;

    let mut conn = migrator_pool.acquire().await.map_err(SqlError::Connect)?;

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
