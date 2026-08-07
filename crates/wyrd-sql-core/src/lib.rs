//! Plane-neutral Postgres primitives for the Wyrd foundation.
//!
//! `wyrd-sql-core` owns the reusable Postgres building blocks that both the
//! Wyrd and Vala planes share: DSN resolution, role-specific pool construction,
//! the audited cross-tenant [`OperatorPool`], the tenant-scoped [`TenantConn`]
//! transaction wrapper, the [`SqlError`] catalog, and a generic
//! advisory-lock migration runner ([`run_migrations`]).
//!
//! Plane-specific stores, queries, row types, schema constants, and embedded
//! migration directories are intentionally excluded; those belong in the plane
//! crates that depend on this one.

#![deny(missing_docs)]

pub mod dsn;
pub mod error;
pub mod migration;
pub mod operator_pool;
pub mod pool;
pub mod tenant_conn;

// DSN API
pub use dsn::{
    APP_DSN_ENV, MIGRATOR_PASSWORD_ENV, PLATFORM_ADMIN_PASSWORD_ENV, ResolvedDsns, WYRD_APP_ROLE,
    WYRD_MIGRATOR_ROLE, WYRD_PLATFORM_ADMIN_ROLE, resolve_external_dsns,
    resolve_external_dsns_from_env, role_dsn, role_dsn_from_base,
};

// Pool API
pub use pool::{
    PoolConfig, build_app_pool, build_migrator_pool, build_platform_admin_pool, build_pool,
};

// Operator pool
pub use operator_pool::OperatorPool;

// Tenant-scoped connection
pub use tenant_conn::{BIND_CURRENT_TENANT_SQL, CURRENT_TENANT_GUC, TenantConn};

// Error catalog
pub use error::SqlError;

// Migration runner
pub use migration::{MigrateOptions, MigrationSet, run_migrations};
