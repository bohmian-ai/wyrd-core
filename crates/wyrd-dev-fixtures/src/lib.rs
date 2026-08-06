//! Development fixture crate.
//!
//! SQL fixtures are feature-gated so non-Postgres fixture users do not pull in
//! embedded database dependencies.
//!
//! The `pg` feature and its supporting modules (`pg`, `cards`) are rebuilt in T3
//! with `wyrd-sql-core` to remove plane-specific dependencies. This stub is the
//! T1 snapshot placeholder.
