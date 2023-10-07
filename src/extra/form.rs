//! # Support for `Form<T>` from `axum-extra`
//!
//! ## Feature
//!
//! Enable the `extra_form` feature to use `Valid<Form<T>>`.
//!
//! ## Usage
//!
//! 1. Implement `Deserialize` and `Validate` for your data type `T`.
//! 2. In your handler function, use `Valid<Form<T>>` as some parameter's type.
//!
//! ## Example
//!
//! ```no_run
//! use axum::routing::post;
//! use axum::Router;
//! use axum_extra::extract::Form;
//! use axum_valid::Valid;
//! use serde::Deserialize;
//! use validator::Validate;
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let router = Router::new().route("/extra_form", post(handler));
//!     axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
//!         .serve(router.into_make_service())
//!         .await?;
//!     Ok(())
//! }
//! async fn handler(Valid(Form(parameter)): Valid<Form<Parameter>>) {
//!     assert!(parameter.validate().is_ok());
//! }
//! #[derive(Validate, Deserialize)]
//! pub struct Parameter {
//!     #[validate(range(min = 5, max = 10))]
//!     pub v0: i32,
//!     #[validate(length(min = 1, max = 10))]
//!     pub v1: String,
//! }
//! ```

use crate::{HasValidate, HasValidateArgs};
use axum_extra::extract::Form;
use validator::ValidateArgs;

impl<T> HasValidate for Form<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

impl<'v, T: ValidateArgs<'v>> HasValidateArgs<'v> for Form<T> {
    type ValidateArgs = T;
    fn get_validate_args(&self) -> &Self::ValidateArgs {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum_extra::extract::Form;
    use reqwest::{RequestBuilder, StatusCode};

    impl<T: ValidTestParameter> ValidTest for Form<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::BAD_REQUEST;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.form(T::valid())
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            builder.form(T::error())
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.form(T::invalid())
        }
    }
}
