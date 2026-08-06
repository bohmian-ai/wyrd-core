#[cfg(feature = "pg")]
mod pg_tests {
    //! Live Postgres `run_migrations` behavior tests for `wyrd-sql-core`.
    //!
    //! Gated on the `pg` feature so the default test suite remains
    //! credential-free. Run via:
    //!
    //! ```text
    //! scripts/postgres/with-test-postgres.sh -- \
    //!     cargo test --locked -p wyrd-sql-core --features pg -- --test-threads=1
    //! ```
    //!
    //! The wrapper exports the DSN environment variables that
    //! `resolve_external_dsns_from_env` reads. Each test builds its own
    //! `PgFixture` so migration ledgers are isolated per test.

    use secrecy::ExposeSecret as _;
    use wyrd_dev_fixtures::pg::PgFixture;
    use wyrd_sql_core::migration::MigrateOptions;
    use wyrd_sql_core::pool::{PoolConfig, build_pool};
    use wyrd_sql_core::run_migrations;

    /// `run_migrations` applies the caller-supplied migrator and the resulting
    /// schema changes are visible to subsequent queries.
    #[tokio::test]
    async fn run_migrations_applies_supplied_migrator() {
        if std::env::var("WYRD_DATABASE_URL").is_err() {
            return;
        }

        let fixture = PgFixture::start(&[]).await.expect("fixture starts");
        let pool = build_pool(
            fixture.resolved_dsns().migrator.expose_secret(),
            PoolConfig::migrator_defaults(),
        )
        .await
        .expect("migrator pool");

        let migrator = sqlx::migrate::Migrator::new(std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/migrations"
        )))
        .await
        .expect("good migrator loaded");

        let options = MigrateOptions {
            advisory_lock_key: 0x5c01_0001_i64,
            ensure_schemas: &["test_schema"],
            search_path: &["test_schema"],
        };

        run_migrations(&pool, &migrator, &options)
            .await
            .expect("run_migrations succeeds");

        let exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS (
               SELECT 1 FROM pg_class c
               JOIN pg_namespace n ON n.oid = c.relnamespace
               WHERE n.nspname = 'test_schema' AND c.relname = 'sql_core_fixture_table'
             )",
        )
        .fetch_one(&pool)
        .await
        .expect("table existence query");
        assert!(exists.0, "sql_core_fixture_table exists after migration");

        pool.close().await;
        fixture.shutdown().await.expect("shutdown");
    }

    /// `run_migrations` is idempotent: running the same migrator twice against
    /// the same database succeeds without error.
    #[tokio::test]
    async fn run_migrations_is_idempotent_on_repeat() {
        if std::env::var("WYRD_DATABASE_URL").is_err() {
            return;
        }

        let fixture = PgFixture::start(&[]).await.expect("fixture starts");
        let pool = build_pool(
            fixture.resolved_dsns().migrator.expose_secret(),
            PoolConfig::migrator_defaults(),
        )
        .await
        .expect("migrator pool");

        let migrator = sqlx::migrate::Migrator::new(std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/migrations"
        )))
        .await
        .expect("good migrator loaded");

        let options = MigrateOptions {
            advisory_lock_key: 0x5c01_0002_i64,
            ensure_schemas: &["test_schema"],
            search_path: &["test_schema"],
        };

        run_migrations(&pool, &migrator, &options)
            .await
            .expect("first run_migrations succeeds");
        run_migrations(&pool, &migrator, &options)
            .await
            .expect("second run_migrations is idempotent");

        pool.close().await;
        fixture.shutdown().await.expect("shutdown");
    }

    /// `run_migrations` with an invalid-SQL migrator returns a migration error
    /// and releases the advisory lock, so a subsequent good-migrator run
    /// against the same isolated database succeeds.
    #[tokio::test]
    async fn run_migrations_returns_migration_error_and_releases_lock() {
        if std::env::var("WYRD_DATABASE_URL").is_err() {
            return;
        }

        let fixture = PgFixture::start(&[]).await.expect("fixture starts");
        let pool = build_pool(
            fixture.resolved_dsns().migrator.expose_secret(),
            PoolConfig::migrator_defaults(),
        )
        .await
        .expect("migrator pool");

        let failing_migrator = sqlx::migrate::Migrator::new(std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/migrations_failing"
        )))
        .await
        .expect("failing migrator loaded");
        let good_migrator = sqlx::migrate::Migrator::new(std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/migrations"
        )))
        .await
        .expect("good migrator loaded");

        let fail_options = MigrateOptions {
            advisory_lock_key: 0x5c01_0003_i64,
            ensure_schemas: &["test_schema"],
            search_path: &["test_schema"],
        };
        let good_options = MigrateOptions {
            advisory_lock_key: 0x5c01_0004_i64,
            ensure_schemas: &["test_schema"],
            search_path: &["test_schema"],
        };

        // First run: failing migrator must return a Migrate error.
        let err = run_migrations(&pool, &failing_migrator, &fail_options)
            .await
            .expect_err("failing migrator returns Err");
        assert!(
            matches!(err, wyrd_sql_core::SqlError::Migrate(_)),
            "expected SqlError::Migrate, got: {err:?}"
        );

        // Second run: good migrator succeeds, proving the advisory lock was
        // released after the failure.
        run_migrations(&pool, &good_migrator, &good_options)
            .await
            .expect("good migrator succeeds after lock released by failing run");

        pool.close().await;
        fixture.shutdown().await.expect("shutdown");
    }
}
