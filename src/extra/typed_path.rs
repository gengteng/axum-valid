//! # Support for `T: TypedPath` from `axum-extra`
//!
//! ## Feature
//!
//! Enable the `extra_typed_path` feature to use `Valid<T: TypedPath>`.
//!
//! ## Usage
//!
//! 1. Implement `TypedPath`, `Deserialize`, `Validate` and `HasValidate` for your data type `T`.
//! 2. In your handler function, use `Valid<T>` as some parameter's type.
//!
//! ## Example
//!
//! ```no_run
//! use axum::Router;
//! use axum_extra::routing::{RouterExt, TypedPath};
//! use axum_valid::{HasValidate, Valid};
//! use serde::Deserialize;
//! use validator::Validate;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let router = Router::new().typed_get(handler);
//!     axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
//!         .serve(router.into_make_service())
//!         .await?;
//!     Ok(())
//! }
//!
//! async fn handler(parameter: Valid<Parameter>) {
//!     assert!(parameter.validate().is_ok());
//! }
//!
//! #[derive(TypedPath, Deserialize, Validate)]
//! #[typed_path("/extra_typed_path/:v0/:v1")]
//! struct Parameter {
//!     #[validate(range(min = 5, max = 10))]
//!     v0: i32,
//!     #[validate(length(min = 1, max = 10))]
//!     v1: String,
//! }
//!
//! impl HasValidate for Parameter {
//!     type Validate = Self;
//!
//!     fn get_validate(&self) -> &Self::Validate {
//!         self
//!     }
//! }
//! ```

use crate::garde::Garde;
use crate::{Valid, ValidEx};
use axum_extra::routing::TypedPath;
use std::fmt::Display;

impl<T: TypedPath + Display> TypedPath for Valid<T> {
    const PATH: &'static str = T::PATH;
}

impl<T: TypedPath + Display, A> TypedPath for ValidEx<T, A> {
    const PATH: &'static str = T::PATH;
}

impl<T: TypedPath + Display> TypedPath for Garde<T> {
    const PATH: &'static str = T::PATH;
}
