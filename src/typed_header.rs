//! # Support for `TypedHeader<T>`
//!
//! ## Feature
//!
//! Enable the `typed_header` feature to use `Valid<TypedHeader<T>>`.
//!
//! ## Usage
//!
//! 1. Implement `Header` and `Validate` for your data type `T`.
//! 2. In your handler function, use `Valid<TypedHeader<T>>` as some parameter's type.
//!
//! ## Example
//!
//! ```no_run
//! use axum::headers::{Error, Header, HeaderValue};
//! use axum::http::HeaderName;
//! use axum::routing::post;
//! use axum::{Router, TypedHeader};
//! use axum_valid::Valid;
//! use validator::Validate;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let router = Router::new().route("/typed_header", post(handler));
//!     axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
//!         .serve(router.into_make_service())
//!         .await?;
//!     Ok(())
//! }
//!
//! async fn handler(Valid(TypedHeader(parameter)): Valid<TypedHeader<Parameter>>) {
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
//! static HEADER_NAME: HeaderName = HeaderName::from_static("my-header");
//!
//! impl Header for Parameter {
//!     fn name() -> &'static HeaderName {
//!         &HEADER_NAME
//!     }
//!
//!     fn decode<'i, I>(_values: &mut I) -> Result<Self, Error>
//!     where
//!         Self: Sized,
//!         I: Iterator<Item = &'i HeaderValue>,
//!     {
//!         todo!()
//!     }
//!
//!     fn encode<E: Extend<HeaderValue>>(&self, _values: &mut E) {
//!         todo!()
//!     }
//! }
//! ```

use crate::{HasValidate, HasValidateArgs};
use axum::TypedHeader;
use validator::{Validate, ValidateArgs};

impl<T: Validate> HasValidate for TypedHeader<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

impl<'v, T: ValidateArgs<'v>> HasValidateArgs<'v> for TypedHeader<T> {
    type ValidateArgs = T;
    fn get_validate_args(&self) -> &Self::ValidateArgs {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum::headers::{Header, HeaderMapExt};
    use axum::http::StatusCode;
    use axum::TypedHeader;
    use reqwest::header::HeaderMap;
    use reqwest::RequestBuilder;

    impl<T: ValidTestParameter + Header + Clone> ValidTest for TypedHeader<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::BAD_REQUEST;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            let mut headers = HeaderMap::default();
            headers.typed_insert(T::valid().clone());
            builder.headers(headers)
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            builder
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            let mut headers = HeaderMap::default();
            headers.typed_insert(T::invalid().clone());
            builder.headers(headers)
        }
    }
}
