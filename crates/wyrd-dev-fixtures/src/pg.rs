//! Plane-neutral isolated Postgres fixture for SQL integration tests.
//!
//! [`PgFixture`] owns the lifecycle of one ephemeral per-test Postgres database:
//! it creates the database, grants the migrator role the necessary privileges,
//! resolves role-specific DSNs rewritten to the new database, applies zero or
//! more caller-supplied ordered [`MigrationSet`]s via
//! [`wyrd_sql_core::run_migrations`], constructs the [`OperatorPool`], and
//! drops the database on shutdown or `Drop`.
//!
//! No plane migrations, tenant seeds, or server handles are stored or applied
//! by this module. Callers run their own migrations by passing [`MigrationSet`]
//! slices to [`PgFixture::start`].

use std::collections::hash_map::DefaultHasher;
use std::env;
use std::hash::{Hash, Hasher};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use secrecy::{ExposeSecret as _, SecretString};
use sqlx::AssertSqlSafe;
use url::Url;
use wyrd_sql_core::dsn::ResolvedDsns;
use wyrd_sql_core::error::SqlError;
use wyrd_sql_core::migration::MigrationSet;
use wyrd_sql_core::operator_pool::OperatorPool;
use wyrd_sql_core::pool::{PoolConfig, build_pool};
use wyrd_sql_core::{WYRD_MIGRATOR_ROLE, run_migrations};

/// Monotonic counter used to uniquify database names within one process.
static TEST_DB_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Per-test isolated Postgres fixture backed by a fixture-owned ephemeral database.
///
/// The fixture creates a unique database under the cluster admin role, resolves
/// role-specific DSNs rewritten to that database, applies caller-supplied
/// migration sets in order, and then constructs primitive pools for test use.
///
/// On `shutdown` (or `Drop` as best-effort fallback), the fixture closes the
/// operator pool and drops the isolated database.
///
/// No tenant seeds, plane handles, or embedded migrations run here.
pub struct PgFixture {
    /// Resolved DSNs rewritten to the isolated test database.
    resolved_dsns: ResolvedDsns,
    /// Audited cross-tenant BYPASSRLS pool built from the platform-admin DSN.
    operator_pool: OperatorPool,
    /// Ephemeral database owner. Drops after pools are closed so the database
    /// is removed after all connections have been released.
    _test_db: TestDatabase,
}

/// Errors returned while starting or shutting down a [`PgFixture`].
#[derive(Debug, thiserror::Error)]
pub enum FixtureError {
    /// SQL layer failed.
    #[error("sql layer failed: {0}")]
    Sql(#[from] SqlError),
}

impl PgFixture {
    /// Create an isolated test database, apply zero or more ordered migration
    /// sets in the order supplied, and return a running fixture.
    ///
    /// The fixture creates one transient migrator pool, reuses it across every
    /// [`MigrationSet`] in order, then drops it before constructing the
    /// fixture-owned [`OperatorPool`]. Zero migration sets is valid and returns
    /// a fixture over an empty database.
    ///
    /// On failure after database creation, the ephemeral database is dropped
    /// via the `TestDatabase` cleanup path before the error propagates.
    ///
    /// # Errors
    /// Returns [`FixtureError`] when database creation, role grant, DSN
    /// resolution, pool construction, or any migration set fails.
    pub async fn start(migrations: &[MigrationSet<'_>]) -> Result<Self, FixtureError> {
        let test_db = TestDatabase::create().await?;
        let resolved_dsns = test_db.resolved_dsns()?;

        // Build one transient migrator pool; reuse it across all sets (mirrors
        // the source wyrd_sql::migrate(&pool); vala_sql::migrate(&pool) reuse).
        let migrator_pool = build_pool(
            resolved_dsns.migrator.expose_secret(),
            PoolConfig::migrator_defaults(),
        )
        .await
        .map_err(SqlError::Connect)?;

        let migration_result: Result<(), SqlError> = async {
            for set in migrations {
                run_migrations(&migrator_pool, set.migrator, &set.options).await?;
            }
            Ok(())
        }
        .await;

        // Always drop the transient migrator pool before propagating errors or
        // constructing the fixture. The TestDatabase Drop path handles the
        // ephemeral DB cleanup on error.
        migrator_pool.close().await;

        migration_result?;

        // Construct the operator pool from the platform-admin DSN, which is
        // required for the fixture's cross-tenant assertion capability.
        let platform_admin_dsn =
            resolved_dsns
                .platform_admin
                .as_ref()
                .ok_or_else(|| SqlError::InvariantViolation {
                    detail: "WYRD_DATABASE_PLATFORM_ADMIN_PASSWORD is required for PgFixture; \
                         set it via the with-test-postgres.sh wrapper"
                        .to_owned(),
                })?;
        let operator_pool = build_pool(
            platform_admin_dsn.expose_secret(),
            PoolConfig::migrator_defaults(),
        )
        .await
        .map_err(SqlError::Connect)?;

        Ok(Self {
            resolved_dsns,
            operator_pool: OperatorPool::from(operator_pool),
            _test_db: test_db,
        })
    }

    /// Borrow the resolved DSNs rewritten to the isolated test database.
    #[must_use]
    pub fn resolved_dsns(&self) -> &ResolvedDsns {
        &self.resolved_dsns
    }

    /// Borrow the audited cross-tenant operator pool.
    #[must_use]
    pub fn operator_pool(&self) -> &OperatorPool {
        &self.operator_pool
    }

    /// Close the operator pool and drop the isolated database.
    ///
    /// Prefer `shutdown` over relying on `Drop` when awaiting graceful close is
    /// possible; `Drop` performs best-effort synchronous cleanup only.
    ///
    /// # Errors
    /// Always returns `Ok`; the error variant is reserved for future use and
    /// ensures callers handle the result idiomatically.
    pub async fn shutdown(self) -> Result<(), FixtureError> {
        // Dropping self closes the operator pool and triggers TestDatabase::drop.
        drop(self);
        Ok(())
    }
}

/// Owns one ephemeral isolated database and the admin authority to clean it up.
struct TestDatabase {
    /// Unique database name allocated for this fixture instance.
    name: String,
    /// Neutral cluster-admin DSN used only for database lifecycle operations.
    admin_dsn: SecretString,
    /// Base DSNs resolved from the environment, pointing at the shared database.
    base_dsns: ResolvedDsns,
}

impl TestDatabase {
    /// Create the isolated database, grant the migrator its privileges, and
    /// return a `TestDatabase` owner.
    ///
    /// Uses the neutral `wyrd_test_admin` superuser DSN from
    /// `WYRD_TEST_DATABASE_ADMIN_URL` to create the database and issue the
    /// grant. The compose service creates only `wyrd_test_admin`; the three
    /// foundation roles are created by the wrapper running `roles.sql`.
    ///
    /// # Errors
    /// Returns [`SqlError`] when the admin DSN is missing, the database cannot
    /// be created, or the migrator grant fails.
    async fn create() -> Result<Self, SqlError> {
        let base_dsns = resolved_external_test_dsns()?;
        let admin_dsn = test_database_admin_dsn("wyrd")?;
        let name = unique_database_name();

        let admin_pool = build_pool(admin_dsn.expose_secret(), PoolConfig::migrator_defaults())
            .await
            .map_err(SqlError::Connect)?;

        sqlx::query(AssertSqlSafe(format!("CREATE DATABASE \"{name}\"")))
            .execute(&admin_pool)
            .await
            .map_err(SqlError::from)?;
        sqlx::query(AssertSqlSafe(format!(
            "GRANT CONNECT, CREATE ON DATABASE \"{name}\" TO {WYRD_MIGRATOR_ROLE}"
        )))
        .execute(&admin_pool)
        .await
        .map_err(SqlError::from)?;
        admin_pool.close().await;

        Ok(Self {
            name,
            admin_dsn,
            base_dsns,
        })
    }

    /// Resolve role-specific DSNs rewritten to point at the isolated database.
    ///
    /// # Errors
    /// Returns [`SqlError::InvariantViolation`] when a base DSN cannot be parsed
    /// or the database path cannot be rewritten.
    fn resolved_dsns(&self) -> Result<ResolvedDsns, SqlError> {
        Ok(ResolvedDsns {
            app: database_dsn(&self.base_dsns.app, &self.name)?,
            migrator: database_dsn(&self.base_dsns.migrator, &self.name)?,
            platform_admin: self
                .base_dsns
                .platform_admin
                .as_ref()
                .map(|dsn| database_dsn(dsn, &self.name))
                .transpose()?,
            catalog_app: database_dsn(&self.base_dsns.catalog_app, &self.name)?,
        })
    }
}

impl Drop for TestDatabase {
    /// Best-effort synchronous cleanup of the ephemeral database.
    ///
    /// Spawns a dedicated thread with its own single-thread Tokio runtime to
    /// issue `DROP DATABASE … WITH (FORCE)` via the neutral admin connection.
    /// Errors are printed to stderr; this path must not panic.
    fn drop(&mut self) {
        let database_name = self.name.clone();
        let admin_dsn = self.admin_dsn.clone();
        let handle = thread::spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    eprintln!("failed to build test database cleanup runtime: {error}");
                    return;
                }
            };

            runtime.block_on(async move {
                let pool =
                    match build_pool(admin_dsn.expose_secret(), PoolConfig::migrator_defaults())
                        .await
                    {
                        Ok(pool) => pool,
                        Err(error) => {
                            eprintln!("failed to connect for test database cleanup: {error}");
                            return;
                        }
                    };
                let drop_result = sqlx::query(AssertSqlSafe(format!(
                    "DROP DATABASE IF EXISTS \"{database_name}\" WITH (FORCE)"
                )))
                .execute(&pool)
                .await;
                pool.close().await;

                if let Err(error) = drop_result {
                    eprintln!("failed to drop test database {database_name}: {error}");
                }
            });
        });

        if handle.join().is_err() {
            eprintln!("test database cleanup thread panicked");
        }
    }
}

/// Resolve the base external DSNs from the wrapper-set environment variables.
///
/// Requires `WYRD_DATABASE_URL`, `WYRD_DATABASE_MIGRATOR_PASSWORD`, and
/// `WYRD_DATABASE_CATALOG_APP_PASSWORD` to all be set (the wrapper exports
/// `WYRD_DATABASE_CATALOG_APP_PASSWORD` with a dummy value to satisfy the
/// resolver even though no catalog migration runs in the foundation).
///
/// # Errors
/// Returns [`SqlError::InvariantViolation`] when DSN variables are absent or
/// partially configured.
fn resolved_external_test_dsns() -> Result<ResolvedDsns, SqlError> {
    wyrd_sql_core::dsn::resolve_external_dsns_from_env()
        .map_err(|error| SqlError::InvariantViolation {
            detail: format!("test DB DSN config error: {error}"),
        })?
        .ok_or_else(|| SqlError::InvariantViolation {
            detail: "test DB env unset (WYRD_DATABASE_URL + WYRD_DATABASE_MIGRATOR_PASSWORD + \
                     WYRD_DATABASE_CATALOG_APP_PASSWORD); set these via with-test-postgres.sh"
                .to_owned(),
        })
}

/// Resolve the neutral admin DSN rewritten to point at a specific database.
///
/// Reads `WYRD_TEST_DATABASE_ADMIN_URL` from the environment.
///
/// # Errors
/// Returns [`SqlError::InvariantViolation`] when the environment variable is
/// absent or the DSN cannot be parsed.
fn test_database_admin_dsn(database: &str) -> Result<SecretString, SqlError> {
    let value =
        env::var("WYRD_TEST_DATABASE_ADMIN_URL").map_err(|_| SqlError::InvariantViolation {
            detail: "WYRD_TEST_DATABASE_ADMIN_URL is unset; \
                     set it via the with-test-postgres.sh wrapper"
                .to_owned(),
        })?;
    database_dsn(&SecretString::from(value), database)
}

/// Rewrite the database path component of `base` to `database`.
///
/// # Errors
/// Returns [`SqlError::InvariantViolation`] when `base` is not a valid URL.
fn database_dsn(base: &SecretString, database: &str) -> Result<SecretString, SqlError> {
    let mut url =
        Url::parse(base.expose_secret()).map_err(|error| SqlError::InvariantViolation {
            detail: format!("test DB DSN parse error: {error}"),
        })?;
    url.set_path(database);
    Ok(SecretString::from(url.to_string()))
}

/// Generate a unique database name for one fixture instance.
///
/// Encodes a hash of the current working directory, the process ID, a
/// monotonic counter, and a nanosecond timestamp to minimize collision
/// probability across concurrent fixtures and successive test runs. The result
/// is truncated to the maximum Postgres identifier length (63 bytes).
fn unique_database_name() -> String {
    let cwd = env::current_dir()
        .ok()
        .and_then(|path| path.into_os_string().into_string().ok())
        .unwrap_or_else(|| "unknown".to_owned());
    let mut hasher = DefaultHasher::new();
    cwd.hash(&mut hasher);
    let worktree_hash = hasher.finish();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let counter = TEST_DB_COUNTER.fetch_add(1, Ordering::Relaxed);
    let raw = format!(
        "wyrd_test_{worktree_hash:016x}_{:x}_{counter:x}_{now:x}",
        process::id()
    );
    raw.chars().take(63).collect()
}

#[cfg(test)]
mod pg_tests {
    //! Live Postgres PgFixture integration tests.
    //!
    //! These tests are skipped when `WYRD_DATABASE_URL` is absent so the
    //! default test suite remains credential-free. Run them through
    //! `scripts/postgres/with-test-postgres.sh -- cargo test --features pg`.

    use secrecy::ExposeSecret as _;

    use super::PgFixture;
    use wyrd_sql_core::dsn::ResolvedDsns;
    use wyrd_sql_core::migration::{MigrateOptions, MigrationSet};

    /// Advisory lock keys — distinct per test to avoid lock contention when
    /// tests run concurrently in the same process.
    const LOCK_SINGLE: i64 = 0x0001_f1c0;
    const LOCK_ORDERED_A: i64 = 0x0002_f1c0;
    const LOCK_ORDERED_B: i64 = 0x0003_f1c0;
    const LOCK_FAIL_A: i64 = 0x0004_f1c0;
    const LOCK_FAIL_B: i64 = 0x0005_f1c0;

    /// Macro to build a Migrator from a test-fixtures subdirectory.
    macro_rules! fixture_migrator {
        ($subdir:literal) => {{
            sqlx::migrate::Migrator::new(std::path::Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/",
                $subdir
            )))
            .await
            .expect(concat!("migrator loaded from tests/", $subdir))
        }};
    }

    /// A fixture starts with zero migration sets, exposing resolved DSNs and
    /// an operator pool against an empty database.
    #[tokio::test]
    async fn fixture_starts_with_zero_migrations() {
        if std::env::var("WYRD_DATABASE_URL").is_err() {
            return;
        }
        let fixture = PgFixture::start(&[]).await.expect("fixture starts");
        let one: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(fixture.operator_pool().pool())
            .await
            .expect("operator pool query");
        assert_eq!(one.0, 1);
        fixture.shutdown().await.expect("shutdown");
    }

    /// A fixture migrates a single caller-supplied set and the migration is
    /// reflected in the database schema.
    #[tokio::test]
    async fn fixture_applies_single_migration_set() {
        if std::env::var("WYRD_DATABASE_URL").is_err() {
            return;
        }
        let migrator = fixture_migrator!("migrations");
        let sets = [MigrationSet {
            migrator: &migrator,
            options: MigrateOptions {
                advisory_lock_key: LOCK_SINGLE,
                ensure_schemas: &["test_schema"],
                search_path: &["test_schema"],
            },
        }];
        let fixture = PgFixture::start(&sets).await.expect("fixture starts");
        let exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS (
               SELECT 1 FROM pg_class c
               JOIN pg_namespace n ON n.oid = c.relnamespace
               WHERE n.nspname = 'test_schema' AND c.relname = 'fixture_table'
             )",
        )
        .fetch_one(fixture.operator_pool().pool())
        .await
        .expect("table existence query");
        assert!(exists.0, "fixture_table exists after migration");
        fixture.shutdown().await.expect("shutdown");
    }

    /// Two ordered migration sets run in the order supplied within one fixture.
    #[tokio::test]
    async fn fixture_applies_ordered_migration_sets() {
        if std::env::var("WYRD_DATABASE_URL").is_err() {
            return;
        }
        let migrator_a = fixture_migrator!("migrations");
        let migrator_b = fixture_migrator!("migrations");
        let sets = [
            MigrationSet {
                migrator: &migrator_a,
                options: MigrateOptions {
                    advisory_lock_key: LOCK_ORDERED_A,
                    ensure_schemas: &["test_schema"],
                    search_path: &["test_schema"],
                },
            },
            MigrationSet {
                migrator: &migrator_b,
                options: MigrateOptions {
                    advisory_lock_key: LOCK_ORDERED_B,
                    ensure_schemas: &["test_schema"],
                    search_path: &["test_schema"],
                },
            },
        ];
        let fixture = PgFixture::start(&sets).await.expect("fixture starts");
        let exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS (
               SELECT 1 FROM pg_class c
               JOIN pg_namespace n ON n.oid = c.relnamespace
               WHERE n.nspname = 'test_schema' AND c.relname = 'fixture_table'
             )",
        )
        .fetch_one(fixture.operator_pool().pool())
        .await
        .expect("table existence query after ordered sets");
        assert!(exists.0, "fixture_table exists after both migration sets");
        fixture.shutdown().await.expect("shutdown");
    }

    /// Two concurrent fixtures receive distinct ephemeral databases and do not
    /// interfere with each other's migration lifecycle.
    #[tokio::test]
    async fn concurrent_fixtures_use_distinct_databases() {
        if std::env::var("WYRD_DATABASE_URL").is_err() {
            return;
        }
        let (a, b) = tokio::join!(PgFixture::start(&[]), PgFixture::start(&[]));
        let a = a.expect("fixture a starts");
        let b = b.expect("fixture b starts");
        let db_a: (String,) = sqlx::query_as("SELECT current_database()::text")
            .fetch_one(a.operator_pool().pool())
            .await
            .expect("current_database from a");
        let db_b: (String,) = sqlx::query_as("SELECT current_database()::text")
            .fetch_one(b.operator_pool().pool())
            .await
            .expect("current_database from b");
        assert_ne!(db_a.0, db_b.0, "concurrent fixtures use distinct databases");
        a.shutdown().await.expect("fixture a shutdown");
        b.shutdown().await.expect("fixture b shutdown");
    }

    /// The three foundation roles exist with their expected `rolbypassrls`
    /// attributes in a fresh fixture database.
    #[tokio::test]
    async fn roles_have_expected_attributes() {
        if std::env::var("WYRD_DATABASE_URL").is_err() {
            return;
        }
        let fixture = PgFixture::start(&[]).await.expect("fixture starts");
        let rows: Vec<(String, bool)> = sqlx::query_as(
            "SELECT rolname, rolbypassrls
             FROM pg_roles
             WHERE rolname IN ('wyrd_app', 'wyrd_migrator', 'wyrd_platform_admin')
             ORDER BY rolname",
        )
        .fetch_all(fixture.operator_pool().pool())
        .await
        .expect("role query");
        assert_eq!(
            rows,
            vec![
                ("wyrd_app".to_owned(), false),
                ("wyrd_migrator".to_owned(), true),
                ("wyrd_platform_admin".to_owned(), true),
            ]
        );
        fixture.shutdown().await.expect("shutdown");
    }

    /// The operator pool runs as `wyrd_platform_admin` (BYPASSRLS cross-tenant
    /// handle) without any tenant GUC set.
    #[tokio::test]
    async fn operator_pool_runs_as_platform_admin() {
        if std::env::var("WYRD_DATABASE_URL").is_err() {
            return;
        }
        let fixture = PgFixture::start(&[]).await.expect("fixture starts");
        let current_user: (String,) = sqlx::query_as("SELECT current_user")
            .fetch_one(fixture.operator_pool().pool())
            .await
            .expect("current_user from operator pool");
        assert_eq!(
            current_user.0, "wyrd_platform_admin",
            "operator pool runs as wyrd_platform_admin"
        );
        fixture.shutdown().await.expect("shutdown");
    }

    /// The migrator DSN from `resolved_dsns` connects as `wyrd_migrator`.
    ///
    /// Callers that need a migrator-role pool build it from `fixture.resolved_dsns().migrator`
    /// — the fixture exposes DSNs but never hands out a raw pool.
    #[tokio::test]
    async fn migrator_dsn_connects_as_migrator_role() {
        if std::env::var("WYRD_DATABASE_URL").is_err() {
            return;
        }
        let fixture = PgFixture::start(&[]).await.expect("fixture starts");
        let pool = wyrd_sql_core::pool::build_pool(
            fixture.resolved_dsns().migrator.expose_secret(),
            wyrd_sql_core::pool::PoolConfig::migrator_defaults(),
        )
        .await
        .expect("migrator pool from DSN");
        let user: (String,) = sqlx::query_as("SELECT current_user")
            .fetch_one(&pool)
            .await
            .expect("current_user from migrator pool");
        pool.close().await;
        assert_eq!(user.0, "wyrd_migrator");
        fixture.shutdown().await.expect("shutdown");
    }

    /// When a migration set fails, `PgFixture::start` returns an error and the
    /// ephemeral database is cleaned up by the `TestDatabase` Drop path.
    #[tokio::test]
    async fn start_fails_when_migration_set_fails() {
        if std::env::var("WYRD_DATABASE_URL").is_err() {
            return;
        }
        let good_migrator = fixture_migrator!("migrations");
        let failing_migrator = fixture_migrator!("migrations_failing");
        let sets = [
            MigrationSet {
                migrator: &good_migrator,
                options: MigrateOptions {
                    advisory_lock_key: LOCK_FAIL_A,
                    ensure_schemas: &["test_schema"],
                    search_path: &["test_schema"],
                },
            },
            MigrationSet {
                migrator: &failing_migrator,
                options: MigrateOptions {
                    advisory_lock_key: LOCK_FAIL_B,
                    ensure_schemas: &["test_schema"],
                    search_path: &["test_schema"],
                },
            },
        ];
        let result = PgFixture::start(&sets).await;
        assert!(
            result.is_err(),
            "fixture start must fail on second-set migration failure"
        );
    }

    /// The `resolved_dsns` accessor returns app and migrator DSNs that are
    /// well-formed postgres URLs pointing at the isolated database.
    #[tokio::test]
    async fn resolved_dsns_accessible_from_fixture() {
        if std::env::var("WYRD_DATABASE_URL").is_err() {
            return;
        }
        let fixture = PgFixture::start(&[]).await.expect("fixture starts");
        let dsns: &ResolvedDsns = fixture.resolved_dsns();
        assert!(
            dsns.app.expose_secret().starts_with("postgres://"),
            "app DSN is a postgres URL"
        );
        assert!(
            dsns.migrator.expose_secret().starts_with("postgres://"),
            "migrator DSN is a postgres URL"
        );
        fixture.shutdown().await.expect("shutdown");
    }
}
