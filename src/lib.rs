//! # Crudly
//! Derivable crud helpers for sqlx. Write your SELECT, INSERT, UPDATE, DELETE queries once and reuse them for all your entities.
//!
//! Provides the `#[derive(IntoRow)]` and `#[derive(Crudly)]` macros.
//!
//! ## Features
//!
//! - `derive`: Enables the `#[derive(Crudly)]` and `#[derive(IntoRow)]` macros.
//! - `postgres`: Enables default crud implementations for database.
//! - `mysql`: Enables default crud implementations for MySQL
//! - `sqlite`: Enables default crud implementations for sqlite
//! - `all-databases`: Enables default crud implementations all DBs above.
//! - `json`: Enables support for the `#[sqlx(json)]` / `#[crudly(json)]` attribute using [serde](https://crates.io/crates/serde).
mod crud_executor;
mod traits;

#[cfg(feature = "derive")]
pub use crudly_macros::{Crudly, IntoRow};

pub use crud_executor::*;
pub use traits::*;
