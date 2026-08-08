//! Plane-neutral Postgres fixture primitives for Wyrd SQL integration tests.
//!
//! The `pg` feature gates the [`pg`] module, which provides [`pg::PgFixture`]:
//! an isolated per-test database with configurable ordered migration sets.
//! All seed logic, plane handles, and tenant insertion live in the callers,
//! not in this crate.

#[cfg(feature = "pg")]
pub mod pg;
