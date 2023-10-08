//! # Support for `MsgPack<T>` and `MsgPackRaw<T>` from `axum-msgpack`
//!
//! ## Feature
//!
//! Enable the `msgpack` feature to use `Valid<MsgPack<T>>` and `Valid<MsgPackRaw<T>>`.
//!
//! ## Usage
//!
//! 1. Implement `Deserialize` and `Validate` for your data type `T`.
//! 2. In your handler function, use `Valid<MsgPack<T>>` or `Valid<MsgPackRaw<T>>` as some parameter's type.
//!
//! ## Example
//!
//! ```no_run
//!#![cfg(feature = "validator")]
//!
//! use axum::routing::post;
//! use axum::Router;
//! use axum_msgpack::{MsgPack, MsgPackRaw};
//! use axum_valid::Valid;
//! use serde::Deserialize;
//! use validator::Validate;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let router = Router::new()
//!         .route("/msgpack", post(handler))
//!         .route("/msgpackraw", post(raw_handler));
//!     axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
//!         .serve(router.into_make_service())
//!         .await?;
//!     Ok(())
//! }
//!
//! async fn handler(Valid(MsgPack(parameter)): Valid<MsgPack<Parameter>>) {
//!     assert!(parameter.validate().is_ok());
//! }
//!
//! async fn raw_handler(Valid(MsgPackRaw(parameter)): Valid<MsgPackRaw<Parameter>>) {
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

use crate::HasValidate;
#[cfg(feature = "validator")]
use crate::HasValidateArgs;
use axum_msgpack::{MsgPack, MsgPackRaw};
#[cfg(feature = "validator")]
use validator::ValidateArgs;

impl<T> HasValidate for MsgPack<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

#[cfg(feature = "validator")]
impl<'v, T: ValidateArgs<'v>> HasValidateArgs<'v> for MsgPack<T> {
    type ValidateArgs = T;
    fn get_validate_args(&self) -> &Self::ValidateArgs {
        &self.0
    }
}

impl<T> HasValidate for MsgPackRaw<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

#[cfg(feature = "validator")]
impl<'v, T: ValidateArgs<'v>> HasValidateArgs<'v> for MsgPackRaw<T> {
    type ValidateArgs = T;
    fn get_validate_args(&self) -> &Self::ValidateArgs {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum::http::StatusCode;
    use axum_msgpack::{MsgPack, MsgPackRaw};
    use reqwest::RequestBuilder;
    use serde::Serialize;

    impl<T: ValidTestParameter + Serialize> ValidTest for MsgPack<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::BAD_REQUEST;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder
                .header(reqwest::header::CONTENT_TYPE, "application/msgpack")
                .body(
                    rmp_serde::to_vec_named(T::valid())
                        .expect("Failed to serialize parameters to msgpack"),
                )
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            // `Content-Type` not set, `MsgPack` should return `415 Unsupported Media Type`
            builder
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder
                .header(reqwest::header::CONTENT_TYPE, "application/msgpack")
                .body(
                    rmp_serde::to_vec_named(T::invalid())
                        .expect("Failed to serialize parameters to msgpack"),
                )
        }
    }

    impl<T: ValidTestParameter + Serialize> ValidTest for MsgPackRaw<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::BAD_REQUEST;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder
                .header(reqwest::header::CONTENT_TYPE, "application/msgpack")
                .body(
                    rmp_serde::to_vec(T::valid())
                        .expect("Failed to serialize parameters to msgpack"),
                )
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            // `Content-Type` not set, `MsgPack` should return `415 Unsupported Media Type`
            builder
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder
                .header(reqwest::header::CONTENT_TYPE, "application/msgpack")
                .body(
                    rmp_serde::to_vec(T::invalid())
                        .expect("Failed to serialize parameters to msgpack"),
                )
        }
    }
}
