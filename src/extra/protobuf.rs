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
//! #[cfg(feature = "validator")]
//! mod validator_example {
//!     use axum::routing::post;
//!     use axum_extra::protobuf::Protobuf;
//!     use axum::Router;
//!     use axum_valid::Valid;
//!     use serde::Deserialize;
//!     use validator::Validate;
//!
//!     pub fn router() -> Router {
//!         Router::new().route("/protobuf", post(handler))
//!     }
//!
//!     async fn handler(Valid(Protobuf(parameter)): Valid<Protobuf<Parameter>>) {
//!         assert!(parameter.validate().is_ok());
//!         // Support automatic dereferencing
//!         println!("v0 = {}, v1 = {}", parameter.v0, parameter.v1);
//!     }
//!
//!     #[derive(Validate, prost::Message)]
//!     pub struct Parameter {
//!         #[validate(range(min = 5, max = 10))]
//!         #[prost(int32, tag = "1")]
//!         pub v0: i32,
//!         #[validate(length(min = 1, max = 10))]
//!         #[prost(string, tag = "2")]
//!         pub v1: String,
//!     }
//! }
//!
//! #[cfg(feature = "garde")]
//! mod garde_example {
//!     use axum::routing::post;
//!     use axum::Router;
//!     use axum_extra::protobuf::Protobuf;
//!     use axum_valid::Garde;
//!     use serde::Deserialize;
//!     use garde::Validate;
//!
//!     pub fn router() -> Router {
//!         Router::new().route("/protobuf", post(handler))
//!     }
//!
//!     async fn handler(Garde(Protobuf(parameter)): Garde<Protobuf<Parameter>>) {
//!         assert!(parameter.validate_with(&()).is_ok());
//!         // Support automatic dereferencing
//!         println!("v0 = {}, v1 = {}", parameter.v0, parameter.v1);
//!     }
//!
//!     #[derive(Validate, prost::Message)]
//!     pub struct Parameter {
//!         #[garde(range(min = 5, max = 10))]
//!         #[prost(int32, tag = "1")]
//!         pub v0: i32,
//!         #[garde(length(min = 1, max = 10))]
//!         #[prost(string, tag = "2")]
//!         pub v1: String,
//!     }
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! #     use std::net::SocketAddr;
//! #     use axum::Router;
//! #     use tokio::net::TcpListener;
//! #     let router = Router::new();
//! #     #[cfg(feature = "validator")]
//! #     let router = router.nest("/validator", validator_example::router());
//! #     #[cfg(feature = "garde")]
//! #     let router = router.nest("/garde", garde_example::router());
//! #     let listener = TcpListener::bind(&SocketAddr::from(([0u8, 0, 0, 0], 0u16))).await?;
//! #     axum::serve(listener, router.into_make_service())
//! #         .await?;
//! #     Ok(())
//! # }
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

#[cfg(feature = "validify")]
impl<T: validify::Modify> crate::HasModify for Protobuf<T> {
    type Modify = T;

    fn get_modify(&mut self) -> &mut Self::Modify {
        &mut self.0
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
