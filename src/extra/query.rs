//! # Support for `Query<T>` from `axum-extra`
//!
//! ## Feature
//!
//! Enable the `extra_query` feature to use `Valid<Query<T>>`.
//!
//! ## Usage
//!
//! 1. Implement `Deserialize` and `Validate` for your data type `T`.
//! 2. In your handler function, use `Valid<Query<T>>` as some parameter's type.
//!
//! ## Example
//!
//! ```no_run
//! use axum::routing::post;
//! use axum::Router;
//! use axum_extra::extract::Query;
//! use axum_valid::Valid;
//! use serde::Deserialize;
//! use validator::Validate;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let router = Router::new().route("/extra_query", post(handler));
//!     axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
//!         .serve(router.into_make_service())
//!         .await?;
//!     Ok(())
//! }
//!
//! async fn handler(Valid(Query(parameter)): Valid<Query<Parameter>>) {
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
use axum_extra::extract::Query;
use validator::{Validate, ValidateArgs};

impl<T: Validate> HasValidate for Query<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

impl<'v, T: ValidateArgs<'v>> HasValidateArgs<'v> for Query<T> {
    type ValidateArgs = T;
    fn get_validate_args(&self) -> &Self::ValidateArgs {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum::http::StatusCode;
    use axum_extra::extract::Query;
    use reqwest::RequestBuilder;

    impl<T: ValidTestParameter> ValidTest for Query<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::BAD_REQUEST;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.query(&T::valid())
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            builder.query(T::error())
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.query(&T::invalid())
        }
    }
}
