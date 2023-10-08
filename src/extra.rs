//! # Support for extractors from `axum-extra`
//!
//! ## Feature
//!
//! Enable the `extra` feature to use `Valid<Cached<T>>`, `Valid<WithRejection<T, R>>` and `WithRejection<Valid<T>, R>`.
//!
//! ## Modules
//!
//! * [`self`] : `Cache<T>`
//! * [`self`] : `WithRejection<T, R>`
//! * [`form`] : `Form<T>`
//! * [`protobuf`] : `Protobuf<T>`
//! * [`query`] : `Query<T>`
//! * [`typed_path`] : `T: TypedPath`
//!
//! ## `Cached<T>` and `WithRejection<T, R>`
//!
//! ### `Valid<Cached<T>>`
//!
//! #### Usage
//!
//! 0. Implement your own extractor `T`.
//! 1. Implement `Clone` and `Validate` for your extractor type `T`.
//! 2. In your handler function, use `Valid<Cached<T>>` as some parameter's type.
//!
//! #### Example
//!
//! ```no_run
//! use axum::extract::FromRequestParts;
//! use axum::http::request::Parts;
//! use axum::response::{IntoResponse, Response};
//! use axum::routing::post;
//! use axum::Router;
//! use axum_extra::extract::Cached;
//! use axum_valid::Valid;
//! use validator::Validate;
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let router = Router::new().route("/cached", post(handler));
//!     axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
//!         .serve(router.into_make_service())
//!         .await?;
//!     Ok(())
//! }
//! async fn handler(Valid(Cached(parameter)): Valid<Cached<Parameter>>) {
//!     assert!(parameter.validate().is_ok());
//! }
//! #[derive(Validate, Clone)]
//! pub struct Parameter {
//!     #[validate(range(min = 5, max = 10))]
//!     pub v0: i32,
//!     #[validate(length(min = 1, max = 10))]
//!     pub v1: String,
//! }
//!
//! pub struct ParameterRejection;
//!
//! impl IntoResponse for ParameterRejection {
//!     fn into_response(self) -> Response {
//!         todo!()
//!     }
//! }
//!
//! #[axum::async_trait]
//! impl<S> FromRequestParts<S> for Parameter
//! where
//!     S: Send + Sync,
//! {
//!     type Rejection = ParameterRejection;
//!
//!     async fn from_request_parts(_parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
//!         todo!()
//!     }
//! }
//! ```
//!
//! ### `Valid<WithRejection<T, R>>`
//!
//! #### Usage
//!
//! 0. Implement your own extractor `T` and rejection type `R`.
//! 1. Implement `Validate` for your extractor type `T`.
//! 2. In your handler function, use `Valid<WithRejection<T, R>>` as some parameter's type.
//!
//! #### Example
//!
//! ```no_run
//! use axum::extract::FromRequestParts;
//! use axum::http::request::Parts;
//! use axum::http::StatusCode;
//! use axum::response::{IntoResponse, Response};
//! use axum::routing::post;
//! use axum::Router;
//! use axum_extra::extract::WithRejection;
//! use axum_valid::Valid;
//! use validator::Validate;
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let router = Router::new().route("/valid_with_rejection", post(handler));
//!     axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
//!         .serve(router.into_make_service())
//!         .await?;
//!     Ok(())
//! }
//! async fn handler(
//!     Valid(WithRejection(parameter, _)): Valid<
//!         WithRejection<Parameter, ValidWithRejectionRejection>,
//!     >,
//! ) {
//!     assert!(parameter.validate().is_ok());
//! }
//!
//! #[derive(Validate)]
//! pub struct Parameter {
//!     #[validate(range(min = 5, max = 10))]
//!     pub v0: i32,
//!     #[validate(length(min = 1, max = 10))]
//!     pub v1: String,
//! }
//!
//! pub struct ValidWithRejectionRejection;
//!
//! impl IntoResponse for ValidWithRejectionRejection {
//!     fn into_response(self) -> Response {
//!         StatusCode::BAD_REQUEST.into_response()
//!     }
//! }
//!
//! #[axum::async_trait]
//! impl<S> FromRequestParts<S> for Parameter
//! where
//!     S: Send + Sync,
//! {
//!     type Rejection = ValidWithRejectionRejection;
//!
//!     async fn from_request_parts(_parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
//!         todo!()
//!     }
//! }
//! ```
//!
//! ### `WithRejection<Valid<T>, R>`
//!
//! #### Usage
//!
//! 0. Implement your own extractor `T` and rejection type `R`.
//! 1. Implement `Validate` and `HasValidate` for your extractor type `T`.
//! 2. Implement `From<ValidRejection<T::Rejection>>` for `R`.
//! 3. In your handler function, use `WithRejection<Valid<T>, R>` as some parameter's type.
//!
//! #### Example
//!
//! ```no_run
//! use axum::extract::FromRequestParts;
//! use axum::http::request::Parts;
//! use axum::response::{IntoResponse, Response};
//! use axum::routing::post;
//! use axum::Router;
//! use axum_extra::extract::WithRejection;
//! use axum_valid::{HasValidate, Valid, ValidRejection};
//! use validator::Validate;
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let router = Router::new().route("/with_rejection_valid", post(handler));
//!     axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
//!         .serve(router.into_make_service())
//!         .await?;
//!     Ok(())
//! }
//! async fn handler(
//!     WithRejection(Valid(parameter), _): WithRejection<
//!         Valid<Parameter>,
//!         WithRejectionValidRejection,
//!     >,
//! ) {
//!     assert!(parameter.validate().is_ok());
//! }
//!
//! #[derive(Validate)]
//! pub struct Parameter {
//!     #[validate(range(min = 5, max = 10))]
//!     pub v0: i32,
//!     #[validate(length(min = 1, max = 10))]
//!     pub v1: String,
//! }
//!
//! impl HasValidate for Parameter {
//!     type Validate = Self;
//!
//!     fn get_validate(&self) -> &Self::Validate {
//!         self
//!     }
//! }
//!
//! pub struct ParameterRejection;
//!
//! impl IntoResponse for ParameterRejection {
//!     fn into_response(self) -> Response {
//!         todo!()
//!     }
//! }
//!
//! #[axum::async_trait]
//! impl<S> FromRequestParts<S> for Parameter
//! where
//!     S: Send + Sync,
//! {
//!     type Rejection = ParameterRejection;
//!
//!     async fn from_request_parts(_parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
//!         todo!()
//!     }
//! }
//!
//! pub struct WithRejectionValidRejection;
//!
//! impl From<ValidRejection<ParameterRejection>> for WithRejectionValidRejection {
//!     fn from(_inner: ValidRejection<ParameterRejection>) -> Self {
//!         todo!()
//!     }
//! }
//!
//! impl IntoResponse for WithRejectionValidRejection {
//!     fn into_response(self) -> Response {
//!         todo!()
//!     }
//! }
//! ```

#[cfg(feature = "extra_form")]
pub mod form;
#[cfg(feature = "extra_protobuf")]
pub mod protobuf;
#[cfg(feature = "extra_query")]
pub mod query;
#[cfg(feature = "extra_typed_path")]
pub mod typed_path;

use crate::HasValidate;
#[cfg(feature = "validator")]
use crate::HasValidateArgs;
use axum_extra::extract::{Cached, WithRejection};
#[cfg(feature = "validator")]
use validator::ValidateArgs;

impl<T> HasValidate for Cached<T> {
    type Validate = T;

    fn get_validate(&self) -> &Self::Validate {
        &self.0
    }
}

#[cfg(feature = "validator")]
impl<'v, T: ValidateArgs<'v>> HasValidateArgs<'v> for Cached<T> {
    type ValidateArgs = T;
    fn get_validate_args(&self) -> &Self::ValidateArgs {
        &self.0
    }
}

impl<T, R> HasValidate for WithRejection<T, R> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

#[cfg(feature = "validator")]
impl<'v, T: ValidateArgs<'v>, R> HasValidateArgs<'v> for WithRejection<T, R> {
    type ValidateArgs = T;
    fn get_validate_args(&self) -> &Self::ValidateArgs {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{Rejection, ValidTest};
    #[cfg(feature = "validator")]
    use crate::Valid;
    use axum::http::StatusCode;
    use axum_extra::extract::{Cached, WithRejection};
    use reqwest::RequestBuilder;

    impl<T: ValidTest> ValidTest for Cached<T> {
        const ERROR_STATUS_CODE: StatusCode = T::ERROR_STATUS_CODE;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            T::set_valid_request(builder)
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            // cached never fails
            T::set_error_request(builder)
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            T::set_invalid_request(builder)
        }
    }

    impl<T: ValidTest, R: Rejection> ValidTest for WithRejection<T, R> {
        const ERROR_STATUS_CODE: StatusCode = R::STATUS_CODE;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            T::set_valid_request(builder)
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            // cached never fails
            T::set_error_request(builder)
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            T::set_invalid_request(builder)
        }
    }

    #[cfg(feature = "validator")]
    impl<T: ValidTest, R> ValidTest for WithRejection<Valid<T>, R> {
        // just use `418 I'm a teapot` to test
        const ERROR_STATUS_CODE: StatusCode = StatusCode::IM_A_TEAPOT;
        // If `WithRejection` is the outermost extractor,
        // the error code returned will always be the one provided by WithRejection.
        const INVALID_STATUS_CODE: StatusCode = StatusCode::IM_A_TEAPOT;
        // If `WithRejection` is the outermost extractor,
        // the returned body may not be in JSON format.
        const JSON_SERIALIZABLE: bool = false;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            T::set_valid_request(builder)
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            // invalid requests will cause the Valid extractor to fail.
            T::set_invalid_request(builder)
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            T::set_invalid_request(builder)
        }
    }
}
