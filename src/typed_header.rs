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
//! #[cfg(feature = "validator")]
//! mod validator_example {
//!     use axum::headers::{Error, Header, HeaderValue};
//!     use axum::http::HeaderName;
//!     use axum::routing::post;
//!     use axum::{Router, TypedHeader};
//!     use axum_valid::Valid;
//!     use validator::Validate;
//!
//!     pub fn router() -> Router {
//!         Router::new().route("/typed_header", post(handler))
//!     }
//!
//!     async fn handler(Valid(TypedHeader(parameter)): Valid<TypedHeader<Parameter>>) {
//!         assert!(parameter.validate().is_ok());
//!     }
//!
//!     #[derive(Validate)]
//!     pub struct Parameter {
//!         #[validate(range(min = 5, max = 10))]
//!         pub v0: i32,
//!         #[validate(length(min = 1, max = 10))]
//!         pub v1: String,
//!     }
//!
//!     static HEADER_NAME: HeaderName = HeaderName::from_static("my-header");
//!
//!     impl Header for Parameter {
//!         fn name() -> &'static HeaderName {
//!             &HEADER_NAME
//!         }
//!
//!         fn decode<'i, I>(_values: &mut I) -> Result<Self, Error>
//!         where
//!             Self: Sized,
//!             I: Iterator<Item = &'i HeaderValue>,
//!         {
//!             todo!()
//!         }
//!
//!         fn encode<E: Extend<HeaderValue>>(&self, _values: &mut E) {
//!             todo!()
//!         }
//!     }
//! }
//!
//! #[cfg(feature = "garde")]
//! mod garde_example {
//!     use axum::headers::{Error, Header, HeaderValue};
//!     use axum::http::HeaderName;
//!     use axum::routing::post;
//!     use axum::{Router, TypedHeader};
//!     use axum_valid::Garde;
//!     use garde::Validate;
//!
//!     pub fn router() -> Router {
//!         Router::new().route("/typed_header", post(handler))
//!     }
//!
//!     async fn handler(Garde(TypedHeader(parameter)): Garde<TypedHeader<Parameter>>) {
//!         assert!(parameter.validate(&()).is_ok());
//!     }
//!
//!     #[derive(Validate)]
//!     pub struct Parameter {
//!         #[garde(range(min = 5, max = 10))]
//!         pub v0: i32,
//!         #[garde(length(min = 1, max = 10))]
//!         pub v1: String,
//!     }
//!
//!     static HEADER_NAME: HeaderName = HeaderName::from_static("my-header");
//!
//!     impl Header for Parameter {
//!         fn name() -> &'static HeaderName {
//!             &HEADER_NAME
//!         }
//!
//!         fn decode<'i, I>(_values: &mut I) -> Result<Self, Error>
//!         where
//!             Self: Sized,
//!             I: Iterator<Item = &'i HeaderValue>,
//!         {
//!             todo!()
//!         }
//!
//!         fn encode<E: Extend<HeaderValue>>(&self, _values: &mut E) {
//!             todo!()
//!         }
//!     }
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! #     use axum::Router;
//! #     let router = Router::new();
//! #     #[cfg(feature = "validator")]
//! #     let router = router.nest("/validator", validator_example::router());
//! #     #[cfg(feature = "garde")]
//! #     let router = router.nest("/garde", garde_example::router());
//! #     axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
//! #         .serve(router.into_make_service())
//! #         .await?;
//! #     Ok(())
//! # }
//! ```

use crate::HasValidate;
#[cfg(feature = "validator")]
use crate::HasValidateArgs;
use axum::TypedHeader;
#[cfg(feature = "validator")]
use validator::ValidateArgs;

impl<T> HasValidate for TypedHeader<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

#[cfg(feature = "validator")]
impl<'v, T: ValidateArgs<'v>> HasValidateArgs<'v> for TypedHeader<T> {
    type ValidateArgs = T;
    fn get_validate_args(&self) -> &Self::ValidateArgs {
        &self.0
    }
}

#[cfg(feature = "validify")]
impl<T: validify::Modify> crate::HasModify for TypedHeader<T> {
    type Modify = T;

    fn get_modify(&mut self) -> &mut Self::Modify {
        &mut self.0
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
