//! # Support for `Path<T>`
//!
//! ## Usage
//!
//! 1. Implement `Deserialize` and `Validate` for your data type `T`.
//! 2. In your handler function, use `Valid<Path<T>>` as some parameter's type.
//!
//! ## Example
//!
//! ```no_run
//! use axum::extract::Path;
//! use axum::routing::post;
//! use axum::Router;
//! use axum_valid::Valid;
//! use serde::Deserialize;
//! use validator::Validate;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let router = Router::new().route("/path/:v0/:v1", post(handler));
//!     axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
//!         .serve(router.into_make_service())
//!         .await?;
//!     Ok(())
//! }
//!
//! async fn handler(Valid(Path(parameter)): Valid<Path<Parameter>>) {
//!     assert!(parameter.validate().is_ok());
//! }
//!
//! #[derive(Validate, Deserialize)]
//! pub struct Parameter {
//!     #[validate(range(min = 5, max = 10))]
//!     pub v0: i32,
//!     #[validate(length(min = 1, max = 10))]
//!     pub v1: String,
//! }
//! ```

use crate::{HasValidate, HasValidateArgs};
use axum::extract::Path;
use validator::{Validate, ValidateArgs};

impl<T: Validate> HasValidate for Path<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

impl<'v, T: ValidateArgs<'v>> HasValidateArgs<'v> for Path<T> {
    type ValidateArgs = T;
    fn get_validate_args(&self) -> &Self::ValidateArgs {
        &self.0
    }
}
