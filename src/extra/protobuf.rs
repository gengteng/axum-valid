//! # Support for `Protobuf<T>` from `axum-extra`
//!
//! ## Feature
//!
//! Enable the `extra_protobuf` feature to use `Valid<Protobuf<T>>`.
//!
//! ## Usage
//!
//! 1. Implement `prost::Message` and `Validate` for your data type `T`.
//! 2. In your handler function, use `Valid<Protobuf<T>>` as some parameter's type.
//!
//! ## Example
//!
//! ```no_run
//! use axum::routing::post;
//! use axum::Router;
//! use axum_extra::protobuf::Protobuf;
//! use axum_valid::Valid;
//! use validator::Validate;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let router = Router::new().route("/protobuf", post(handler));
//!     axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
//!         .serve(router.into_make_service())
//!         .await?;
//!     Ok(())
//! }
//! async fn handler(Valid(Protobuf(parameter)): Valid<Protobuf<Parameter>>) {
//!     assert!(parameter.validate().is_ok());
//! }
//! #[derive(Validate, prost::Message)]
//! pub struct Parameter {
//!     #[validate(range(min = 5, max = 10))]
//!     #[prost(int32, tag = "1")]
//!     pub v0: i32,
//!     #[validate(length(min = 1, max = 10))]
//!     #[prost(string, tag = "2")]
//!     pub v1: String,
//! }
//! ```

use crate::HasValidate;
#[cfg(feature = "validator")]
use crate::HasValidateArgs;
use axum_extra::protobuf::Protobuf;
#[cfg(feature = "validator")]
use validator::ValidateArgs;

impl<T> HasValidate for Protobuf<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

#[cfg(feature = "validator")]
impl<'v, T: ValidateArgs<'v>> HasValidateArgs<'v> for Protobuf<T> {
    type ValidateArgs = T;
    fn get_validate_args(&self) -> &Self::ValidateArgs {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum::http::StatusCode;
    use axum_extra::protobuf::Protobuf;
    use reqwest::RequestBuilder;

    impl<T: ValidTestParameter + prost::Message> ValidTest for Protobuf<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::UNPROCESSABLE_ENTITY;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.body(T::valid().encode_to_vec())
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            builder.body("invalid protobuf")
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.body(T::invalid().encode_to_vec())
        }
    }
}
