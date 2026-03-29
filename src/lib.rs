#![forbid(unsafe_code)]
//! # Crudly
//! Derivable crud helpers for sqlx. Write your SELECT, INSERT, UPDATE, and DELETE queries once and reuse them for all your entities.
//!
//! Provides the `#[derive(IntoRow)]` and `#[derive(Crudly)]` macros.
//!
//! The public API lives at the crate root: `crudly::HasColumns`, `crudly::CRUDExecutor`, etc.
//!
//! ## Features
//!
//! - `derive`: Enables the `#[derive(Crudly)]` and `#[derive(IntoRow)]` macros.
//! - `postgres`: Enables default crud implementations for database.
//! - `mysql`: Enables default crud implementations for MySQL
//! - `sqlite`: Enables default crud implementations for sqlite
//! - `all-databases`: Enables default crud implementations all DBs above.
//! - `json`: Enables support for the `#[sqlx(json)]` / `#[crudly(json)]` attribute using [serde](https://crates.io/crates/serde).
//! 
//! ## MSRV
//! The minimum supported Rust version is 1.85.0 (the version that released edition 2024).

mod executor;
mod traits;

#[cfg(feature = "derive")]
pub use crudly_macros::{Crudly, IntoRow};

pub use executor::*;
pub use traits::*;
